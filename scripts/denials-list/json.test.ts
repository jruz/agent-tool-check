import { expect, test } from "bun:test";
import { parseJson } from "./json.ts";
import type { JsonErrorCode, JsonFailure } from "./json.ts";

const invalidJson: JsonErrorCode = "invalid-json";
const failure: JsonFailure = { code: invalidJson };

test("returns a typed failure for malformed JSON without its contents", () => {
  const result = parseJson("{private-value");
  expect(result).toEqual({
    success: false,
    error: failure,
  });
});

test("parses JSON values", () => {
  const result = parseJson('{"valid":true}');
  expect(result).toEqual({
    success: true,
    data: { valid: true },
  });
  const value = parseJson("null");
  expect(value).toEqual({ success: true, data: null });
});
