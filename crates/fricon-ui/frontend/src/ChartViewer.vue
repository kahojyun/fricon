<script setup lang="ts">
import { useTemplateRef, onUnmounted, watch } from "vue";
import * as echarts from "echarts";
import { useDark } from "@vueuse/core";

const props = defineProps<{
  datasetId: number | null;
}>();
const isDark = useDark();
const chart = useTemplateRef("chart");
let chartInstance: echarts.ECharts | null = null;
const observer = new ResizeObserver(() => {
  requestAnimationFrame(() => {
    chartInstance?.resize();
  });
});

const option = {
  animation: false,
  xAxis: {
    type: "category",
    data: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
  },
  yAxis: {
    type: "value",
  },
  series: [
    {
      data: [150, 230, 224, 218, 135, 147, 260],
      type: "line",
    },
  ],
};

function cleanup() {
  observer.disconnect();
  chartInstance?.dispose();
  chartInstance = null;
}

function initChart() {
  const chartDiv = chart.value;
  if (!chartDiv) return;
  chartInstance = echarts.init(chartDiv);
  chartInstance.setOption(option);
  observer.observe(chartDiv);
  watch(
    isDark,
    () => {
      chartInstance?.setTheme(isDark.value ? "dark" : "default");
    },
    { immediate: true },
  );
}

watch(chart, () => {
  cleanup();
  initChart();
});
watch(
  () => props.datasetId,
  () => console.log(`log from child: ${props.datasetId}`),
);

onUnmounted(cleanup);
</script>

<template>
  <div ref="chart" class="w-full h-full"></div>
</template>
