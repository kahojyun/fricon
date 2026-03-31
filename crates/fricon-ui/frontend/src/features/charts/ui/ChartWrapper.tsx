import ReactEChartsCore from "echarts-for-react/esm/core";
import * as echarts from "echarts/core";
import {
  DatasetComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  VisualMapComponent,
  type DatasetComponentOption,
  type GridComponentOption,
  type LegendComponentOption,
  type TooltipComponentOption,
  type VisualMapComponentOption,
} from "echarts/components";
import {
  HeatmapChart,
  LineChart,
  ScatterChart,
  type HeatmapSeriesOption,
  type LineSeriesOption,
  type ScatterSeriesOption,
} from "echarts/charts";
import { CanvasRenderer } from "echarts/renderers";
import { useTheme } from "next-themes";
import type { ChartOptions } from "@/shared/lib/chartTypes";

echarts.use([
  DatasetComponent,
  GridComponent,
  LegendComponent,
  TooltipComponent,
  VisualMapComponent,
  LineChart,
  HeatmapChart,
  ScatterChart,
  CanvasRenderer,
]);

type EChartsOption = echarts.ComposeOption<
  | DatasetComponentOption
  | GridComponentOption
  | LegendComponentOption
  | TooltipComponentOption
  | VisualMapComponentOption
  | LineSeriesOption
  | HeatmapSeriesOption
  | ScatterSeriesOption
>;

interface ChartWrapperProps {
  data?: ChartOptions;
  liveMode?: boolean;
}

function buildOption(data?: ChartOptions, liveMode?: boolean): EChartsOption {
  if (!data) return {};

  if (data.type === "heatmap") {
    const { xName, yName, xCategories, yCategories, series } = data;

    const seriesOption = series.map(
      (s): HeatmapSeriesOption => ({
        name: s.name,
        type: "heatmap",
        data: s.data,
        progressive: 5000,
      }),
    );

    let min = Infinity;
    let max = -Infinity;
    for (const s of series) {
      for (const v of s.data) {
        const value = v[2];
        if (value === undefined) continue;
        if (value < min) min = value;
        if (value > max) max = value;
      }
    }
    if (!isFinite(min)) min = 0;
    if (!isFinite(max)) max = 1;

    return {
      animation: false,
      xAxis: { type: "category", name: xName, data: xCategories },
      yAxis: { type: "category", name: yName, data: yCategories },
      visualMap: {
        min,
        max,
        calculable: true,
        orient: "vertical",
        right: 12,
        top: "middle",
        itemWidth: 14,
        itemHeight: 120,
        inRange: {
          color: ["#2c7bb6", "#abd9e9", "#ffffbf", "#fdae61", "#d7191c"],
        },
      },
      grid: { right: "18%" },
      tooltip: liveMode ? { show: false } : { trigger: "item" },
      series: seriesOption,
    };
  }

  if (data.type === "scatter") {
    const { xName, yName, series } = data;
    const seriesOption = series.map((s, index): ScatterSeriesOption => {
      if (liveMode && series.length > 1) {
        const total = series.length;
        const isNewest = index === total - 1;
        const opacity = isNewest
          ? 1.0
          : 0.12 + (0.5 * index) / Math.max(total - 2, 1);
        return {
          name: s.name,
          type: "scatter",
          data: s.data,
          symbolSize: isNewest ? 6 : 4,
          color: isNewest ? "#2563eb" : "#94a3b8",
          itemStyle: { opacity },
        };
      }
      return {
        name: s.name,
        type: "scatter",
        data: s.data,
        symbolSize: 6,
      };
    });
    return {
      animation: false,
      xAxis: { type: "value", name: xName },
      yAxis: { type: "value", name: yName },
      legend: liveMode ? { show: false } : {},
      tooltip: liveMode ? { show: false } : { trigger: "item" },
      series: seriesOption,
    };
  }

  const { xName, series } = data;
  const seriesOption = series.map((s, index): LineSeriesOption => {
    if (liveMode && series.length > 1) {
      const total = series.length;
      const isNewest = index === total - 1;
      const opacity = isNewest
        ? 1.0
        : 0.12 + (0.5 * index) / Math.max(total - 2, 1);
      return {
        name: s.name,
        type: "line",
        data: s.data,
        color: isNewest ? "#2563eb" : "#94a3b8",
        lineStyle: {
          width: isNewest ? 2.5 : 1.5,
          opacity,
        },
        itemStyle: { opacity: 0 },
        showSymbol: false,
      };
    }
    return {
      name: s.name,
      type: "line",
      data: s.data,
    };
  });
  return {
    animation: false,
    xAxis: { type: "value", name: xName },
    yAxis: { type: "value" },
    legend: liveMode ? { show: false } : {},
    tooltip: liveMode ? { show: false } : { trigger: "axis" },
    series: seriesOption,
  };
}

export function ChartWrapper({ data, liveMode }: ChartWrapperProps) {
  const { resolvedTheme } = useTheme();
  const theme = resolvedTheme === "dark" ? "dark" : "default";

  return (
    <div className="relative size-full">
      <ReactEChartsCore
        echarts={echarts}
        style={{ width: "100%", height: "100%" }}
        option={buildOption(data, liveMode)}
        notMerge
        lazyUpdate
        theme={theme}
      />
      {!data ? (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-muted-foreground">
          No chart data
        </div>
      ) : null}
    </div>
  );
}
