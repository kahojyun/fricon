import { describe, expect, it, vi } from "vitest";
import type { DatasetDeleteResult, DatasetInfo } from "../api/types";
import {
  deriveDatasetTagMenuTarget,
  runDatasetTagMutation,
} from "./datasetTableTagMenuLogic";

function makeDataset(overrides: Partial<DatasetInfo> = {}): DatasetInfo {
  return {
    id: 1,
    name: "Dataset 1",
    description: "desc",
    favorite: false,
    tags: ["vision"],
    status: "Completed",
    createdAt: new Date("2026-01-01T00:00:00Z"),
    trashedAt: null,
    ...overrides,
  };
}

function createNotifier() {
  return {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
  };
}

describe("datasetTableTagMenuLogic", () => {
  it("targets only the clicked dataset when there is no multi-row selection", () => {
    const dataset = makeDataset({ id: 5, tags: [] });
    const target = deriveDatasetTagMenuTarget(dataset, []);

    expect(target).toEqual({
      targetIds: [5],
      targetLabel: "",
      removableTags: [],
    });
  });

  it("targets all selected rows and combines removable tags", () => {
    const first = makeDataset({ id: 10, name: "Dataset A", tags: ["vision"] });
    const second = makeDataset({
      id: 11,
      name: "Dataset B",
      tags: ["audio", "vision"],
    });

    const target = deriveDatasetTagMenuTarget(second, [first, second]);

    expect(target).toEqual({
      targetIds: [10, 11],
      targetLabel: " (2)",
      removableTags: ["audio", "vision"],
    });
  });

  it("adds an existing tag for a single target row", async () => {
    const notifier = createNotifier();
    const batchAddTags = vi
      .fn()
      .mockResolvedValue([{ id: 5, success: true, error: null }]);
    const batchRemoveTags = vi.fn();

    const target = deriveDatasetTagMenuTarget(
      makeDataset({ id: 5, tags: [] }),
      [],
    );

    await runDatasetTagMutation({
      operation: "add",
      targetIds: target.targetIds,
      tag: "vision",
      batchAddTags,
      batchRemoveTags,
      notify: notifier,
    });

    expect(batchAddTags).toHaveBeenCalledWith([5], ["vision"]);
    expect(notifier.success).toHaveBeenCalledWith(
      'Added tag "vision" to 1 dataset(s).',
    );
  });

  it("adds a tag for all selected rows and reports partial failure", async () => {
    const notifier = createNotifier();
    const results: DatasetDeleteResult[] = [
      { id: 10, success: true, error: null },
      { id: 11, success: false, error: "locked" },
    ];
    const batchAddTags = vi.fn().mockResolvedValue(results);
    const batchRemoveTags = vi.fn();
    const first = makeDataset({ id: 10, name: "Dataset A", tags: [] });
    const second = makeDataset({ id: 11, name: "Dataset B", tags: [] });
    const target = deriveDatasetTagMenuTarget(second, [first, second]);

    await runDatasetTagMutation({
      operation: "add",
      targetIds: target.targetIds,
      tag: "vision",
      batchAddTags,
      batchRemoveTags,
      notify: notifier,
    });

    expect(batchAddTags).toHaveBeenCalledWith([10, 11], ["vision"]);
    expect(notifier.warning).toHaveBeenCalledWith(
      'Added tag "vision" to 1 dataset(s), but 1 failed.',
      {
        description: "ID 11: locked",
      },
    );
  });

  it("removes a tag and reports success", async () => {
    const notifier = createNotifier();
    const batchAddTags = vi.fn();
    const batchRemoveTags = vi
      .fn()
      .mockResolvedValue([{ id: 9, success: true, error: null }]);

    await runDatasetTagMutation({
      operation: "remove",
      targetIds: [9],
      tag: "vision",
      batchAddTags,
      batchRemoveTags,
      notify: notifier,
    });

    expect(batchRemoveTags).toHaveBeenCalledWith([9], ["vision"]);
    expect(notifier.success).toHaveBeenCalledWith(
      'Removed tag "vision" from 1 dataset(s).',
    );
  });
});
