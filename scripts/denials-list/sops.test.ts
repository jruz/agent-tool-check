import { expect, test } from "bun:test";
import { sopsReference } from "./sops.ts";
import { readTranscript } from "./transcripts.ts";

const timestamp = "2026-10-30T12:00:00Z";
const claude = (command: string, results: readonly unknown[]) =>
  readTranscript({
    agent: "claude",
    source: "/session.jsonl",
    content: [
      {
        type: "assistant",
        timestamp,
        cwd: "/project",
        sessionId: "session",
        message: {
          content: [
            { type: "tool_use", id: "call", name: "Bash", input: { command } },
          ],
        },
      },
      {
        type: "user",
        timestamp,
        cwd: "/project",
        sessionId: "session",
        message: { content: results },
      },
    ]
      .map((record) => JSON.stringify(record))
      .join("\n"),
  });

test.each([
  "sops -d private.yaml",
  "/usr/bin/sops --decrypt private.yaml",
  "sandbox bash -c 'sops decrypt private.yaml'",
  "echo $(sops -d private.yaml)",
  "'sops' '-d' private.yaml",
  "rg 'sops --decrypt' src",
])(
  "finds SOPS token candidates without claiming shell execution: %s",
  (command) => {
    const reference = sopsReference({ command });
    expect(reference).toEqual({ match: "sops-token", decryptReference: true });
  },
);

test("separates recipe references and ignores unrelated commands", () => {
  const recipe = sopsReference({ cmd: "just sops-decrypt-dev" });
  const version = sopsReference({ command: "sops --version" });
  const unrelated = sopsReference({ command: "echo sopsish" });
  const invalid = sopsReference({ command: 123 });
  expect(recipe).toEqual({ match: "recipe-reference", decryptReference: true });
  expect(version?.decryptReference).toBe(false);
  expect(unrelated).toBeNull();
  expect(invalid).toBeNull();
});

test.each([
  {
    content: "PreToolUse:Bash hook error: Never print environment variables",
    is_error: true,
    expected: "hook-denied",
  },
  { content: "plaintext-canary", is_error: false, expected: "tool-result" },
  { content: "plaintext-canary", is_error: true, expected: "tool-error" },
  {
    content: [{ type: "text", text: "plaintext-canary" }],
    is_error: false,
    expected: "tool-result",
  },
])(
  "joins Claude outcomes without exposing input or output: $expected",
  ({ content, is_error, expected }) => {
    const report = claude("sops -d private-file-canary.yaml", [
      { type: "tool_result", tool_use_id: "call", content, is_error },
    ]);
    expect(report.sopsCalls[0]?.outcome).toBe(expected);
    const serialized = JSON.stringify(report.sopsCalls);
    expect(serialized).not.toContain("plaintext-canary");
    expect(serialized).not.toContain("private-file-canary");
  },
);

test("does not count missing results as successful execution", () => {
  const report = claude("sops decrypt private.yaml", []);
  expect(report.sopsCalls[0]?.outcome).toBe("missing-result");
});

test.each(["function", "custom_tool"])(
  "joins Codex %s results without printing their output",
  (kind) => {
    const content = [
      { type: "session_meta", payload: { id: "session", cwd: "/project" } },
      {
        type: "response_item",
        timestamp,
        payload: {
          type: `${kind}_call`,
          call_id: "call",
          name: "exec_command",
          ...(kind === "function"
            ? {
                arguments: JSON.stringify({
                  cmd: "sops decrypt private-file-canary.yaml",
                }),
              }
            : {
                input: JSON.stringify({
                  cmd: "sops decrypt private-file-canary.yaml",
                }),
              }),
        },
      },
      {
        type: "response_item",
        timestamp,
        payload: {
          type: `${kind}_call_output`,
          call_id: "call",
          output: "plaintext-canary",
        },
      },
    ]
      .map((record) => JSON.stringify(record))
      .join("\n");
    const report = readTranscript({
      agent: "codex",
      source: "/session.jsonl",
      content,
    });
    expect(report.sopsCalls[0]?.outcome).toBe("tool-result");
    const serialized = JSON.stringify(report);
    expect(serialized).not.toContain("canary");
  },
);
