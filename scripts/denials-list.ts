import { parseArgs } from "node:util";
import { homedir } from "node:os";
import { join } from "node:path";
import { readDenialLog } from "./denials-list/log.ts";
import { z } from "zod";
import { readSources } from "./denials-list/sources.ts";
import { denialMetadata } from "./denials-list/transcripts.ts";
import { groupDenials } from "./denials-list/render.ts";
import { log } from "./lib/log.ts";
import { output } from "./lib/output.ts";

const Options = z
  .object({
    "since-days": z.coerce.number().int().positive(),
    limit: z.coerce.number().int().positive().default(100),
    agent: z.enum(["all", "claude", "codex", "pi"]).default("all"),
  })
  .readonly();
const usage =
  "just denials-list --since-days N [--limit N] [--agent all|claude|codex|pi] [--json] [--transcripts]";
const { values } = parseArgs({
  args: Bun.argv.slice(2),
  options: {
    "since-days": { type: "string" },
    limit: { type: "string" },
    agent: { type: "string" },
    help: { type: "boolean" },
    json: { type: "boolean" },
    transcripts: { type: "boolean" },
  },
});

const main = async (): Promise<void> => {
  if (values.help === true) {
    output.text(
      `${usage}\n--since-days is required and selects the last N × 24 hours.\n--limit defaults to 100. --agent defaults to all.\nOutputs JSON keyed by hook error with occurrence details and commands. Common credential forms are redacted; commands may still contain sensitive text.\n--json outputs the full metadata report without commands.\nReads denial.log by default. --transcripts reads historical agent sessions and includes SOPS and failed-tool reports.\nCounts cover all matches; the limit selects the newest occurrences.`,
    );
    return;
  }
  const options = Options.safeParse(values);
  if (!options.success) {
    log.error(`required options: ${usage}`);
    process.exitCode = 1;
    return;
  }
  const since = new Date(
    Date.now() - options.data["since-days"] * 86_400_000,
  ).toISOString();
  const home = homedir();
  const stateHome = process.env["XDG_STATE_HOME"];
  const logPath = join(
    stateHome === undefined ? join(home, ".local", "state") : stateHome,
    "agent-tool-check",
    "denial.log",
  );
  const sources =
    values.transcripts === true
      ? await readSources(home, options.data.agent)
      : await readDenialLog(logPath);
  sources.failures.forEach((failure) => {
    log.error(`${failure.code}: ${failure.source}: ${failure.message}`);
  });
  if (sources.failures.length > 0) process.exitCode = 1;
  const reports = sources.reports;
  const denials = reports
    .flatMap((report) => report.denials)
    .filter((denial) => Date.parse(denial.timestamp) >= Date.parse(since))
    .filter(
      (denial) =>
        options.data.agent === "all" || denial.agent === options.data.agent,
    )
    .toSorted(
      (left, right) => Date.parse(right.timestamp) - Date.parse(left.timestamp),
    );
  const groups = Object.groupBy(denials, (denial) =>
    JSON.stringify([denial.agent, denial.reason]),
  );
  const summary = Object.values(groups)
    .map((entries) => {
      if (entries?.[0] === undefined) throw new Error("empty denial group");
      return {
        agent: entries[0].agent,
        reason: entries[0].reason,
        count: entries.length,
      };
    })
    .toSorted((left, right) => right.count - left.count);
  const sopsCalls = reports
    .flatMap((report) => report.sopsCalls)
    .filter((call) => Date.parse(call.timestamp) >= Date.parse(since))
    .toSorted(
      (left, right) => Date.parse(right.timestamp) - Date.parse(left.timestamp),
    );
  const sopsGroups = Object.groupBy(sopsCalls, (call) =>
    JSON.stringify([
      call.agent,
      call.match,
      call.outcome,
      call.decryptReference,
    ]),
  );
  const sopsSummary = Object.values(sopsGroups)
    .map((entries) => {
      const first = entries?.[0];
      if (first === undefined || entries === undefined)
        throw new Error("empty SOPS group");
      return {
        agent: first.agent,
        match: first.match,
        outcome: first.outcome,
        decryptReference: first.decryptReference,
        count: entries.length,
      };
    })
    .toSorted((left, right) => right.count - left.count);
  const failures = reports.flatMap((report) => report.failures);
  if (failures.length > 0) process.exitCode = 1;
  failures.forEach((failure) => {
    log.warn(`${failure.code}: ${failure.source}:${failure.line}`);
  });
  if (values.json !== true) {
    const grouped = groupDenials(denials, options.data.limit);
    output.json(grouped);
    return;
  }
  output.json({
    since,
    agent: options.data.agent,
    total: denials.length,
    summary,
    denials: denials.slice(0, options.data.limit).map(denialMetadata),
    sops: {
      total: sopsCalls.length,
      summary: sopsSummary,
      calls: sopsCalls.slice(0, options.data.limit),
    },
    failedToolCalls: reports
      .flatMap((report) => report.failedToolCalls)
      .filter((call) => Date.parse(call.timestamp) >= Date.parse(since))
      .toSorted(
        (left, right) =>
          Date.parse(right.timestamp) - Date.parse(left.timestamp),
      )
      .slice(0, options.data.limit),
    failures,
    sourceFailures: sources.failures,
  });
};

await main();
