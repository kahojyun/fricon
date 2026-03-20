import type { TauriCommandError as WireError } from "@/shared/lib/bindings";

export function unwrapResult<T>(
  result: { status: "ok"; data: T } | { status: "error"; error: WireError },
): T {
  if (result.status === "ok") {
    return result.data;
  }
  throw new Error(result.error.message);
}

export async function invoke<T>(
  commandCall: Promise<
    { status: "ok"; data: T } | { status: "error"; error: WireError }
  >,
): Promise<T> {
  return unwrapResult(await commandCall);
}

export function toDate(value: string): Date {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    throw new Error(`Invalid date value from backend: ${value}`);
  }
  return date;
}

export function normalizeCreatedAtDate<T extends { createdAt: string }>(
  value: T,
): Omit<T, "createdAt"> & { createdAt: Date } {
  return {
    ...value,
    createdAt: toDate(value.createdAt),
  };
}

export function normalizeDatasetDates<
  T extends { createdAt: string; trashedAt: string | null },
>(
  value: T,
): Omit<T, "createdAt" | "trashedAt"> & {
  createdAt: Date;
  trashedAt: Date | null;
} {
  return {
    ...value,
    createdAt: toDate(value.createdAt),
    trashedAt: value.trashedAt === null ? null : toDate(value.trashedAt),
  };
}
