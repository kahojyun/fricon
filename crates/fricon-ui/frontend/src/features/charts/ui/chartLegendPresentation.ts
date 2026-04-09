import type { ChartSeries } from "@/shared/lib/chartTypes";
import { parseSeriesLabel } from "../model/seriesLabeling";

export interface LegendPresentation {
  items: { id: string; text: string }[];
}

export function deriveLegendPresentation(
  series: ChartSeries[],
): LegendPresentation {
  const parsed = series.map((item) => ({
    id: item.id,
    ...parseLegendLabel(item.label),
  }));
  const firstBase = parsed[0]?.base ?? null;

  if (!firstBase || parsed.some((item) => item.base !== firstBase)) {
    return {
      items: parsed.map((item) => ({ id: item.id, text: item.original })),
    };
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

  const items = parsed.map((item) => {
    const parts: string[] = [];

    if (parenValues.size > 1 && item.paren) {
      parts.push(item.paren);
    }
    if (bracketValues.size > 1 && item.bracket) {
      parts.push(`[${item.bracket}]`);
    }

    return {
      id: item.id,
      text: parts.join(" ") || item.original,
    };
  });

  return { items };
}

function parseLegendLabel(label: string) {
  return parseSeriesLabel(label);
}
