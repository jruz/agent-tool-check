import { z } from "zod";
import { hookError } from "./hook-errors.ts";

const TextContent = z
  .object({ type: z.literal("text"), text: z.string() })
  .readonly();
const ErrorContent = z.tuple([TextContent]).readonly();

export const piDenialReason = (
  tool: string,
  content: unknown,
): string | null => {
  if (!["bash", "read", "grep"].includes(tool)) return null;
  const parsed = ErrorContent.safeParse(content);
  if (!parsed.success) return null;
  return hookError(tool, parsed.data[0].text);
};
