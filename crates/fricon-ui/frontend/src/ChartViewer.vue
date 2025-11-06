<script setup lang="ts">
import { useTemplateRef, onUnmounted, watch, ref } from "vue";
import * as echarts from "echarts";
import { useDark } from "@vueuse/core";
import { type DatasetDetail, datasetDetail, fetchData } from "@/backend.ts";
import type { Table } from "apache-arrow";
import { DataTable, Column, Splitter, SplitterPanel } from "primevue";

const props = defineProps<{
  datasetId: number | null;
}>();
const isDark = useDark();
const chart = useTemplateRef("chart");
const detail = ref<DatasetDetail | null>(null);
const indexTable = ref<Table | null>(null);
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
  async () => {
    const datasetId = props.datasetId;
    if (datasetId != null) {
      detail.value = await datasetDetail(datasetId);
      const index_columns = detail.value.index;
      if (index_columns != null) {
        indexTable.value = await fetchData(datasetId, {
          columns: index_columns,
        });
      }
    }
  },
);

onUnmounted(cleanup);
</script>

<template>
  <Splitter class="w-full h-full" layout="vertical">
    <SplitterPanel>
      <div ref="chart" class="w-full h-full"></div>
    </SplitterPanel>
    <SplitterPanel>
      <DataTable
        size="small"
        :value="indexTable?.toArray()"
        scrollable
        scroll-height="flex"
      >
        <Column
          v-for="col in indexTable?.schema.fields"
          :key="col.name"
          :field="col.name"
          :header="col.name"
        />
      </DataTable>
    </SplitterPanel>
  </Splitter>
</template>
