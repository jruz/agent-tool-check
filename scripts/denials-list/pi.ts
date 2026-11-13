import { z } from "zod";
import { executable, invocation } from "./executable.ts";
import { piDenialReason } from "./pi-denials.ts";
import { sopsReference, type SopsCall, type SopsOutcome } from "./sops.ts";
import type { TranscriptDenial, TranscriptFailure } from "./transcripts.ts";

const RecordValue = z.object({ type: z.string() }).loose().readonly();
const Record = z.object({ line: z.number(), value: RecordValue }).readonly();
const Records = z.array(Record).readonly();
const Unknown = z.unknown();
const Text = z.string();
const Session = z.object({ id: z.string(), cwd: z.string() }).readonly();
const Message = z
  .object({
    timestamp: z.iso.datetime({ offset: true }),
    message: z.object({ role: z.string() }).loose().readonly(),
  })
  .readonly();
const Assistant = z
  .object({
    role: z.literal("assistant"),
    content: z.array(Unknown).readonly(),
  })
  .readonly();
const Call = z
  .object({
    type: z.literal("toolCall"),
    id: z.string(),
    name: z.string(),
    arguments: z.record(Text, Unknown).readonly(),
  })
  .readonly();
const ToolResult = z
  .object({
    role: z.literal("toolResult"),
    toolCallId: z.string(),
    toolName: z.string(),
    isError: z.boolean(),
    content: z.array(Unknown).readonly(),
  })
  .readonly();
const FailedToolCall = z
  .object({
    agent: z.literal("pi"),
    timestamp: z.string(),
    project: z.string(),
    session: z.string(),
    tool: z.string(),
    callId: z.string(),
    executable: z.string().nullable(),
    source: z.string(),
    line: z.number(),
  })
  .readonly();
export type PiFailedToolCall = z.infer<typeof FailedToolCall>;

export const readPiTranscript = (source: string, input: unknown) => {
  const records = Records.parse(input);
  const session = records.flatMap(({ value }) => {
    if (value.type !== "session") return [];
    const parsed = Session.safeParse(value);
    return parsed.success ? [parsed.data] : [];
  })[0];
  const messages = records.flatMap(({ line, value }) => {
    if (value.type !== "message") return [];
    const parsed = Message.safeParse(value);
    return parsed.success ? [{ line, ...parsed.data }] : [];
  });
  const calls = messages.flatMap(({ line, timestamp, message }) => {
    const parsed = Assistant.safeParse(message);
    if (!parsed.success) return [];
    return parsed.data.content.flatMap((content) => {
      const call = Call.safeParse(content);
      return call.success ? [{ line, timestamp, ...call.data }] : [];
    });
  });
  const results = messages.flatMap(({ line, timestamp, message }) => {
    const result = ToolResult.safeParse(message);
    if (!result.success) return [];
    const reason = result.data.isError
      ? piDenialReason(result.data.toolName, result.data.content)
      : null;
    return [{ line, timestamp, ...result.data, reason }];
  });
  const failures: readonly TranscriptFailure[] = [
    ...results
      .filter(
        (result) =>
          result.isError &&
          !calls.some((call) => call.id === result.toolCallId),
      )
      .map((result) => ({
        code: "missing-call" as const,
        source,
        line: result.line,
      })),
    ...(session === undefined
      ? messages
          .filter(
            ({ message }) =>
              ToolResult.safeParse(message).success ||
              Assistant.safeParse(message).success,
          )
          .map(({ line }) => ({
            code: "missing-session" as const,
            source,
            line,
          }))
      : []),
  ];
  const failedToolCalls: readonly PiFailedToolCall[] = results.flatMap(
    (result) => {
      if (!result.isError || session === undefined) return [];
      const call = calls.find(
        (candidate) => candidate.id === result.toolCallId,
      );
      if (call === undefined) return [];
      return [
        {
          agent: "pi",
          timestamp: result.timestamp,
          project: session.cwd,
          session: session.id,
          tool: call.name,
          callId: call.id,
          executable: executable(call.arguments),
          source,
          line: result.line,
        },
      ];
    },
  );
  const sopsCalls: readonly SopsCall[] = calls.flatMap((call) => {
    if (call.name !== "bash" || session === undefined) return [];
    const reference = sopsReference(call.arguments);
    if (reference === null) return [];
    const result = results.find(
      (candidate) => candidate.toolCallId === call.id,
    );
    const outcome = ((): SopsOutcome => {
      if (result === undefined) return "missing-result";
      if (result.reason !== null && result.toolName === call.name)
        return "hook-denied";
      return result.isError ? "tool-error" : "tool-result";
    })();
    return [
      {
        agent: "pi",
        timestamp: call.timestamp,
        project: session.cwd,
        session: session.id,
        tool: call.name,
        callId: call.id,
        source,
        line: call.line,
        ...reference,
        outcome,
      },
    ];
  });
  const denials: readonly TranscriptDenial[] = results.flatMap((result) => {
    if (result.reason === null || session === undefined) return [];
    const call = calls.find((candidate) => candidate.id === result.toolCallId);
    if (call === undefined || call.name !== result.toolName) return [];
    return [
      {
        agent: "pi",
        timestamp: result.timestamp,
        project: session.cwd,
        session: session.id,
        tool: call.name,
        callId: call.id,
        executable: executable(call.arguments),
        command: invocation(call.name, call.arguments),
        reason: result.reason,
        source,
        line: result.line,
      },
    ];
  });
  return { denials, failedToolCalls, sopsCalls, failures };
};
