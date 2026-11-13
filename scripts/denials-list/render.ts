import { basename } from "node:path";
import type { TranscriptDenial } from "./transcripts.ts";

const errorKey = (error: string) =>
  error
    .toLowerCase()
    .replace(/[^a-z0-9]+/gu, "-")
    .replace(/^-|-$/gu, "");

export const groupDenials = (
  denials: readonly TranscriptDenial[],
  limit: number,
) => {
  const selected = denials.slice(0, limit);
  const grouped = Object.groupBy(selected, (denial) => errorKey(denial.reason));
  const entries = Object.entries(grouped).map(([key, denials]) => {
    if (denials === undefined) throw new Error("missing denial group");
    const details = denials.map((denial) => ({
      command: denial.command,
      project: basename(denial.project),
      agent: denial.agent,
      timestamp: denial.timestamp,
    }));
    const group: readonly [string, typeof details] = [key, details];
    return group;
  });
  return Object.fromEntries(entries);
};
