<script setup lang="ts">
import * as echarts from "echarts/core";
import {
  DatasetComponent,
  type DatasetComponentOption,
  GridComponent,
  type GridComponentOption,
  LegendComponent,
  type LegendComponentOption,
  TooltipComponent,
  type TooltipComponentOption,
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
import type { ChartOptions } from "@/types/chart";
import {
  VisualMapComponent,
  type VisualMapComponentOption,
} from "echarts/components";

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

const { data = undefined } = defineProps<{ data?: ChartOptions }>();
const chartDiv = useTemplateRef("chartDiv");
const isDark = useDark();
const colorTheme = computed(() => (isDark.value ? "dark" : "default"));
let instance: echarts.ECharts | null = null;

function makeOption(data?: ChartOptions): EChartsOption {
  if (!data) {
    return {};
  }
  const { type, series } = data;
  if (type === "heatmap") {
    const { xName, yName } = data;
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
      (series): HeatmapSeriesOption => ({
        name: series.name,
        type: "heatmap",
        data: series.data,
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
      tooltip: {
        trigger: "item",
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
        data: series.data,
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

  const { xName } = data;
  const seriesOption = series.map(
    (series): LineSeriesOption => ({
      name: series.name,
      type: "line",
      data: series.data,
    }),
  );
  return {
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
