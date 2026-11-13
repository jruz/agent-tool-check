import { z } from "zod";
import { knownHookError } from "./hook-errors.ts";

const TextBlock = z
  .object({ type: z.enum(["text", "input_text"]), text: z.string() })
  .readonly();
const TextBlocks = z.array(TextBlock).readonly();
const Text = z.string();

const denialReason = (text: string): string | null => {
  const prefix = "Command blocked by PreToolUse hook: ";
  if (!text.startsWith(prefix)) return knownHookError(text) ? text : null;
  const reason = text.slice(prefix.length).split(". Command:")[0];
  if (reason === undefined) throw new Error("missing denial reason");
  return knownHookError(reason) ? reason : null;
};

const scriptError = (text: string): string | null => {
  const prefix = "Script error:\n";
  if (!text.startsWith(prefix)) return null;
  const error = text.slice(prefix.length);
  return denialReason(error);
};

export const codexOutputError = (output: unknown): string | null => {
  const text = Text.safeParse(output);
  if (text.success) {
    const script = scriptError(text.data);
    if (script !== null) return script;
    return denialReason(text.data);
  }
  const blocks = TextBlocks.safeParse(output);
  if (!blocks.success) return null;
  const reasons = blocks.data.flatMap((block) => {
    const reason = scriptError(block.text);
    return reason === null ? [] : [reason];
  });
  if (reasons.length === 0) return null;
  const reason = reasons[0];
  if (reason === undefined) throw new Error("missing script denial reason");
  return reason;
};
