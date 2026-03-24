import type { DatasetInfo, DatasetTagBatchResult } from "../api/types";

export type DatasetTagOperation = "add" | "remove";

export interface DatasetTagMenuTarget {
  targetIds: number[];
  targetLabel: string;
  removableTags: string[];
}

interface DatasetTagMutationNotifier {
  success: (message: string) => void;
  error: (message: string, options?: { description?: string }) => void;
  warning: (message: string, options?: { description?: string }) => void;
}

interface RunDatasetTagMutationArgs {
  operation: DatasetTagOperation;
  targetIds: number[];
  tag: string;
  batchAddTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetTagBatchResult[]>;
  batchRemoveTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetTagBatchResult[]>;
  notify: DatasetTagMutationNotifier;
}

export function deriveDatasetTagMenuTarget(
  dataset: DatasetInfo,
  selectedDatasets: DatasetInfo[],
): DatasetTagMenuTarget {
  const includesDataset = selectedDatasets.some(
    (selectedDataset) => selectedDataset.id === dataset.id,
  );
  const targetDatasets =
    selectedDatasets.length > 1 && includesDataset
      ? selectedDatasets
      : [dataset];

  return {
    targetIds: targetDatasets.map((targetDataset) => targetDataset.id),
    targetLabel: targetDatasets.length > 1 ? ` (${targetDatasets.length})` : "",
    removableTags: Array.from(
      new Set(targetDatasets.flatMap((targetDataset) => targetDataset.tags)),
    ).sort(),
  };
}

function getTagMutationDescription(results: DatasetTagBatchResult[]) {
  return results
    .flatMap((result) => {
      const messages: string[] = [];
      if (result.addError) {
        messages.push(`ID ${result.id}: add: ${result.addError.message}`);
      }
      if (result.removeError) {
        messages.push(`ID ${result.id}: remove: ${result.removeError.message}`);
      }
      return messages;
    })
    .join("\n");
}

export function notifyDatasetTagMutationResult(
  operation: DatasetTagOperation,
  tag: string,
  results: DatasetTagBatchResult[],
  notify: DatasetTagMutationNotifier,
) {
  const actionLabel = operation === "add" ? "Added" : "Removed";
  const actionVerb = operation === "add" ? "add" : "remove";
  const preposition = operation === "add" ? "to" : "from";
  const successCount = results.filter((result) => result.success).length;
  const failedCount = results.length - successCount;

  if (failedCount === 0) {
    notify.success(
      `${actionLabel} tag "${tag}" ${preposition} ${successCount} dataset(s).`,
    );
    return;
  }

  if (successCount === 0) {
    notify.error(
      `Failed to ${actionVerb} tag "${tag}" ${preposition} ${failedCount} dataset(s).`,
      {
        description: getTagMutationDescription(results),
      },
    );
    return;
  }

  notify.warning(
    `${actionLabel} tag "${tag}" ${preposition} ${successCount} dataset(s), but ${failedCount} failed.`,
    {
      description: getTagMutationDescription(results),
    },
  );
}

export function notifyDatasetTagMutationError(
  operation: DatasetTagOperation,
  tag: string,
  error: unknown,
  notify: DatasetTagMutationNotifier,
) {
  const actionVerb = operation === "add" ? "add" : "remove";
  notify.error(
    error instanceof Error
      ? error.message
      : `Failed to ${actionVerb} tag "${tag}".`,
  );
}

export async function runDatasetTagMutation({
  operation,
  targetIds,
  tag,
  batchAddTags,
  batchRemoveTags,
  notify,
}: RunDatasetTagMutationArgs) {
  try {
    const results =
      operation === "add"
        ? await batchAddTags(targetIds, [tag])
        : await batchRemoveTags(targetIds, [tag]);
    notifyDatasetTagMutationResult(operation, tag, results, notify);
    return results;
  } catch (error) {
    notifyDatasetTagMutationError(operation, tag, error, notify);
    return null;
  }
}
