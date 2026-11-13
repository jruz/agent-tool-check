import { expect, test } from "bun:test";
import { readTranscript, denialMetadata } from "./transcripts.ts";

const timestamp = "2026-11-03T12:00:00Z";
const reason = "Never print environment variables";
const records = (isError: boolean, toolName: string) => [
  { type: "session", id: "session", cwd: "/project" },
  {
    type: "message",
    timestamp,
    message: {
      role: "assistant",
      content: [
        {
          type: "toolCall",
          id: "call",
          name: "bash",
          arguments: { command: "sops --version secret-input-canary" },
        },
      ],
    },
  },
  {
    type: "message",
    timestamp,
    message: {
      role: "toolResult",
      toolCallId: "call",
      toolName,
      isError,
      content: [{ type: "text", text: reason }],
    },
  },
];
const read = (entries: readonly unknown[]) =>
  readTranscript({
    agent: "pi",
    source: "/session.jsonl",
    content: entries.map((entry) => JSON.stringify(entry)).join("\n"),
  });

test("joins Pi hook errors to their calls and classifies SOPS outcomes", () => {
  const entries = records(true, "bash");
  const report = read(entries);
  expect(report.denials).toEqual([
    {
      agent: "pi",
      timestamp,
      project: "/project",
      session: "session",
      tool: "bash",
      callId: "call",
      executable: "sops",
      command: "sops --version secret-input-canary",
      reason,
      source: "/session.jsonl",
      line: 3,
    },
  ]);
  expect(report.sopsCalls[0]?.outcome).toBe("hook-denied");
  const metadata = report.denials.map(denialMetadata);
  const text = JSON.stringify(metadata);
  expect(text).not.toContain("secret-input-canary");
});

test("does not treat successful output as a denial", () => {
  const entries = records(false, "bash");
  const report = read(entries);
  expect(report.denials).toEqual([]);
  expect(report.sopsCalls[0]?.outcome).toBe("tool-result");
});

test("does not classify mismatched tool results as hook denials", () => {
  const entries = records(true, "edit");
  const report = read(entries);
  expect(report.denials).toEqual([]);
  expect(report.sopsCalls[0]?.outcome).toBe("tool-error");
});

test("reports missing calls or session metadata instead of inventing denial context", () => {
  const entries = records(true, "bash");
  const withoutCall = entries.filter(
    (entry) => entry.message?.role !== "assistant",
  );
  const missingCall = read(withoutCall);
  expect(missingCall.denials).toEqual([]);
  expect(missingCall.failures[0]?.code).toBe("missing-call");
  const withoutSession = entries.filter((entry) => entry.type !== "session");
  const missingSession = read(withoutSession);
  expect(missingSession.denials).toEqual([]);
  expect(missingSession.failures[0]?.code).toBe("missing-session");
});
