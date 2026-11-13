import { expect, test } from "bun:test";
import { codexOutputError } from "./codex-output.ts";

const reason = "Never print environment variables";
const blocked = `Command blocked by PreToolUse hook: ${reason}. Command: printenv`;

test.each(["text", "input_text"])(
  "reads Script error blocks with type %s",
  (type) => {
    const blocks = [
      { type, text: "Script output:\n" },
      { type, text: `Script error:\n${blocked}` },
      { type, text: "Script completed" },
    ];
    const error = codexOutputError(blocks);
    expect(error).toBe(reason);
  },
);

test("preserves support for direct function-result errors", () => {
  const error = codexOutputError(blocked);
  expect(error).toBe(reason);
});

const unrelatedOutputs = [
  [{ type: "input_text", text: `Script output:\n${blocked}` }],
  [{ type: "input_text", text: blocked }],
  [{ type: "input_text", text: "Script error:\nordinary command failure" }],
  [
    {
      type: "input_text",
      text: `Script error:\n${reason}\nsecret-output-canary`,
    },
  ],
  [{ type: "image", text: `Script error:\n${blocked}` }],
].map((output) => ({ output }));

test.each(unrelatedOutputs)(
  "ignores success output and unknown error shapes: %j",
  ({ output }) => {
    const error = codexOutputError(output);
    expect(error).toBeNull();
  },
);
