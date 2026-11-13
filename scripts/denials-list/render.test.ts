import { expect, test } from "bun:test";
import { groupDenials } from "./render.ts";
import type { TranscriptDenial } from "./transcripts.ts";

const denial = (reason: string, command: string | null): TranscriptDenial => ({
  agent: "pi",
  timestamp: "2026-11-03T12:00:00Z",
  project: "/projects/example",
  session: "session",
  tool: "bash",
  callId: "call",
  executable: "bun",
  command,
  reason,
  source: "/session.jsonl",
  line: 3,
});

test("groups errors under readable kebab-case keys with occurrence details", () => {
  const sandbox =
    "Must be prefixed by `sandbox` (e.g. `sandbox cargo build`). Use `just` recipes when available.";
  const entries = [
    denial(sandbox, "bun test\nbun run check"),
    denial("Never print environment variables", "printenv"),
    denial(sandbox, "cargo test"),
  ];
  const grouped = groupDenials(entries, 100);
  expect(grouped).toEqual({
    "must-be-prefixed-by-sandbox-e-g-sandbox-cargo-build-use-just-recipes-when-available":
      [
        {
          command: "bun test\nbun run check",
          project: "example",
          agent: "pi",
          timestamp: "2026-11-03T12:00:00Z",
        },
        {
          command: "cargo test",
          project: "example",
          agent: "pi",
          timestamp: "2026-11-03T12:00:00Z",
        },
      ],
    "never-print-environment-variables": [
      {
        command: "printenv",
        project: "example",
        agent: "pi",
        timestamp: "2026-11-03T12:00:00Z",
      },
    ],
  });
});

test("uses the actual error wording rather than category aliases", () => {
  const entries = [denial("Access to sensitive file blocked", "read /path")];
  const grouped = groupDenials(entries, 100);
  const keys = Object.keys(grouped);
  expect(keys).toEqual(["access-to-sensitive-file-blocked"]);
});

test("limits occurrences globally before grouping", () => {
  const entries = [
    denial("First error!", "first command"),
    denial("Second error.", "second command"),
    denial("First error!", "third command"),
  ];
  const grouped = groupDenials(entries, 1);
  const keys = Object.keys(grouped);
  expect(keys).toEqual(["first-error"]);
  expect(grouped["first-error"]).toHaveLength(1);
  expect(grouped["first-error"]?.[0]?.command).toBe("first command");
});

test("preserves missing commands as null and returns an empty object for no matches", () => {
  const empty = groupDenials([], 100);
  expect(empty).toEqual({});
  const entries = [denial("Error", null)];
  const grouped = groupDenials(entries, 100);
  expect(grouped["error"]?.[0]?.command).toBeNull();
});
