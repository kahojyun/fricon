<script setup lang="ts">
import * as echarts from "echarts/core";
import {
  DatasetComponent,
  type DatasetComponentOption,
  GridComponent,
  type GridComponentOption,
  LegendComponent,
  type LegendComponentOption,
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
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import { useDark } from "@vueuse/core";
import { vResizeObserver } from "@vueuse/components";
import type { TypedArray } from "apache-arrow/interfaces";
import {
  VisualMapComponent,
  type VisualMapComponentOption,
} from "echarts/components";

echarts.use([
  DatasetComponent,
  GridComponent,
  LegendComponent,
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
  | VisualMapComponentOption
  | LineSeriesOption
  | HeatmapSeriesOption
  | ScatterSeriesOption
>;

export type ChartSeriesData = number[] | TypedArray | [number, number][];

export interface ChartSeries {
  name: string;
  data: ChartSeriesData;
}

export type ChartOptions =
  | {
      type: "line";
      x: number[] | TypedArray;
      xName: string;
      series: ChartSeries[];
    }
  | {
      type: "heatmap";
      x: number[] | TypedArray;
      xName: string;
      y: number[] | TypedArray;
      yName: string;
      series: ChartSeries[];
    }
  | {
      type: "scatter";
      xName: string;
      yName: string;
      series: ChartSeries[];
    };

const { data = undefined } = defineProps<{
  data?: ChartOptions;
}>();
const chartDiv = useTemplateRef("chartDiv");
const isDark = useDark();
const colorTheme = computed(() => (isDark.value ? "dark" : "default"));
let instance: echarts.ECharts | null = null;

function makeOption(data?: ChartOptions): EChartsOption {
  if (!data) {
    return {};
  }
  const { type, series } = data;
  let source: Record<string, number[] | TypedArray>;

  if (type === "heatmap") {
    const { x, xName, y, yName } = data;
    if (!y || !yName) {
      console.warn("Heatmap requires y axis data");
      return {};
    }
    source = {
      [xName]: x,
      [yName]: y,
      ...Object.fromEntries(
        series.map((series) => [
          series.name,
          series.data as number[] | TypedArray,
        ]),
      ),
    };

    // Calculate min/max for visual map
    let min = Infinity;
    let max = -Infinity;
    for (const s of series) {
      // Basic min/max - can be optimized
      for (const v of s.data as number[] | TypedArray) {
        if (v < min) min = v;
        if (v > max) max = v;
      }
    }
    if (!isFinite(min)) min = 0;
    if (!isFinite(max)) max = 1;

    const seriesOption = series.map(
      (series): HeatmapSeriesOption => ({
        name: series.name,
        type: "heatmap",
        encode: { x: xName, y: yName, value: series.name },
        progressive: 5000,
      }),
    );
    return {
      dataset: { source },
      animation: false,
      xAxis: { type: "category", name: xName },
      yAxis: { type: "category", name: yName },
      visualMap: {
        min,
        max,
        dimension: 2, // Index 2 is the first series value (x: 0, y: 1, series: 2)
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
      grid: { right: "18%" }, // Make room for visualMap
      tooltip: {
        position: "top",
      },
      series: seriesOption,
    };
  }

  if (type === "scatter") {
    const { xName, yName } = data;
    const seriesOption = series.map(
      (series): ScatterSeriesOption => ({
        name: series.name,
        type: "scatter",
        data: series.data as [number, number][],
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

  const { x, xName } = data;
  source = {
    [xName]: x,
    ...Object.fromEntries(
      series.map((series) => [
        series.name,
        series.data as number[] | TypedArray,
      ]),
    ),
  };
  // Line chart
  const seriesOption = series.map(
    (series): LineSeriesOption => ({
      name: series.name,
      type: "line",
      encode: { x: xName, y: series.name },
    }),
  );
  return {
    dataset: { source },
    animation: false,
    xAxis: { type: "value", name: xName }, // Line chart usually value axis
    yAxis: { type: "value" },
    legend: {},
    tooltip: { trigger: "axis" },
    series: seriesOption,
  };
}

onMounted(() => {
  instance = echarts.init(chartDiv.value, colorTheme.value);
  instance.setOption(makeOption(data), { notMerge: true });
});
onUnmounted(() => {
  instance?.dispose();
  instance = null;
});
watch(colorTheme, (value) => {
  instance?.setOption(makeOption(data), { notMerge: true });
  instance?.setTheme(value);
});
watch(
  () => data,
  (data) => {
    instance?.setOption(makeOption(data), { notMerge: true });
  },
);
function resize() {
  instance?.resize();
}
</script>

<template>
  <div ref="chartDiv" v-resize-observer="resize" class="size-full"></div>
</template>
