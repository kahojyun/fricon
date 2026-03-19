import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createDatasetColumns } from "../ui/DatasetTableColumns";
import {
  COLUMN_VISIBILITY_STORAGE_KEY,
  useDatasetColumnVisibility,
} from "./useDatasetColumnVisibility";

function createColumns() {
  return createDatasetColumns({
    toggleFavorite: vi.fn().mockResolvedValue(undefined),
  });
}

describe("useDatasetColumnVisibility", () => {
  beforeEach(() => {
    window.localStorage.clear();
  });

  it("uses column metadata defaults and always keeps the name column visible", () => {
    const { result } = renderHook(() =>
      useDatasetColumnVisibility(createColumns()),
    );

    expect(result.current.columnVisibility).toMatchObject({
      favorite: true,
      id: true,
      name: true,
      status: true,
      tags: false,
      createdAt: false,
    });
  });

  it("falls back safely when stored column visibility is invalid JSON", () => {
    window.localStorage.setItem(COLUMN_VISIBILITY_STORAGE_KEY, "not-json");

    const { result } = renderHook(() =>
      useDatasetColumnVisibility(createColumns()),
    );

    expect(result.current.columnVisibility).toMatchObject({
      name: true,
      status: true,
      tags: false,
      createdAt: false,
    });
  });

  it("sanitizes stored visibility and ignores unknown columns", () => {
    window.localStorage.setItem(
      COLUMN_VISIBILITY_STORAGE_KEY,
      JSON.stringify({
        name: false,
        status: false,
        tags: true,
        unknown: true,
      }),
    );

    const { result } = renderHook(() =>
      useDatasetColumnVisibility(createColumns()),
    );

    expect(result.current.columnVisibility).toMatchObject({
      name: true,
      status: false,
      tags: true,
      createdAt: false,
    });
    expect(result.current.columnVisibility).not.toHaveProperty("unknown");
  });

  it("shows all columns, resets defaults, and persists visibility changes", async () => {
    const { result } = renderHook(() =>
      useDatasetColumnVisibility(createColumns()),
    );

    act(() => {
      result.current.handleColumnVisibilityChange("status", false);
    });

    await waitFor(() => {
      expect(
        JSON.parse(
          window.localStorage.getItem(COLUMN_VISIBILITY_STORAGE_KEY) ?? "{}",
        ),
      ).toMatchObject({
        status: false,
        name: true,
      });
    });

    act(() => {
      result.current.handleColumnVisibilityChange("name", false);
    });

    expect(result.current.columnVisibility.name).toBe(true);

    act(() => {
      result.current.showAllColumns();
    });

    expect(result.current.columnVisibility).toMatchObject({
      name: true,
      tags: true,
      createdAt: true,
    });

    act(() => {
      result.current.resetColumnVisibilityToDefault();
    });

    expect(result.current.columnVisibility).toMatchObject({
      name: true,
      status: true,
      tags: false,
      createdAt: false,
    });
  });
});
