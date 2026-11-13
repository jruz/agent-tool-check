import { z } from "zod";
import { redactCommand } from "./redact.ts";

const Input = z
  .object({
    command: z.string().optional(),
    cmd: z.string().optional(),
    path: z.string().optional(),
    file_path: z.string().optional(),
  })
  .readonly();

export const invocation = (tool: string, input: unknown): string | null => {
  const parsed = Input.safeParse(input);
  if (!parsed.success) return null;
  const command =
    parsed.data.command === undefined ? parsed.data.cmd : parsed.data.command;
  if (command !== undefined) return redactCommand(command);
  const path =
    parsed.data.path === undefined ? parsed.data.file_path : parsed.data.path;
  if (path === undefined) return null;
  return `${tool} ${JSON.stringify(path)}`;
};

export const executable = (input: unknown): string | null => {
  const parsed = Input.safeParse(input);
  if (!parsed.success) return null;
  const command =
    parsed.data.command === undefined ? parsed.data.cmd : parsed.data.command;
  if (command === undefined) return null;
  const match = /^\s*([a-zA-Z][a-zA-Z0-9_./-]*)(?=\s|$)/u.exec(command);
  if (match?.[1] === undefined) return null;
  return match[1];
};
