import { useEffect, useMemo, useRef } from "react";
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
    const { xName, yName, series } = data;
    const xCategories: number[] = [];
    const yCategories: number[] = [];
    const xSet = new Set<number>();
    const ySet = new Set<number>();
    for (const s of series) {
      for (const point of s.data) {
        const xValue = point[0];
        const yValue = point[1];
        if (xValue !== undefined && !xSet.has(xValue)) {
          xSet.add(xValue);
          xCategories.push(xValue);
        }
        if (yValue !== undefined && !ySet.has(yValue)) {
          ySet.add(yValue);
          yCategories.push(yValue);
        }
      }
    }

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
  const chartRef = useRef<HTMLDivElement | null>(null);
  const instanceRef = useRef<echarts.ECharts | null>(null);
  const prefersDark = useMemo(() => {
    if (typeof window === "undefined") return false;
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
  }, []);

  useEffect(() => {
    if (!chartRef.current) return;
    const theme = document.documentElement.classList.contains("dark")
      ? "dark"
      : prefersDark
        ? "dark"
        : "default";
    const instance = echarts.init(chartRef.current, theme);
    instanceRef.current = instance;

    const resizeObserver = new ResizeObserver(() => {
      instance.resize();
    });
    resizeObserver.observe(chartRef.current);

    return () => {
      resizeObserver.disconnect();
      instance.dispose();
      instanceRef.current = null;
    };
  }, [prefersDark]);

  useEffect(() => {
    const instance = instanceRef.current;
    if (!instance) return;
    instance.setOption(buildOption(data), { notMerge: true });
  }, [data]);

  return (
    <div className="relative size-full">
      <div ref={chartRef} className="size-full" />
      {!data ? (
        <div className="text-muted-foreground absolute inset-0 flex items-center justify-center text-sm">
          No chart data
        </div>
      ) : null}
    </div>
  );
}
