import { expect, test } from "bun:test";
import { readTranscript } from "./transcripts.ts";

const timestamp = "2026-10-31T12:00:00Z";
const session = { type: "session", id: "pi-session", cwd: "/project" };
const call = (id: string, name: string, args: unknown) => ({
  type: "message",
  timestamp,
  message: {
    role: "assistant",
    content: [{ type: "toolCall", id, name, arguments: args }],
  },
});
const result = (id: string, isError: boolean) => ({
  type: "message",
  timestamp,
  message: {
    role: "toolResult",
    toolCallId: id,
    toolName: "bash",
    isError,
    content: [{ type: "text", text: "secret-output-canary" }],
  },
});
const read = (records: readonly unknown[]) =>
  readTranscript({
    agent: "pi",
    source: "/session.jsonl",
    content: records.map((record) => JSON.stringify(record)).join("\n"),
  });

test("reports Pi failed calls without claiming they are hook denials or exposing content", () => {
  const report = read([
    session,
    call("failed", "bash", { command: "fd secret-input-canary" }),
    result("failed", true),
    call("ok", "read", { path: "secret-input-canary" }),
    result("ok", false),
  ]);
  expect(report.denials).toEqual([]);
  expect(report.failures).toEqual([]);
  expect(report.failedToolCalls).toEqual([
    {
      agent: "pi",
      timestamp,
      project: "/project",
      session: "pi-session",
      tool: "bash",
      callId: "failed",
      executable: "fd",
      source: "/session.jsonl",
      line: 3,
    },
  ]);
  const text = JSON.stringify(report);
  expect(text).not.toContain("canary");
});

test.each([
  { isError: true, outcome: "tool-error" },
  { isError: false, outcome: "tool-result" },
  { isError: undefined, outcome: "missing-result" },
])("audits Pi SOPS outcomes: $outcome", ({ isError, outcome }) => {
  const records = [
    session,
    call("sops", "bash", { command: "sops --version" }),
  ];
  const report = read(
    isError === undefined ? records : [...records, result("sops", isError)],
  );
  expect(report.sopsCalls[0]?.outcome).toBe(outcome);
  expect(report.sopsCalls[0]?.agent).toBe("pi");
});

test("reports Pi missing metadata and missing calls", () => {
  const missingSession = read([
    call("id", "bash", { command: "sops --version" }),
    result("id", true),
  ]);
  const codes = missingSession.failures.map((failure) => failure.code);
  expect(codes).toContain("missing-session");
  const missingCall = read([session, result("missing", true)]);
  expect(missingCall.failures).toEqual([
    { code: "missing-call", source: "/session.jsonl", line: 2 },
  ]);
});

test("keeps later Pi failures after malformed JSON", () => {
  const records = [
    session,
    call("id", "read", { path: "/missing" }),
    result("id", true),
  ];
  const report = readTranscript({
    agent: "pi",
    source: "/session.jsonl",
    content: [
      "{broken",
      ...records.map((record) => JSON.stringify(record)),
    ].join("\n"),
  });
  expect(report.failedToolCalls).toHaveLength(1);
  expect(report.failures[0]?.code).toBe("invalid-json");
});
