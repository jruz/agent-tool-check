import { expect, test } from "bun:test";
import { readTranscript } from "./transcripts.ts";

const timestamp = "2026-09-04T07:01:48.278Z";
const script = 'await exec_command({cmd: "sops --version"})';
const reason = "Never print environment variables";
const records = [
  {
    type: "session_meta",
    timestamp,
    payload: { id: "session", cwd: "/project" },
  },
  {
    type: "response_item",
    timestamp,
    payload: {
      type: "custom_tool_call",
      call_id: "call",
      name: "exec",
      input: script,
    },
  },
  {
    type: "response_item",
    timestamp,
    payload: {
      type: "custom_tool_call_output",
      call_id: "call",
      output: [
        { type: "input_text", text: "Script output:\n" },
        {
          type: "input_text",
          text: `Script error:\nCommand blocked by PreToolUse hook: ${reason}. Command: sops --version`,
        },
      ],
    },
  },
];

test("joins real Codex exec and input_text result shapes without evaluating scripts", () => {
  const content = records.map((record) => JSON.stringify(record)).join("\n");
  const report = readTranscript({
    agent: "codex",
    source: "/codex.jsonl",
    content,
  });
  expect(report.failures).toEqual([]);
  expect(report.denials).toHaveLength(1);
  expect(report.denials[0]?.reason).toBe(reason);
  expect(report.denials[0]?.tool).toBe("exec");
  expect(report.denials[0]?.command).toBe(script);
  expect(report.denials[0]?.line).toBe(3);
  expect(report.sopsCalls[0]?.outcome).toBe("hook-denied");
});

test("does not classify Script output echoes as denials", () => {
  const altered = records
    .map((record) =>
      JSON.stringify(record).replace("Script error:", "Script output:"),
    )
    .join("\n");
  const report = readTranscript({
    agent: "codex",
    source: "/codex.jsonl",
    content: altered,
  });
  expect(report.denials).toEqual([]);
  expect(report.sopsCalls[0]?.outcome).toBe("tool-result");
});
