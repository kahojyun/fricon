import { useEffect, useMemo, useState } from "react";
import ReactEChartsCore from "echarts-for-react/lib/core";
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
import type { ChartOptions } from "@/lib/chartTypes";

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
}

function buildOption(data?: ChartOptions): EChartsOption {
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
      tooltip: { trigger: "item" },
      series: seriesOption,
    };
  }

  if (data.type === "scatter") {
    const { xName, yName, series } = data;
    const seriesOption = series.map(
      (s): ScatterSeriesOption => ({
        name: s.name,
        type: "scatter",
        data: s.data,
        symbolSize: 6,
      }),
    );
    return {
      animation: false,
      xAxis: { type: "value", name: xName },
      yAxis: { type: "value", name: yName },
      legend: {},
      tooltip: { trigger: "item" },
      series: seriesOption,
    };
  }

  const { xName, series } = data;
  const seriesOption = series.map(
    (s): LineSeriesOption => ({
      name: s.name,
      type: "line",
      data: s.data,
    }),
  );
  return {
    animation: false,
    xAxis: { type: "value", name: xName },
    yAxis: { type: "value" },
    legend: {},
    tooltip: { trigger: "axis" },
    series: seriesOption,
  };
}

export function ChartWrapper({ data }: ChartWrapperProps) {
  const prefersDark = useMemo(() => {
    if (typeof window === "undefined") return false;
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }, []);
  const [theme, setTheme] = useState<"dark" | "default">(() => {
    if (typeof document === "undefined") return "default";
    return document.documentElement.classList.contains("dark") || prefersDark
      ? "dark"
      : "default";
  });

  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const updateTheme = () => {
      const next =
        document.documentElement.classList.contains("dark") || media.matches
          ? "dark"
          : "default";
      setTheme(next);
    };
    updateTheme();
    const handler = () => updateTheme();
    media.addEventListener("change", handler);
    const observer = new MutationObserver(updateTheme);
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });
    return () => {
      media.removeEventListener("change", handler);
      observer.disconnect();
    };
  }, []);

  const option = useMemo(() => buildOption(data), [data]);

  return (
    <div className="relative size-full">
      <ReactEChartsCore
        echarts={echarts}
        style={{ width: "100%", height: "100%" }}
        option={option}
        notMerge
        lazyUpdate
        theme={theme}
      />
      {!data ? (
        <div className="text-muted-foreground absolute inset-0 flex items-center justify-center text-sm">
          No chart data
        </div>
      ) : null}
    </div>
  );
}
