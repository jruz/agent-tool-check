import { describe, expect, test } from "bun:test";
import { readTranscript, denialMetadata } from "./transcripts.ts";
import type {
  TranscriptDenial,
  TranscriptErrorCode,
  TranscriptFailure,
} from "./transcripts.ts";

const timestamp = "2026-10-30T12:00:00.000Z";
const message = (type: string, content: readonly unknown[]) => ({
  type,
  timestamp,
  cwd: "/project",
  sessionId: "claude-session",
  message: { content },
});
const call = message("assistant", [
  {
    type: "tool_use",
    id: "call-1",
    name: "Bash",
    input: { command: "fd --exclude .env ." },
  },
]);
const result = (content: string) =>
  message("user", [
    {
      type: "tool_result",
      tool_use_id: "call-1",
      is_error: true,
      content,
    },
  ]);
const envelope = (agent: "claude" | "codex", records: readonly unknown[]) => ({
  agent,
  source: `/transcripts/${agent}.jsonl`,
  content: records.map((record) => JSON.stringify(record)).join("\n"),
});
const read = (agent: "claude" | "codex", records: readonly unknown[]) => {
  const input = envelope(agent, records);
  return readTranscript(input);
};
const readClaude = (content: string) => {
  const output = result(content);
  return read("claude", [call, output]);
};
const codexRecord = (type: string, payload: unknown) => ({
  type,
  timestamp,
  payload,
});
const session = codexRecord("session_meta", {
  id: "codex-session",
  cwd: "/project",
});

describe("Claude transcripts", () => {
  test("joins explicit hook denials to calls and keeps commands out of metadata", () => {
    const report = readClaude(
      "PreToolUse:Bash hook error: Never access .env files directly",
    );
    expect(report.failures).toEqual([]);
    const denials: readonly TranscriptDenial[] = [
      {
        agent: "claude",
        timestamp,
        project: "/project",
        session: "claude-session",
        tool: "Bash",
        callId: "call-1",
        executable: "fd",
        command: "fd --exclude .env .",
        reason: "Never access .env files directly",
        source: "/transcripts/claude.jsonl",
        line: 2,
      },
    ];
    expect(report.denials).toEqual([...denials]);
    const metadata = report.denials.map(denialMetadata);
    const serialized = JSON.stringify(metadata);
    expect(serialized).not.toContain("--exclude");
  });

  test("recognizes our exact errors without requiring a provider hook prefix", () => {
    const report = readClaude("Never print environment variables");
    expect(report.denials[0]?.reason).toBe("Never print environment variables");
  });

  test.each([
    "PreToolUse:Bash hook error: unterminated here-document sequence tag(s): EOF found at 1:18 detected near line 19 col 1",
    "PreToolUse:Bash hook error: unrelated hook failure",
    "tool output: Never print environment variables",
  ])("does not count unrecognized hook or tool errors: %s", (text) => {
    const report = readClaude(text);
    expect(report.denials).toEqual([]);
  });

  test("does not mistake command failures mentioning hooks for denials", () => {
    const report = readClaude("tests: PreToolUse hook failed");
    expect(report.denials).toEqual([]);
  });

  test("reports malformed lines without exposing their contents or losing later records", () => {
    const input = envelope("claude", [call]);
    const output = result(
      "PreToolUse:Bash hook error: Never print environment variables",
    );
    const serialized = JSON.stringify(output);
    const report = readTranscript({
      ...input,
      content: `${input.content}\n{secret\n${serialized}\n`,
    });
    const code: TranscriptErrorCode = "invalid-json";
    const failures: readonly TranscriptFailure[] = [
      { code, source: input.source, line: 2 },
    ];
    expect(report.failures).toEqual([...failures]);
    expect(report.denials[0]?.line).toBe(3);
    const reportText = JSON.stringify(report);
    expect(reportText).not.toContain("secret");
  });

  test("reports missing calls instead of silently dropping denials", () => {
    const output = result(
      "PreToolUse:Bash hook error: Never print environment variables",
    );
    const report = read("claude", [output]);
    expect(report.failures[0]?.code).toBe("missing-call");
  });

  test("omits assignments and command arguments from previews", () => {
    const secretCall = message("assistant", [
      {
        type: "tool_use",
        id: "call-1",
        name: "Bash",
        input: { command: "TOKEN=private-value curl https://example.com" },
      },
    ]);
    const output = result(
      "PreToolUse:Bash hook error: Never print environment variables",
    );
    const report = read("claude", [secretCall, output]);
    expect(report.denials[0]?.executable).toBeNull();
    const serialized = JSON.stringify(report);
    expect(serialized).not.toContain("private-value");
  });
});

describe("Codex transcripts", () => {
  test.each(["function", "custom_tool"])(
    "joins %s call outputs and removes embedded commands",
    (kind) => {
      const argumentsText = JSON.stringify({ cmd: "fd --exclude .env ." });
      const invocation = codexRecord("response_item", {
        type: `${kind}_call`,
        call_id: "call-2",
        name: "exec_command",
        ...(kind === "function"
          ? { arguments: argumentsText }
          : { input: argumentsText }),
      });
      const output = codexRecord("response_item", {
        type: `${kind}_call_output`,
        call_id: "call-2",
        output:
          "Command blocked by PreToolUse hook: Never access .env files directly. Command: fd --exclude .env .",
      });
      const report = read("codex", [session, invocation, output]);
      expect(report.failures).toEqual([]);
      expect(report.denials[0]).toEqual({
        agent: "codex",
        timestamp,
        project: "/project",
        session: "codex-session",
        tool: "exec_command",
        callId: "call-2",
        executable: "fd",
        command: "fd --exclude .env .",
        reason: "Never access .env files directly",
        source: "/transcripts/codex.jsonl",
        line: 3,
      });
    },
  );

  test.each([
    { output: "Never print environment variables", count: 1 },
    {
      output: "Command blocked by PreToolUse hook: unrelated hook failure",
      count: 0,
    },
    {
      output:
        "Command blocked by PreToolUse hook: unterminated here-document sequence tag(s): EOF found at 1:18 detected near line 19 col 1",
      count: 0,
    },
    { output: "test output: Never print environment variables", count: 0 },
  ])(
    "matches only our known errors in Codex results: $output",
    ({ output, count }) => {
      const argumentsText = JSON.stringify({ cmd: "printenv" });
      const invocation = codexRecord("response_item", {
        type: "function_call",
        call_id: "call-2",
        name: "exec_command",
        arguments: argumentsText,
      });
      const result = codexRecord("response_item", {
        type: "function_call_output",
        call_id: "call-2",
        output,
      });
      const report = read("codex", [session, invocation, result]);
      expect(report.denials).toHaveLength(count);
    },
  );

  test("does not match hook messages embedded in normal command output", () => {
    const output = codexRecord("response_item", {
      type: "function_call_output",
      call_id: "call-2",
      output: "test output: Command blocked by PreToolUse hook: example",
    });
    const report = read("codex", [session, output]);
    expect(report.denials).toEqual([]);
  });

  test("reports missing session metadata", () => {
    const output = codexRecord("response_item", {
      type: "function_call_output",
      call_id: "call-2",
      output:
        "Command blocked by PreToolUse hook: Never print environment variables",
    });
    const report = read("codex", [output]);
    expect(report.failures[0]?.code).toBe("missing-session");
  });

  test("reports structurally invalid records", () => {
    const report = read("codex", [null]);
    expect(report.failures[0]?.code).toBe("invalid-record");
  });
});
