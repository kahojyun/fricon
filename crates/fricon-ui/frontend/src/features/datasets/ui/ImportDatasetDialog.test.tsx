import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ImportDatasetDialog } from "./ImportDatasetDialog";
import type { UiPreviewImportResult } from "../api/types";
import type { DuplicateBatchConflict } from "./useDatasetImportFlow";

function makePreviewResult(
  archivePath: string,
  uid: string,
  options: {
    name?: string;
    conflict?: UiPreviewImportResult["preview"]["conflict"];
  } = {},
): UiPreviewImportResult {
  return {
    archivePath,
    preview: {
      metadata: {
        uid,
        name: options.name ?? uid,
        description: "dataset preview",
        favorite: false,
        status: "Completed",
        createdAt: "2026-01-01T00:00:00Z",
        tags: [],
      },
      conflict: options.conflict ?? null,
    },
  };
}

function renderDialog({
  previewResults = [makePreviewResult("/tmp/alpha.tar.zst", "uid-a")],
  duplicateBatchConflicts = [],
}: {
  previewResults?: UiPreviewImportResult[];
  duplicateBatchConflicts?: DuplicateBatchConflict[];
} = {}) {
  render(
    <ImportDatasetDialog
      open
      onOpenChange={vi.fn()}
      previewResults={previewResults}
      duplicateBatchConflicts={duplicateBatchConflicts}
      isImporting={false}
      onConfirm={vi.fn()}
    />,
  );
}

describe("ImportDatasetDialog", () => {
  it("shows a blocking alert and disables confirm for duplicate batch UUIDs", () => {
    const alpha = makePreviewResult("/tmp/alpha.tar.zst", "dup-uid", {
      name: "Alpha",
    });
    const beta = makePreviewResult("/tmp/beta.tar.zst", "dup-uid", {
      name: "Beta",
    });

    renderDialog({
      previewResults: [alpha, beta],
      duplicateBatchConflicts: [
        {
          uid: "dup-uid",
          entries: [alpha, beta],
        },
      ],
    });

    expect(screen.getByText("Duplicate Dataset UUIDs")).toBeInTheDocument();
    expect(
      screen.getByText(
        /remove duplicate archives before importing this batch/i,
      ),
    ).toBeInTheDocument();
    expect(
      screen.getByText(/alpha\.tar\.zst, beta\.tar\.zst/i),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Import Dataset(s)" }),
    ).toBeDisabled();
  });

  it("keeps a non-duplicate batch confirmable", () => {
    renderDialog();

    expect(screen.getByText("alpha.tar.zst")).toBeInTheDocument();
    expect(
      screen.queryByText("Duplicate Dataset UUIDs"),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Import Dataset(s)" }),
    ).toBeEnabled();
  });

  it("shows overwrite messaging for workspace conflicts without batch duplicate blocking", () => {
    renderDialog({
      previewResults: [
        makePreviewResult("/tmp/beta.tar.zst", "uid-b", {
          name: "Beta",
          conflict: {
            existing: {
              uid: "uid-b",
              name: "Existing Beta",
              description: "existing",
              favorite: false,
              status: "Completed",
              createdAt: "2026-01-01T00:00:00Z",
              tags: [],
            },
            diffs: [
              {
                field: "name",
                existingValue: "Existing Beta",
                incomingValue: "Beta",
              },
            ],
          },
        }),
      ],
    });

    expect(
      screen.queryByText("Duplicate Dataset UUIDs"),
    ).not.toBeInTheDocument();
    expect(screen.getByText("Conflict Detected")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Overwrite & Import" }),
    ).toBeEnabled();
  });
});
