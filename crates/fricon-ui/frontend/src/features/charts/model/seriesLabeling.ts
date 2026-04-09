import type { ChartOptions, ChartSeries } from "@/shared/lib/chartTypes";

interface ParsedSeriesLabel {
  original: string;
  base: string;
  paren: string | null;
  bracket: string | null;
}

export function deriveSharedSeriesLabel(
  series: Pick<ChartSeries, "label">[],
): string | null {
  if (series.length === 0) {
    return null;
  }

  const parsed = series.map((item) => parseSeriesLabel(item.label));
  if (parsed.length === 1) {
    return parsed[0]?.original ?? null;
  }

  if (parsed.every((item) => item.original === parsed[0]?.original)) {
    return parsed[0]?.original ?? null;
  }

  const firstBase = parsed[0]?.base ?? null;
  if (!firstBase || parsed.some((item) => item.base !== firstBase)) {
    return null;
  }

  const parenValues = new Set(
    parsed
      .map((item) => item.paren)
      .filter((value): value is string => value !== null),
  );
  const bracketValues = new Set(
    parsed
      .map((item) => item.bracket)
      .filter((value): value is string => value !== null),
  );

  if (parenValues.size > 1 || bracketValues.size > 1) {
    return firstBase;
  }

  return null;
}

export function resolveXYYAxisLabel(
  data: Pick<
    Extract<ChartOptions, { type: "xy" }>,
    "plotMode" | "yName" | "series"
  >,
) {
  if (data.yName) {
    return data.yName;
  }
  if (data.plotMode === "quantity_vs_sweep") {
    return deriveSharedSeriesLabel(data.series) ?? "";
  }
  return "";
}

export function parseSeriesLabel(label: string): ParsedSeriesLabel {
  const match = /^(.*?)(?: \(([^()]+)\))?(?: \[([^\]]+)\])?$/.exec(label);
  return {
    original: label,
    base: match?.[1]?.trim() ?? label,
    paren: match?.[2] ?? null,
    bracket: match?.[3] ?? null,
  };
}
