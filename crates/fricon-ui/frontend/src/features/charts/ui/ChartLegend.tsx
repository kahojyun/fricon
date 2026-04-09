import type { ChartSeries } from "@/shared/lib/chartTypes";
import { SERIES_COLORS } from "../rendering/webgl";

interface ChartLegendProps {
  series: ChartSeries[];
}

export function ChartLegend({ series }: ChartLegendProps) {
  if (series.length <= 1) return null;

  return (
    <div className="absolute top-1 right-2 flex flex-wrap gap-x-3 gap-y-0.5 rounded bg-background/80 px-2 py-1 text-xs backdrop-blur-sm">
      {series.map((s, i) => (
        <div key={s.name} className="flex items-center gap-1">
          <span
            className="inline-block size-2.5 shrink-0 rounded-full"
            style={{
              backgroundColor: SERIES_COLORS[i % SERIES_COLORS.length],
            }}
          />
          <span className="text-foreground/80">{s.name}</span>
        </div>
      ))}
    </div>
  );
}
