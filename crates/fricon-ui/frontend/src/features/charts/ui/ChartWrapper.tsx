import { useTheme } from "next-themes";
import type { ChartOptions, ChartSeries } from "@/shared/lib/chartTypes";
import { useWebGLChart } from "../hooks/useWebGLChart";
import { ChartFrameHeader } from "./ChartFrameHeader";
import { ChartLegend } from "./ChartLegend";
import { ChartTooltip } from "./ChartTooltip";
import type { ChartFrameHeaderData } from "./chartFrameHeaderModel";
import { deriveLegendPresentation } from "./chartLegendPresentation";

interface ChartWrapperProps {
  data?: ChartOptions;
  interactionKey?: string | null;
  liveMode?: boolean;
  header?: ChartFrameHeaderData | null;
}

export function ChartWrapper({
  data,
  interactionKey,
  liveMode,
  header,
}: ChartWrapperProps) {
  const { resolvedTheme } = useTheme();
  const visibleLegendSeries =
    data && data.type !== "heatmap"
      ? liveMode
        ? currentLiveLegendSeries(data.series)
        : data.series
      : [];
  const legendPresentation = deriveLegendPresentation(visibleLegendSeries);
  const showLegend = legendPresentation.items.length > 1;
  const { canvasRef, svgRef, getInteractionState } = useWebGLChart({
    data,
    interactionKey,
    liveMode,
    theme: resolvedTheme,
  });

  return (
    <div className="relative size-full overflow-hidden">
      <canvas
        ref={canvasRef}
        className="absolute inset-0 size-full"
        style={{ display: "block" }}
      />
      <svg
        ref={svgRef}
        className="absolute inset-0 size-full"
        style={{ pointerEvents: "auto" }}
      />
      {/* Live mode intentionally omits hover tooltips to avoid noisy multi-sweep
          overlays; users rely on the chart readout/crosshair instead. */}
      {!liveMode && data ? (
        <ChartTooltip
          data={data}
          svgRef={svgRef}
          getInteractionState={getInteractionState}
        />
      ) : null}
      {header ? (
        <div className="pointer-events-none absolute inset-x-0 top-0 z-10 flex justify-center p-2">
          <ChartFrameHeader header={header} />
        </div>
      ) : null}
      {showLegend ? (
        <div className="pointer-events-none absolute top-2 right-2 z-10 max-w-[40%]">
          <ChartLegend items={legendPresentation.items} />
        </div>
      ) : null}
      {!data ? (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-muted-foreground">
          No chart data
        </div>
      ) : null}
    </div>
  );
}

function currentLiveLegendSeries(series: ChartSeries[]): ChartSeries[] {
  const currentGroup = liveSeriesGroupId(series.at(-1)?.id);
  if (!currentGroup) {
    return series;
  }

  return series.filter((item) => liveSeriesGroupId(item.id) === currentGroup);
}

function liveSeriesGroupId(id: string | undefined): string | null {
  if (!id) return null;
  const match = /^(row:\d+|group:\d+)(?::|$)/.exec(id);
  return match?.[1] ?? null;
}
