import { expect, test } from "bun:test";
import { piDenialReason } from "./pi-denials.ts";

const content = (text: string) => [{ type: "text", text }];

test.each([
  "Never print environment variables",
  "Variable output permits plain lowercase names, HOME, USER, PATH, and exit status ($?).",
  "Never print variables with TOKEN, KEY, SECRET, PASSWORD, or PASSPHRASE name parts (case-insensitive). Unsupported output expansions remain blocked.",
  "Access to sensitive file blocked",
  "Could not inspect this command because parsing failed.",
  "Must be prefixed by `sandbox` (e.g. `sandbox cargo build`). Use `just` recipes when available.",
])("matches exact hook errors: %s", (reason) => {
  const blocks = content(reason);
  const actual = piDenialReason("bash", blocks);
  expect(actual).toBe(reason);
});

test.each([
  "Command exited with code 1",
  "output: Never print environment variables",
  "Never print environment variables\nsecret-canary",
])(
  "does not match ordinary output or errors containing hook text: %s",
  (text) => {
    const blocks = content(text);
    const reason = piDenialReason("bash", blocks);
    expect(reason).toBeNull();
  },
);

test("matches only tools handled by the corresponding hook policy", () => {
  const sensitive = content("Access to sensitive file blocked");
  const environment = content("Never print environment variables");
  const readReason = piDenialReason("read", sensitive);
  const grepReason = piDenialReason("grep", sensitive);
  const editReason = piDenialReason("edit", sensitive);
  const unrelatedReason = piDenialReason("read", environment);
  expect(readReason).toBe("Access to sensitive file blocked");
  expect(grepReason).toBe("Access to sensitive file blocked");
  expect(editReason).toBeNull();
  expect(unrelatedReason).toBeNull();
});

test("does not match additional blocks or malformed content", () => {
  const blocks = [
    ...content("Never print environment variables"),
    ...content("secret-canary"),
  ];
  const multipleBlocks = piDenialReason("bash", blocks);
  const malformed = piDenialReason("bash", "Never print environment variables");
  expect(multipleBlocks).toBeNull();
  expect(malformed).toBeNull();
});
