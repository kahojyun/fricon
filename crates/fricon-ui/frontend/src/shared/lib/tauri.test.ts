import { describe, expect, it } from "vitest";
import {
  normalizeCreatedAtDate,
  normalizeDatasetDates,
  toDate,
  unwrapResult,
} from "./tauri";

describe("tauri helpers", () => {
  it("unwraps ok results", () => {
    expect(unwrapResult({ status: "ok", data: 42 })).toBe(42);
  });

  it("throws the backend error message for error results", () => {
    expect(() =>
      unwrapResult({
        status: "error",
        error: { code: "internal", message: "backend exploded" },
      }),
    ).toThrow("[internal] backend exploded");
  });

  it("rejects invalid date values", () => {
    expect(() => toDate("not-a-date")).toThrow(
      "Invalid date value from backend: not-a-date",
    );
  });

  it("normalizes createdAt to Date", () => {
    const normalized = normalizeCreatedAtDate({
      id: 1,
      createdAt: "2026-01-01T00:00:00Z",
    });

    expect(normalized.createdAt).toBeInstanceOf(Date);
    expect(normalized.createdAt.toISOString()).toBe("2026-01-01T00:00:00.000Z");
  });

  it("normalizes createdAt and trashedAt to Date values", () => {
    const normalized = normalizeDatasetDates({
      id: 1,
      createdAt: "2026-01-01T00:00:00Z",
      trashedAt: "2026-01-02T03:04:05Z",
      deletedAt: null,
    });

    expect(normalized.createdAt).toBeInstanceOf(Date);
    expect(normalized.trashedAt).toBeInstanceOf(Date);
    expect(normalized.trashedAt?.toISOString()).toBe(
      "2026-01-02T03:04:05.000Z",
    );
    expect(normalized.deletedAt).toBeNull();
  });
});
