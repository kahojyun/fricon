import type { ApiError as WireError } from "@/shared/lib/bindings";

export class ApiError extends Error {
  readonly code: WireError["code"];
  readonly apiMessage: string;

  constructor(error: WireError) {
    super(`[${error.code}] ${error.message}`);
    this.name = "ApiError";
    Object.setPrototypeOf(this, new.target.prototype);
    this.code = error.code;
    this.apiMessage = error.message;
  }
}

export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

export function unwrapResult<T>(
  result: { status: "ok"; data: T } | { status: "error"; error: WireError },
): T {
  if (result.status === "ok") {
    return result.data;
  }
  throw new ApiError(result.error);
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
  T extends {
    createdAt: string;
    trashedAt: string | null;
    deletedAt: string | null;
  },
>(
  value: T,
): Omit<T, "createdAt" | "trashedAt" | "deletedAt"> & {
  createdAt: Date;
  trashedAt: Date | null;
  deletedAt: Date | null;
} {
  return {
    ...value,
    createdAt: toDate(value.createdAt),
    trashedAt: value.trashedAt === null ? null : toDate(value.trashedAt),
    deletedAt: value.deletedAt === null ? null : toDate(value.deletedAt),
  };
}
