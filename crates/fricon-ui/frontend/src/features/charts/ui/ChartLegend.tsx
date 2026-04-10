import { SERIES_COLORS } from "../rendering/webgl";

interface ChartLegendProps {
  items: { id: string; text: string }[];
}

export function ChartLegend({ items }: ChartLegendProps) {
  if (items.length <= 1) return null;

  return (
    <div className="rounded-lg border border-border/60 bg-background/88 px-2 py-1 text-xs shadow-sm backdrop-blur-sm">
      <div className="flex flex-wrap gap-x-3 gap-y-0.5">
        {items.map((item, i) => (
          <div key={item.id} className="flex items-center gap-1">
            <span
              className="inline-block size-2.5 shrink-0 rounded-full"
              style={{
                backgroundColor: SERIES_COLORS[i % SERIES_COLORS.length],
              }}
            />
            <span className="text-foreground/80">{item.text}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
