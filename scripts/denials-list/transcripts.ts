import { z } from "zod";
import { codexOutputError } from "./codex-output.ts";
import { hookError, knownHookError } from "./hook-errors.ts";
import { executable, invocation } from "./executable.ts";
import { readPiTranscript } from "./pi.ts";
import { parseJson } from "./json.ts";
import { transcriptJsonFailure } from "./failures.ts";
import { sopsReference, type SopsCall, type SopsOutcome } from "./sops.ts";

const Envelope = z
  .object({
    agent: z.enum(["claude", "codex", "pi"]),
    source: z.string(),
    content: z.string(),
  })
  .readonly();
const Record = z.object({ type: z.string() }).loose().readonly();
const Unknown = z.unknown();
const Text = z.string();
const UnknownArray = z.array(Unknown).readonly();
const Content = z.union([z.string(), UnknownArray]);
const Message = z.object({ content: Content }).readonly();
const UnknownRecord = z.record(Text, Unknown).readonly();
const ClaudeMessage = z
  .object({
    timestamp: z.iso.datetime({ offset: true }),
    cwd: z.string(),
    sessionId: z.string(),
    message: Message,
  })
  .readonly();
const ClaudeCall = z
  .object({
    type: z.literal("tool_use"),
    id: z.string(),
    name: z.string(),
    input: UnknownRecord,
  })
  .readonly();
const ClaudeResult = z
  .object({
    type: z.literal("tool_result"),
    tool_use_id: z.string(),
    is_error: z.literal(true),
    content: z.string(),
  })
  .readonly();
const CodexEvent = z
  .object({
    timestamp: z.iso.datetime({ offset: true }),
    payload: z.object({ type: z.string() }).loose().readonly(),
  })
  .readonly();
const CodexCall = z
  .object({
    type: z.enum(["function_call", "custom_tool_call"]),
    call_id: z.string(),
    name: z.string(),
    arguments: z.string().optional(),
    input: z.string().optional(),
  })
  .readonly();
const CodexResult = z
  .object({
    type: z.enum(["function_call_output", "custom_tool_call_output"]),
    call_id: z.string(),
    output: Unknown,
  })
  .readonly();
const claudeError = (input: unknown): string | null => {
  const result = ClaudeResult.safeParse(input);
  if (!result.success) return null;
  const reason = result.data.content.replace(
    /^PreToolUse:[^\s]+ hook error:\s*/u,
    "",
  );
  return knownHookError(reason) ? reason : null;
};
const codexError = (input: unknown): string | null => {
  const result = CodexResult.safeParse(input);
  if (!result.success) return null;
  return codexOutputError(result.data.output);
};
const codexInput = (call: z.infer<typeof CodexCall>): unknown => {
  const text = call.type === "custom_tool_call" ? call.input : call.arguments;
  if (text === undefined) return null;
  if (call.name === "exec") return { command: text };
  const parsed = parseJson(text);
  return parsed.success ? parsed.data : null;
};
const CodexSession = z.object({ id: z.string(), cwd: z.string() }).readonly();
const Failure = z
  .object({
    code: z.enum([
      "invalid-json",
      "invalid-record",
      "missing-call",
      "missing-session",
    ]),
    source: z.string(),
    line: z.number().int().positive(),
  })
  .readonly();
const Denial = z
  .object({
    agent: z.enum(["claude", "codex", "pi"]),
    timestamp: z.string(),
    project: z.string(),
    session: z.string(),
    tool: z.string(),
    callId: z.string(),
    executable: z.string().nullable(),
    command: z.string().nullable(),
    reason: z.string(),
    source: z.string(),
    line: z.number(),
  })
  .readonly();
export type TranscriptFailure = z.infer<typeof Failure>;
export type TranscriptErrorCode = TranscriptFailure["code"];
export type TranscriptDenial = z.infer<typeof Denial>;
const DenialMetadata = Denial.unwrap().omit({ command: true }).readonly();
export const denialMetadata = (denial: TranscriptDenial) =>
  DenialMetadata.parse(denial);
const ParsedRecord = z.object({ line: z.number(), value: Record }).readonly();
type ParsedRecord = z.infer<typeof ParsedRecord>;
type Envelope = z.infer<typeof Envelope>;
type Result<T> =
  | Readonly<{ success: true; data: T }>
  | Readonly<{ success: false; error: TranscriptFailure }>;

const parseRecord = (
  text: string,
  source: string,
  line: number,
): Result<ParsedRecord> => {
  const json = parseJson(text);
  if (!json.success)
    return {
      success: false,
      error: transcriptJsonFailure(json.error, source, line),
    };
  const parsed = Record.safeParse(json.data);
  if (!parsed.success)
    return { success: false, error: { code: "invalid-record", source, line } };
  return { success: true, data: { line, value: parsed.data } };
};

const claudeDenials = (
  envelope: Envelope,
  records: readonly ParsedRecord[],
) => {
  const calls = new Map(
    records.flatMap(({ value }) => {
      if (value.type !== "assistant") return [];
      const message = ClaudeMessage.safeParse(value);
      if (!message.success || typeof message.data.message.content === "string")
        return [];
      return message.data.message.content.flatMap((content) => {
        const call = ClaudeCall.safeParse(content);
        if (!call.success) return [];
        const entry: readonly [string, z.infer<typeof ClaudeCall>] = [
          call.data.id,
          call.data,
        ];
        return [entry];
      });
    }),
  );
  return records.flatMap(
    ({ line, value }): readonly Result<TranscriptDenial>[] => {
      if (value.type !== "user") return [];
      const message = ClaudeMessage.safeParse(value);
      if (!message.success || typeof message.data.message.content === "string")
        return [];
      return message.data.message.content.flatMap(
        (content): readonly Result<TranscriptDenial>[] => {
          const result = ClaudeResult.safeParse(content);
          if (!result.success) return [];
          const reason = claudeError(content);
          if (reason === null) return [];
          const call = calls.get(result.data.tool_use_id);
          if (call === undefined)
            return [
              {
                success: false,
                error: { code: "missing-call", source: envelope.source, line },
              },
            ];
          if (hookError(call.name, reason) === null) return [];
          return [
            {
              success: true,
              data: {
                agent: "claude",
                timestamp: message.data.timestamp,
                project: message.data.cwd,
                session: message.data.sessionId,
                tool: call.name,
                callId: call.id,
                executable: executable(call.input),
                command: invocation(call.name, call.input),
                reason,
                source: envelope.source,
                line,
              },
            },
          ];
        },
      );
    },
  );
};

const codexDenials = (envelope: Envelope, records: readonly ParsedRecord[]) => {
  const sessions = records.flatMap(({ value }) => {
    if (value.type !== "session_meta") return [];
    const session = CodexSession.safeParse(value["payload"]);
    return session.success ? [session.data] : [];
  });
  const calls = new Map(
    records.flatMap(({ value }) => {
      if (value.type !== "response_item") return [];
      const event = CodexEvent.safeParse(value);
      if (!event.success) return [];
      const call = CodexCall.safeParse(event.data.payload);
      if (!call.success) return [];
      const entry: readonly [string, z.infer<typeof CodexCall>] = [
        call.data.call_id,
        call.data,
      ];
      return [entry];
    }),
  );
  return records.flatMap(
    ({ line, value }): readonly Result<TranscriptDenial>[] => {
      if (value.type !== "response_item") return [];
      const event = CodexEvent.safeParse(value);
      if (!event.success) return [];
      const result = CodexResult.safeParse(event.data.payload);
      if (!result.success) return [];
      const reason = codexError(event.data.payload);
      if (reason === null) return [];
      const session = sessions[0];
      if (session === undefined)
        return [
          {
            success: false,
            error: { code: "missing-session", source: envelope.source, line },
          },
        ];
      const call = calls.get(result.data.call_id);
      if (call === undefined)
        return [
          {
            success: false,
            error: { code: "missing-call", source: envelope.source, line },
          },
        ];
      const input = codexInput(call);
      if (hookError(call.name, reason) === null) return [];
      return [
        {
          success: true,
          data: {
            agent: "codex",
            timestamp: event.data.timestamp,
            project: session.cwd,
            session: session.id,
            tool: call.name,
            callId: call.call_id,
            executable: executable(input),
            command: invocation(call.name, input),
            reason,
            source: envelope.source,
            line,
          },
        },
      ];
    },
  );
};

const ClaudeToolResult = z
  .object({
    type: z.literal("tool_result"),
    tool_use_id: z.string(),
    is_error: z.boolean().optional(),
    content: Unknown,
  })
  .readonly();
const CodexToolResult = z
  .object({
    type: z.enum(["function_call_output", "custom_tool_call_output"]),
    call_id: z.string(),
    output: Unknown,
  })
  .readonly();

const claudeSopsCalls = (
  envelope: Envelope,
  records: readonly ParsedRecord[],
): readonly SopsCall[] => {
  const results = new Map(
    records.flatMap(({ value }) => {
      if (value.type !== "user") return [];
      const message = ClaudeMessage.safeParse(value);
      if (!message.success || typeof message.data.message.content === "string")
        return [];
      return message.data.message.content.flatMap((content) => {
        const result = ClaudeToolResult.safeParse(content);
        if (!result.success) return [];
        const outcome = ((): SopsOutcome => {
          if (claudeError(content) !== null) return "hook-denied";
          return result.data.is_error === true ? "tool-error" : "tool-result";
        })();
        const entry: readonly [string, SopsOutcome] = [
          result.data.tool_use_id,
          outcome,
        ];
        return [entry];
      });
    }),
  );
  return records.flatMap(({ line, value }) => {
    if (value.type !== "assistant") return [];
    const message = ClaudeMessage.safeParse(value);
    if (!message.success || typeof message.data.message.content === "string")
      return [];
    return message.data.message.content.flatMap(
      (content): readonly SopsCall[] => {
        const call = ClaudeCall.safeParse(content);
        if (!call.success || call.data.name !== "Bash") return [];
        const reference = sopsReference(call.data.input);
        if (reference === null) return [];
        const result = results.get(call.data.id);
        return [
          {
            agent: "claude",
            timestamp: message.data.timestamp,
            project: message.data.cwd,
            session: message.data.sessionId,
            tool: call.data.name,
            callId: call.data.id,
            source: envelope.source,
            line,
            ...reference,
            outcome: result === undefined ? "missing-result" : result,
          },
        ];
      },
    );
  });
};

const codexSopsCalls = (
  envelope: Envelope,
  records: readonly ParsedRecord[],
) => {
  const session = records.flatMap(({ value }) => {
    if (value.type !== "session_meta") return [];
    const parsed = CodexSession.safeParse(value["payload"]);
    return parsed.success ? [parsed.data] : [];
  })[0];
  const results = new Map(
    records.flatMap(({ value }) => {
      if (value.type !== "response_item") return [];
      const result = CodexToolResult.safeParse(value["payload"]);
      if (!result.success) return [];
      const outcome: SopsOutcome =
        codexError(value["payload"]) !== null ? "hook-denied" : "tool-result";
      const entry: readonly [string, SopsOutcome] = [
        result.data.call_id,
        outcome,
      ];
      return [entry];
    }),
  );
  const matches = records.flatMap(
    ({ line, value }): readonly Result<SopsCall>[] => {
      if (value.type !== "response_item") return [];
      const event = CodexEvent.safeParse(value);
      if (!event.success) return [];
      const call = CodexCall.safeParse(event.data.payload);
      if (
        !call.success ||
        !["exec", "exec_command", "shell_command", "Bash"].includes(
          call.data.name,
        )
      )
        return [];
      const input = codexInput(call.data);
      const reference = sopsReference(input);
      if (reference === null) return [];
      if (session === undefined)
        return [
          {
            success: false,
            error: { code: "missing-session", source: envelope.source, line },
          },
        ];
      const result = results.get(call.data.call_id);
      return [
        {
          success: true,
          data: {
            agent: "codex",
            timestamp: event.data.timestamp,
            project: session.cwd,
            session: session.id,
            tool: call.data.name,
            callId: call.data.call_id,
            source: envelope.source,
            line,
            ...reference,
            outcome: result === undefined ? "missing-result" : result,
          },
        },
      ];
    },
  );
  return {
    calls: matches.flatMap((match) => (match.success ? [match.data] : [])),
    failures: matches.flatMap((match) => (match.success ? [] : [match.error])),
  };
};

export const readTranscript = (input: unknown) => {
  const envelope = Envelope.parse(input);
  const parsed = envelope.content
    .split(/\r?\n/u)
    .flatMap((text, index) =>
      text.trim() === "" ? [] : [parseRecord(text, envelope.source, index + 1)],
    );
  const records = parsed.flatMap((result) =>
    result.success ? [result.data] : [],
  );
  if (envelope.agent === "pi") {
    const report = readPiTranscript(envelope.source, records);
    return {
      ...report,
      failures: parsed
        .flatMap((result) => (result.success ? [] : [result.error]))
        .concat(report.failures),
    };
  }
  const results =
    envelope.agent === "claude"
      ? claudeDenials(envelope, records)
      : codexDenials(envelope, records);
  const sops =
    envelope.agent === "claude"
      ? { calls: claudeSopsCalls(envelope, records), failures: [] }
      : codexSopsCalls(envelope, records);
  return {
    denials: results.flatMap((result) => (result.success ? [result.data] : [])),
    sopsCalls: sops.calls,
    failedToolCalls: [],
    failures: [...parsed, ...results]
      .flatMap((result) => (result.success ? [] : [result.error]))
      .concat(sops.failures),
  };
};
