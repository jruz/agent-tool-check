import { afterAll, expect, test } from "bun:test";
import { mkdir, mkdtemp, rm, symlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";
import { parseJson } from "./json.ts";

const scriptUrl = new URL("../denials-list.ts", import.meta.url);
const script = fileURLToPath(scriptUrl);
const Summary = z
  .object({ agent: z.string(), reason: z.string(), count: z.number() })
  .readonly();
const Denial = z
  .object({ timestamp: z.string(), agent: z.string() })
  .readonly();
const Failure = z.object({ code: z.string(), line: z.number() }).readonly();
const SourceFailure = z
  .object({ code: z.string(), source: z.string() })
  .readonly();
const SopsSummary = z
  .object({
    agent: z.string(),
    match: z.string(),
    outcome: z.string(),
    decryptReference: z.boolean(),
    count: z.number(),
  })
  .readonly();
const SopsCall = z
  .object({ timestamp: z.string(), outcome: z.string() })
  .readonly();
const FailedToolCall = z
  .object({ agent: z.string(), callId: z.string(), timestamp: z.string() })
  .readonly();
const Detail = z
  .strictObject({
    command: z.string().nullable(),
    project: z.string(),
    agent: z.string(),
    timestamp: z.string(),
  })
  .readonly();
const Details = z.array(Detail).readonly();
const Text = z.string();
const Grouped = z.record(Text, Details).readonly();
const Report = z
  .object({
    since: z.string(),
    agent: z.string(),
    total: z.number(),
    summary: z.array(Summary).readonly(),
    denials: z.array(Denial).readonly(),
    failedToolCalls: z.array(FailedToolCall).readonly(),
    failures: z.array(Failure).readonly(),
    sourceFailures: z.array(SourceFailure).readonly(),
    sops: z
      .object({
        total: z.number(),
        summary: z.array(SopsSummary).readonly(),
        calls: z.array(SopsCall).readonly(),
      })
      .readonly(),
  })
  .readonly();
const parseReport = (text: string) => {
  const json = parseJson(text);
  if (!json.success) throw new Error(json.error.code);
  return Report.parse(json.data);
};
const now = Date.now();
const daysAgo = (days: number) =>
  new Date(now - days * 86_400_000).toISOString();
const old = daysAgo(14);
const recent = daysAgo(4);
const newest = daysAgo(3);
const transcript = (timestamp: string, id: string) => [
  {
    type: "assistant",
    timestamp,
    cwd: "/project",
    sessionId: "session",
    message: {
      content: [
        {
          type: "tool_use",
          id,
          name: "Bash",
          input: { command: "fd --exclude .env ." },
        },
      ],
    },
  },
  {
    type: "user",
    timestamp,
    cwd: "/project",
    sessionId: "session",
    message: {
      content: [
        {
          type: "tool_result",
          tool_use_id: id,
          is_error: true,
          content:
            "PreToolUse:Bash hook error: Never print environment variables",
        },
      ],
    },
  },
];
const content = [
  ...transcript(old, "old"),
  ...transcript(recent, "recent"),
  ...transcript(newest, "newest"),
]
  .map((record) => JSON.stringify(record))
  .join("\n");
const temporary = tmpdir();
const prefix = join(temporary, "denials-cli-");
const home = await mkdtemp(prefix);
const claudeDirectory = join(
  home,
  ".claude",
  "projects",
  "project",
  "subagents",
);
const codexDirectory = join(home, ".codex", "sessions");
const piDirectory = join(home, ".pi", "agent", "sessions", "project");
await mkdir(claudeDirectory, { recursive: true });
await mkdir(codexDirectory, { recursive: true });
await mkdir(piDirectory, { recursive: true });
const piSource = join(piDirectory, "session.jsonl");
const piContent = [
  { type: "session", id: "pi-session", cwd: "/pi-project" },
  {
    type: "message",
    timestamp: newest,
    message: {
      role: "assistant",
      content: [
        {
          type: "toolCall",
          id: "pi-call",
          name: "read",
          arguments: { path: "input-canary" },
        },
      ],
    },
  },
  {
    type: "message",
    timestamp: newest,
    message: {
      role: "toolResult",
      toolCallId: "pi-call",
      toolName: "read",
      isError: true,
      content: [{ type: "text", text: "output-canary" }],
    },
  },
]
  .map((record) => JSON.stringify(record))
  .join("\n");
await Bun.write(piSource, piContent);
const source = join(claudeDirectory, "session.jsonl");
await Bun.write(source, content);
const sopsSource = join(claudeDirectory, "sops.jsonl");
const sopsContent = [old, recent, newest]
  .flatMap((timestamp) => [
    {
      type: "assistant",
      timestamp,
      cwd: "/project",
      sessionId: "session",
      message: {
        content: [
          {
            type: "tool_use",
            id: timestamp,
            name: "Bash",
            input: { command: "sops --version" },
          },
        ],
      },
    },
    {
      type: "user",
      timestamp,
      cwd: "/project",
      sessionId: "session",
      message: {
        content: [
          {
            type: "tool_result",
            tool_use_id: timestamp,
            content: "output-canary",
            is_error: false,
          },
        ],
      },
    },
  ])
  .map((record) => JSON.stringify(record))
  .join("\n");
await Bun.write(sopsSource, sopsContent);
const outside = join(home, "outside.jsonl");
const linked = join(claudeDirectory, "linked.jsonl");
const ignored = join(claudeDirectory, "notes.txt");
await Bun.write(outside, content);
await Bun.write(ignored, content);
await symlink(outside, linked);
afterAll(async () => {
  await rm(home, { recursive: true });
});
const run = (args: readonly string[], root: string) =>
  Bun.spawnSync({
    cmd: [process.execPath, script, "--transcripts", "--json", ...args],
    env: { HOME: root },
    stdout: "pipe",
    stderr: "pipe",
  });

test("reads nested JSONL without following symlinks and filters a rolling day window", () => {
  const output = run(
    ["--since-days", "7", "--limit", "1", "--agent", "all"],
    home,
  );
  expect(output.exitCode).toBe(0);
  const text = output.stdout.toString();
  const report = parseReport(text);
  expect(report.total).toBe(2);
  expect(report.summary).toEqual([
    { agent: "claude", reason: "Never print environment variables", count: 2 },
  ]);
  expect(report.denials).toHaveLength(1);
  expect(report.denials[0]?.timestamp).toBe(newest);
  expect(report.sourceFailures).toEqual([]);
  expect(report.failedToolCalls).toHaveLength(1);
  expect(report.failedToolCalls[0]?.agent).toBe("pi");
  expect(text).not.toContain("input-canary");
  expect(text).not.toContain("--exclude");
  expect(text).not.toContain("output-canary");
  expect(report.sops.total).toBe(2);
  expect(report.sops.calls).toHaveLength(1);
  expect(report.sops.calls[0]?.timestamp).toBe(newest);
  expect(report.sops.summary).toEqual([
    {
      agent: "claude",
      match: "sops-token",
      outcome: "tool-result",
      decryptReference: false,
      count: 2,
    },
  ]);
  const cutoff = Date.parse(report.since);
  expect(cutoff).toBeGreaterThanOrEqual(now - 7 * 86_400_000);
  expect(cutoff).toBeLessThanOrEqual(Date.now() - 7 * 86_400_000);
});

test("filters agents before reading directories", () => {
  const output = run(["--since-days", "7", "--agent", "codex"], home);
  const text = output.stdout.toString();
  const report = parseReport(text);
  expect(output.exitCode).toBe(0);
  expect(report.total).toBe(0);
  expect(report.denials).toEqual([]);
  expect(report.sops.total).toBe(0);
});

test("accepts pi and respects since-days", () => {
  const output = run(["--since-days", "7", "--agent", "pi"], home);
  expect(output.exitCode).toBe(0);
  const text = output.stdout.toString();
  const report = parseReport(text);
  expect(report.failedToolCalls[0]?.callId).toBe("pi-call");
  const later = run(["--since-days", "1", "--agent", "pi"], home);
  const laterText = later.stdout.toString();
  const laterReport = parseReport(laterText);
  expect(laterReport.failedToolCalls).toEqual([]);
});

test("includes matching Pi hook errors in counts and summaries", async () => {
  const root = join(home, "pi-only");
  const directory = join(root, ".pi", "agent", "sessions");
  await mkdir(directory, { recursive: true });
  const path = join(directory, "session.jsonl");
  const entries = [
    { type: "session", id: "session", cwd: "/project" },
    {
      type: "message",
      timestamp: newest,
      message: {
        role: "assistant",
        content: [
          {
            type: "toolCall",
            id: "blocked",
            name: "bash",
            arguments: { command: "bun secret-input-canary" },
          },
        ],
      },
    },
    {
      type: "message",
      timestamp: newest,
      message: {
        role: "toolResult",
        toolCallId: "blocked",
        toolName: "bash",
        isError: true,
        content: [
          {
            type: "text",
            text: "Must be prefixed by `sandbox` (e.g. `sandbox cargo build`). Use `just` recipes when available.",
          },
        ],
      },
    },
  ];
  const content = entries.map((entry) => JSON.stringify(entry)).join("\n");
  await Bun.write(path, content);
  const output = run(["--since-days", "7", "--agent", "pi"], root);
  expect(output.exitCode).toBe(0);
  const text = output.stdout.toString();
  const report = parseReport(text);
  expect(report.total).toBe(1);
  expect(report.denials[0]?.agent).toBe("pi");
  expect(report.summary[0]?.count).toBe(1);
  expect(text).not.toContain("secret-input-canary");
});

test("defaults agent to all and limit to 100", async () => {
  const root = join(home, "defaults");
  const directory = join(root, ".claude", "projects");
  await mkdir(directory, { recursive: true });
  const codex = join(root, ".codex", "sessions");
  const pi = join(root, ".pi", "agent", "sessions");
  await mkdir(codex, { recursive: true });
  await mkdir(pi, { recursive: true });
  const entries = Array.from({ length: 105 }, (_, index) => {
    const id = String(index);
    return transcript(newest, id);
  }).flat();
  const content = entries.map((entry) => JSON.stringify(entry)).join("\n");
  const path = join(directory, "session.jsonl");
  await Bun.write(path, content);
  const output = run(["--since-days", "7"], root);
  expect(output.exitCode).toBe(0);
  const text = output.stdout.toString();
  const report = parseReport(text);
  expect(report.agent).toBe("all");
  expect(report.total).toBe(105);
  expect(report.denials).toHaveLength(100);
});

test("reports unavailable transcript directories and exits unsuccessfully", () => {
  const unavailable = join(home, "unavailable");
  const output = run(["--since-days", "7", "--agent", "claude"], unavailable);
  expect(output.exitCode).toBe(1);
  const text = output.stdout.toString();
  const report = parseReport(text);
  expect(report.sourceFailures[0]?.code).toBe("read-directory");
});

test.each([
  { args: [] },
  { args: ["--since-days", "not-a-number"] },
  { args: ["--since-days", "0"] },
  { args: ["--since-days=-1"] },
  { args: ["--since-days", "1.5"] },
  { args: ["--since-days", "2", "--limit", "0"] },
  { args: ["--since-days", "2", "--agent", "unknown"] },
])("rejects missing or invalid options: $args", ({ args }) => {
  const output = run(args, home);
  expect(output.exitCode).toBe(1);
  const error = output.stderr.toString();
  expect(error).toContain("required options");
});

test("transcript output shows triggering commands grouped by exact error", () => {
  const output = Bun.spawnSync({
    cmd: [
      process.execPath,
      script,
      "--transcripts",
      "--since-days",
      "7",
      "--agent",
      "claude",
    ],
    env: { HOME: home },
    stdout: "pipe",
    stderr: "pipe",
  });
  expect(output.exitCode).toBe(0);
  const text = output.stdout.toString();
  const json = parseJson(text);
  if (!json.success) throw new Error(json.error.code);
  const grouped = Grouped.parse(json.data);
  expect(grouped["never-print-environment-variables"]).toHaveLength(2);
  expect(grouped["never-print-environment-variables"]?.[0]).toEqual({
    command: "fd --exclude .env .",
    project: "project",
    agent: "claude",
    timestamp: newest,
  });
  expect(text).not.toContain(".jsonl");
  expect(text).not.toContain("sops");
  expect(text).not.toContain("failedToolCalls");
});

test("provides help without reading transcript directories", () => {
  const unavailable = join(home, "unavailable");
  const output = run(["--help"], unavailable);
  expect(output.exitCode).toBe(0);
  const text = output.stdout.toString();
  expect(text).toContain("--since-days");
  expect(text).toContain("--limit defaults to 100");
  expect(text).toContain("--agent defaults to all");
});
