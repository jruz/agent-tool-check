import { expect, test } from "bun:test";
import { redactCommand } from "./redact.ts";

test.each([
  "TOKEN=private-value curl https://example.com",
  "export API_KEY='private-value'",
  "curl --token private-value https://example.com",
  "curl -H 'Authorization: Bearer private-value' https://example.com",
  "curl -u user:private-value https://example.com",
  "curl https://user:private-value@example.com",
])("redacts common credentials: %s", (command) => {
  const text = redactCommand(command);
  expect(text).not.toContain("private-value");
  expect(text).toContain("[REDACTED]");
});

test("preserves ordinary and multiline commands", () => {
  const command = "bun test --coverage\ncargo test";
  const text = redactCommand(command);
  expect(text).toBe(command);
});
