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
import { LineChart, type LineSeriesOption } from "echarts/charts";
import { CanvasRenderer } from "echarts/renderers";
import { computed, onMounted, onUnmounted, useTemplateRef, watch } from "vue";
import { useDark } from "@vueuse/core";
import { vResizeObserver } from "@vueuse/components";
import type { TypedArray } from "apache-arrow/interfaces";

echarts.use([
  DatasetComponent,
  GridComponent,
  LegendComponent,
  LineChart,
  CanvasRenderer,
]);

type EChartsOption = echarts.ComposeOption<
  | DatasetComponentOption
  | GridComponentOption
  | LegendComponentOption
  | LineSeriesOption
>;

export interface LineSeries {
  name: string;
  data: number[] | TypedArray;
}

export interface LinePlotOptions {
  x: number[] | TypedArray;
  xName: string;
  series: LineSeries[];
}

const { data = undefined } = defineProps<{
  data?: LinePlotOptions;
}>();
const chartDiv = useTemplateRef("chartDiv");
const isDark = useDark();
const colorTheme = computed(() => (isDark.value ? "dark" : "default"));
let instance: echarts.ECharts | null = null;

function makeOption(data?: LinePlotOptions): EChartsOption {
  if (!data) {
    return {};
  }
  const { x, xName, series } = data;
  const source = {
    [xName]: x,
    ...Object.fromEntries(series.map((series) => [series.name, series.data])),
  };
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
    xAxis: { name: xName },
    yAxis: {},
    legend: {},
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
