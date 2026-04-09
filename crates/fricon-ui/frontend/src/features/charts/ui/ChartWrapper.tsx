import { useTheme } from "next-themes";
import type { ChartOptions } from "@/shared/lib/chartTypes";
import { useWebGLChart } from "../hooks/useWebGLChart";
import { ChartLegend } from "./ChartLegend";
import { ChartTooltip } from "./ChartTooltip";

interface ChartWrapperProps {
  data?: ChartOptions;
  interactionKey?: string | null;
  liveMode?: boolean;
}

export function ChartWrapper({
  data,
  interactionKey,
  liveMode,
}: ChartWrapperProps) {
  const { resolvedTheme } = useTheme();
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
      {!liveMode && data ? (
        <ChartTooltip
          data={data}
          svgRef={svgRef}
          getInteractionState={getInteractionState}
        />
      ) : null}
      {!liveMode && data && data.type !== "heatmap" ? (
        <ChartLegend series={data.series} />
      ) : null}
      {!data ? (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-muted-foreground">
          No chart data
        </div>
      ) : null}
    </div>
  );
}
