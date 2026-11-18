import { z } from "zod";
import { readFile } from "node:fs/promises";
import { Denial } from "./transcripts.ts";
import { parseJson } from "./json.ts";
import type { SourceReports } from "./sources.types.ts";

const Entry = Denial.unwrap()
  .omit({ source: true, line: true })
  .extend({ timestamp: z.iso.datetime({ offset: true }) })
  .readonly();

export const readDenialLog = async (source: string): Promise<SourceReports> => {
  const content = await readFile(source, "utf8")
    .then((data) => ({ success: true as const, data }))
    .catch((error: unknown) => {
      if (!(error instanceof Error)) throw error;
      return {
        success: false as const,
        error: { code: "read-log" as const, source, message: error.message },
      };
    });
  if (!content.success) return { reports: [], failures: [content.error] };
  const records = content.data.split(/\r?\n/u).flatMap((text, index) => {
    if (text.trim() === "") return [];
    const json = parseJson(text);
    const parsed = Entry.safeParse(json.success ? json.data : null);
    return [{ parsed, line: index + 1 }];
  });
  return {
    reports: [
      {
        denials: records.flatMap(({ parsed, line }) =>
          parsed.success ? [{ ...parsed.data, source, line }] : [],
        ),
        failures: records.flatMap(({ parsed, line }) =>
          parsed.success
            ? []
            : [{ code: "invalid-record" as const, source, line }],
        ),
        sopsCalls: [],
        failedToolCalls: [],
      },
    ],
    failures: [],
  };
};
