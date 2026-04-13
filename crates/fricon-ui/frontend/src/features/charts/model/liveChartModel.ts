import type {
  ChartModel,
  ChartSeries,
  HeatmapSeries,
} from "@/shared/lib/chartTypes";
import type { LiveChartUpdate } from "../api/types";

export interface LiveChartState {
  rowCount: number;
  chart: ChartModel;
}

export function applyLiveChartUpdate(
  previous: LiveChartState | null,
  update: LiveChartUpdate,
): LiveChartState | null {
  if (update.mode === "reset") {
    return {
      rowCount: update.rowCount,
      chart: update.snapshot,
    };
  }

  if (!previous) {
    return null;
  }

  const nextChart = appendToChart(previous.chart, update);
  if (!nextChart) {
    return null;
  }

  return {
    rowCount: update.rowCount,
    chart: nextChart,
  };
}

export function getLiveSeriesLabel(
  label: string,
  index: number,
  total: number,
): string {
  if (total <= 1) return label;
  if (index === total - 1) return "current";
  return `-${total - 1 - index}`;
}

function appendToChart(
  chart: ChartModel,
  update: Extract<LiveChartUpdate, { mode: "append" }>,
): ChartModel | null {
  if (chart.type === "heatmap") {
    return appendToHeatmap(chart, update);
  }

  const series = chart.series.map((item) => ({ ...item }));
  for (const operation of update.ops) {
    if (operation.kind === "append_points") {
      const target = series.find((item) => item.id === operation.seriesId);
      if (!target) return null;
      target.values = concatFloat64(target.values, operation.values);
      target.pointCount += operation.pointCount;
      continue;
    }

    if (operation.series.shape !== "xy") {
      return null;
    }

    series.push(operation.series.series);
  }

  return { ...chart, series };
}

function appendToHeatmap(
  chart: Extract<ChartModel, { type: "heatmap" }>,
  update: Extract<LiveChartUpdate, { mode: "append" }>,
): ChartModel | null {
  const series = chart.series.map((item) => ({ ...item }));

  for (const operation of update.ops) {
    if (operation.kind === "append_points") {
      const target = series.find((item) => item.id === operation.seriesId);
      if (!target) return null;
      target.values = concatFloat64(target.values, operation.values);
      target.pointCount += operation.pointCount;
      continue;
    }

    if (operation.series.shape !== "xyz") {
      return null;
    }

    series.push(operation.series.series);
  }

  return {
    ...chart,
    series,
  };
}

function concatFloat64(current: Float64Array, appended: Float64Array) {
  const next = new Float64Array(current.length + appended.length);
  next.set(current, 0);
  next.set(appended, current.length);
  return next;
}

export function cloneChartSeries(series: ChartSeries): ChartSeries {
  return {
    ...series,
    values: new Float64Array(series.values),
  };
}

export function cloneHeatmapSeries(series: HeatmapSeries): HeatmapSeries {
  return {
    ...series,
    values: new Float64Array(series.values),
  };
}

export function liveSeriesGroupId(id: string | undefined): string | null {
  if (!id) return null;
  const match = /^(row:\d+|group:\d+)(?::|$)/.exec(id);
  return match?.[1] ?? null;
}
