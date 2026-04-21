import { beforeEach, describe, expect, it, vi } from "vitest";

const { tauriInvokeMock } = vi.hoisted(() => ({
  tauriInvokeMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: tauriInvokeMock,
}));

import {
  ApiError,
  invokeRaw,
  invokeRawBytes,
  isApiError,
  normalizeRawBytes,
  normalizeCreatedAtDate,
  normalizeDatasetDates,
  toDate,
  unwrapResult,
} from "./tauri";

describe("tauri helpers", () => {
  beforeEach(() => {
    tauriInvokeMock.mockReset();
  });

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

  it("throws a structured ApiError for error results", () => {
    try {
      unwrapResult({
        status: "error",
        error: {
          code: "archive_version_unsupported",
          message: "archive is too new",
        },
      });
      throw new Error("expected unwrapResult to throw");
    } catch (error) {
      expect(isApiError(error)).toBe(true);
      expect(error).toBeInstanceOf(ApiError);
      expect((error as ApiError).code).toBe("archive_version_unsupported");
      expect((error as ApiError).apiMessage).toBe("archive is too new");
    }
  });

  it("wraps raw invoke object rejections as ApiError", async () => {
    tauriInvokeMock.mockRejectedValueOnce({
      code: "charts",
      message: "binary payload decode failed",
    });

    await expect(invokeRaw("dataset_chart_data")).rejects.toMatchObject({
      name: "ApiError",
      code: "charts",
      apiMessage: "binary payload decode failed",
      message: "[charts] binary payload decode failed",
    });
  });

  it("passes through raw invoke Error rejections unchanged", async () => {
    const error = new Error("transport exploded");
    tauriInvokeMock.mockRejectedValueOnce(error);

    await expect(invokeRaw("dataset_chart_data")).rejects.toBe(error);
  });

  it("wraps raw byte invoke object rejections as ApiError", async () => {
    tauriInvokeMock.mockRejectedValueOnce({
      code: "workspace",
      message: "workspace unavailable",
    });

    await expect(invokeRawBytes("dataset_chart_data")).rejects.toMatchObject({
      name: "ApiError",
      code: "workspace",
      apiMessage: "workspace unavailable",
      message: "[workspace] workspace unavailable",
    });
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

  it("normalizes ArrayBuffer payloads to Uint8Array", () => {
    const value = new Uint8Array([4, 5, 6]).buffer;
    expect(normalizeRawBytes(value)).toEqual(new Uint8Array([4, 5, 6]));
  });

  it("rejects non-ArrayBuffer raw payloads", () => {
    const value = new Uint8Array([7, 8, 9]);
    expect(() => normalizeRawBytes(value)).toThrow(
      "Expected an ArrayBuffer raw response from backend.",
    );
  });
});
