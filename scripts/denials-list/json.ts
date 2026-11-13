export type JsonErrorCode = "invalid-json";
export type JsonFailure = Readonly<{ code: JsonErrorCode }>;
type JsonResult =
  | Readonly<{ success: true; data: unknown }>
  | Readonly<{ success: false; error: JsonFailure }>;

export const parseJson = (text: string): JsonResult => {
  try {
    const data: unknown = JSON.parse(text);
    return { success: true, data };
  } catch (error) {
    if (!(error instanceof SyntaxError)) throw error;
    return { success: false, error: { code: "invalid-json" } };
  }
};
