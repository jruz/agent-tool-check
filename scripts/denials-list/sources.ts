import { readdir } from "node:fs/promises";
import { join } from "node:path";
import { readTranscript } from "./transcripts.ts";
import type {
  AgentSelection,
  SourceErrorCode,
  SourceFailure,
  SourceReports,
  SourceResult,
} from "./sources.types.ts";

const sourceFailure = (
  code: SourceErrorCode,
  source: string,
  error: unknown,
): SourceFailure => {
  if (!(error instanceof Error)) throw error;
  return { code, source, message: error.message };
};

const transcriptPaths = async (
  directory: string,
): Promise<SourceResult<readonly string[]>> => {
  const entries = await readdir(directory, { withFileTypes: true })
    .then((data) => ({ success: true as const, data }))
    .catch((error: unknown) => {
      const failure = sourceFailure("read-directory", directory, error);
      return { success: false as const, error: failure };
    });
  if (!entries.success) return entries;
  const requests = entries.data.map(
    async (entry): Promise<SourceResult<readonly string[]>> => {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) return transcriptPaths(path);
      const data =
        entry.isFile() && entry.name.endsWith(".jsonl") ? [path] : [];
      return { success: true, data };
    },
  );
  const children = await Promise.all(requests);
  const failure = children.find((child) => !child.success);
  if (failure !== undefined) return failure;
  const data = children.flatMap((child) => (child.success ? child.data : []));
  return { success: true, data };
};

export const readSources = async (
  home: string,
  selection: AgentSelection,
): Promise<SourceReports> => {
  const roots = [
    { agent: "claude" as const, source: join(home, ".claude", "projects") },
    { agent: "codex" as const, source: join(home, ".codex", "sessions") },
    { agent: "pi" as const, source: join(home, ".pi", "agent", "sessions") },
  ].filter((root) => selection === "all" || selection === root.agent);
  const rootRequests = roots.map(async (root) => {
    const paths = await transcriptPaths(root.source);
    if (!paths.success) return { reports: [], failures: [paths.error] };
    const fileRequests = paths.data.map(async (source) => {
      const file = Bun.file(source);
      const content = await file
        .text()
        .then((data) => ({ success: true as const, data }))
        .catch((error: unknown) => {
          const failure = sourceFailure("read-transcript", source, error);
          return { success: false as const, error: failure };
        });
      if (!content.success) return { reports: [], failures: [content.error] };
      const report = readTranscript({
        agent: root.agent,
        source,
        content: content.data,
      });
      return { reports: [report], failures: [] };
    });
    const files = await Promise.all(fileRequests);
    return {
      reports: files.flatMap((file) => file.reports),
      failures: files.flatMap((file) => file.failures),
    };
  });
  const rootsReports = await Promise.all(rootRequests);
  return {
    reports: rootsReports.flatMap((root) => root.reports),
    failures: rootsReports.flatMap((root) => root.failures),
  };
};
