import type { PiFailedToolCall } from "./pi.ts";
import type { SopsCall } from "./sops.ts";
import type { TranscriptDenial, TranscriptFailure } from "./transcripts.ts";

export type SourceReports = Readonly<{
  reports: readonly Readonly<{
    denials: readonly TranscriptDenial[];
    sopsCalls: readonly SopsCall[];
    failedToolCalls: readonly PiFailedToolCall[];
    failures: readonly TranscriptFailure[];
  }>[];
  failures: readonly SourceFailure[];
}>;

export type SourceErrorCode = "read-directory" | "read-transcript";
export type SourceFailure = Readonly<{
  code: SourceErrorCode;
  source: string;
  message: string;
}>;
export type SourceResult<T> =
  | Readonly<{ success: true; data: T }>
  | Readonly<{ success: false; error: SourceFailure }>;
export type AgentSelection = "all" | "claude" | "codex" | "pi";
