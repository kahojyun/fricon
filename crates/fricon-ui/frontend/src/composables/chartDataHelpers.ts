import { DataType, Float64, Struct, Vector } from "apache-arrow";
import type { TypedArray } from "apache-arrow/interfaces";
import type { ColumnInfo } from "@/backend.ts";

/** Result of extracting X-axis data from a trace vector */
export interface TraceXAxisResult {
  x: number[] | TypedArray;
  y: Vector;
}

/**
 * Generates X-axis values from a trace vector.
 * Handles three formats:
 * - List type: uses indices as X
 * - Struct with x/y: uses x values directly
 * - Struct with x0/step: generates X from x0 + i * step
 */
export function extractTraceXAxis(
  traceVector: Vector,
  rowIndex: number,
): TraceXAxisResult | null {
  if (DataType.isList(traceVector.type)) {
    const yVec = traceVector.get(rowIndex) as Vector | null;
    if (!yVec) return null;
    return {
      x: Int32Array.from({ length: yVec.length }, (_, i) => i),
      y: yVec,
    };
  }

  const yChild = traceVector.getChild("y")?.get(rowIndex) as Vector | null;
  if (!yChild) return null;

  if (traceVector.numChildren === 2) {
    // Has explicit x values
    const xChild = traceVector.getChild("x")?.get(rowIndex) as Vector | null;
    return {
      x: xChild
        ? (xChild.toArray() as TypedArray)
        : generateIndices(yChild.length),
      y: yChild,
    };
  }

  // x0/step format
  const rowStruct = (
    traceVector as Vector<Struct<{ x0: Float64; step: Float64 }>>
  ).get(rowIndex);

  if (!rowStruct) {
    return {
      x: generateIndices(yChild.length),
      y: yChild,
    };
  }

  return {
    x: Float64Array.from(
      { length: yChild.length },
      (_, i) => rowStruct.x0 + i * rowStruct.step,
    ),
    y: yChild,
  };
}

/** Generates an array of indices [0, 1, 2, ..., length-1] */
export function generateIndices(length: number): Int32Array {
  return Int32Array.from({ length }, (_, i) => i);
}

/**
 * Finds column indices for the given column names.
 * Returns null if any required column is not found.
 */
export function getColumnIndices(
  columns: ColumnInfo[],
  names: (string | undefined)[],
): number[] | null {
  const indices: number[] = [];
  for (const name of names) {
    if (!name) return null;
    const idx = columns.findIndex((c) => c.name === name);
    if (idx === -1) return null;
    indices.push(idx);
  }
  return indices;
}

export type ComplexViewOption = "real" | "imag" | "mag" | "arg";

interface ComplexComponents {
  reals: Float64Array;
  imags: Float64Array;
}

/**
 * Extracts real and imaginary components from complex data.
 * Handles both Arrow Vector format and plain object arrays.
 */
export function extractComplexComponents(
  data: Vector | { toArray: () => unknown[]; length: number },
): ComplexComponents {
  const rawArray = data.toArray();

  if (
    Array.isArray(rawArray) &&
    rawArray.length > 0 &&
    typeof rawArray[0] === "object"
  ) {
    // Plain object array format
    const arr = rawArray as { real: number; imag: number }[];
    const reals = new Float64Array(arr.length);
    const imags = new Float64Array(arr.length);
    for (let i = 0; i < arr.length; i++) {
      reals[i] = arr[i]!.real;
      imags[i] = arr[i]!.imag;
    }
    return { reals, imags };
  }

  // Arrow Vector with children
  const vector = data as Vector;
  return {
    reals: vector.getChild("real")!.toArray() as Float64Array,
    imags: vector.getChild("imag")!.toArray() as Float64Array,
  };
}

/**
 * Transforms complex components based on the selected view option.
 */
export function transformComplexView(
  components: ComplexComponents,
  option: ComplexViewOption,
): Float64Array {
  const { reals, imags } = components;
  const result = new Float64Array(reals.length);

  switch (option) {
    case "real":
      return reals;
    case "imag":
      return imags;
    case "mag":
      for (let i = 0; i < reals.length; i++) {
        result[i] = Math.sqrt(reals[i]! * reals[i]! + imags[i]! * imags[i]!);
      }
      return result;
    case "arg":
      for (let i = 0; i < reals.length; i++) {
        result[i] = Math.atan2(imags[i]!, reals[i]!);
      }
      return result;
  }
}

/** Creates a mock Vector-like object for accumulated heatmap data */
export function createMockVector<T>(data: T[]): {
  toArray: () => T[];
  length: number;
  get: (i: number) => T | undefined;
} {
  return {
    toArray: () => data,
    length: data.length,
    get: (i: number) => data[i],
  };
}
