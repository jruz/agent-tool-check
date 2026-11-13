import type { JsonErrorCode, JsonFailure } from "./json.ts";
import type { TranscriptErrorCode, TranscriptFailure } from "./transcripts.ts";

const transcriptCodes: Readonly<Record<JsonErrorCode, TranscriptErrorCode>> = {
  "invalid-json": "invalid-json",
};

export const transcriptJsonFailure = (
  error: JsonFailure,
  source: string,
  line: number,
): TranscriptFailure => ({ code: transcriptCodes[error.code], source, line });
