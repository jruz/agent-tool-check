import { z } from "zod";

const CommandInput = z
  .object({
    command: z.string().optional(),
    cmd: z.string().optional(),
  })
  .readonly();
const SopsOutcome = z.enum([
  "hook-denied",
  "tool-result",
  "tool-error",
  "missing-result",
]);
const SopsCall = z
  .object({
    agent: z.enum(["claude", "codex", "pi"]),
    timestamp: z.string(),
    project: z.string(),
    session: z.string(),
    tool: z.string(),
    callId: z.string(),
    source: z.string(),
    line: z.number(),
    match: z.enum(["sops-token", "recipe-reference"]),
    decryptReference: z.boolean(),
    outcome: SopsOutcome,
  })
  .readonly();
export type SopsCall = z.infer<typeof SopsCall>;
export type SopsOutcome = z.infer<typeof SopsOutcome>;

export const sopsReference = (input: unknown) => {
  const parsed = CommandInput.safeParse(input);
  if (!parsed.success) return null;
  const command =
    parsed.data.command === undefined ? parsed.data.cmd : parsed.data.command;
  if (command === undefined) return null;
  const token = /(?:^|[\s/;'"`($|&])sops(?=$|[\s;'"`)|&])/u.test(command);
  const recipe = /\bsops-[\w-]+/u.test(command);
  if (!token && !recipe) return null;
  return {
    match: token ? ("sops-token" as const) : ("recipe-reference" as const),
    decryptReference: /\bdecrypt\b|(?:^|\s|['"])-d(?=$|\s|['"])/u.test(command),
  };
};
