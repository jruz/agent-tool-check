import { afterAll, expect, test } from "bun:test";
import { mkdtemp, mkdir, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { z } from "zod";
import { readDenialLog } from "./log.ts";
import { parseJson } from "./json.ts";

const temporary = tmpdir();
const prefix = join(temporary, "denial-log-");
const home = await mkdtemp(prefix);
const state = join(home, "custom-state");
const directory = join(state, "agent-tool-check");
await mkdir(directory, { recursive: true });
const source = join(directory, "denial.log");
const entry = {
  agent: "pi",
  timestamp: new Date().toISOString(),
  project: "/project",
  session: null,
  callId: "call-1",
  tool: "bash",
  executable: "cat",
  command: "cat .env",
  reason: "Never access .env files directly",
} as const;
await Bun.write(source, `${JSON.stringify(entry)}\n`);
const script = fileURLToPath(new URL("../denials-list.ts", import.meta.url));
const run = (args: readonly string[]) =>
  Bun.spawnSync({
    cmd: [process.execPath, script, "--since-days", "2", ...args],
    env: { HOME: home, XDG_STATE_HOME: state },
    stdout: "pipe",
    stderr: "pipe",
  });
afterAll(async () => {
  await rm(home, { recursive: true });
});

test("reads complete denial records with log line provenance", async () => {
  const report = await readDenialLog(source);
  expect(report.failures).toEqual([]);
  expect(report.reports[0]?.denials).toEqual([{ ...entry, source, line: 1 }]);
});

test("default CLI reads its own log without agent transcripts", () => {
  const result = run([]);
  expect(result.exitCode).toBe(0);
  const error = result.stderr.toString();
  expect(error).toBe("");
  const text = result.stdout.toString();
  const json = parseJson(text);
  if (!json.success) throw new Error(json.error.code);
  const command = z.string();
  const detail = z.object({ command }).readonly();
  const details = z.array(detail).readonly();
  const key = z.string();
  const grouped = z.record(key, details).readonly().parse(json.data);
  expect(grouped["never-access-env-files-directly"]?.[0]?.command).toBe(
    "cat .env",
  );
});

test("metadata mode omits commands and filters agents", () => {
  const result = run(["--json"]);
  expect(result.exitCode).toBe(0);
  const text = result.stdout.toString();
  expect(text).not.toContain("cat .env");
  const filtered = run(["--json", "--agent", "claude"]);
  const filteredText = filtered.stdout.toString();
  const json = parseJson(filteredText);
  if (!json.success) throw new Error(json.error.code);
  const total = z.number();
  const report = z.object({ total }).readonly().parse(json.data);
  expect(report.total).toBe(0);
});

test("reports malformed records without discarding valid entries", async () => {
  const invalid = join(home, "invalid.log");
  await Bun.write(invalid, `not-json\n${JSON.stringify(entry)}\n{}\n`);
  const report = await readDenialLog(invalid);
  expect(report.reports[0]?.denials).toEqual([
    { ...entry, source: invalid, line: 2 },
  ]);
  expect(report.reports[0]?.failures).toEqual([
    { code: "invalid-record", source: invalid, line: 1 },
    { code: "invalid-record", source: invalid, line: 3 },
  ]);
});

test("reports missing denial logs visibly", async () => {
  const missing = join(home, "missing.log");
  const report = await readDenialLog(missing);
  expect(report.failures[0]?.code).toBe("read-log");
});
