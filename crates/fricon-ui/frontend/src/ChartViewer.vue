<script setup lang="ts">
import {
  ref,
  computed,
  onMounted,
  onUnmounted,
  watch,
  useTemplateRef,
} from "vue";
import * as echarts from "echarts";
import {
  getChartSchema,
  getChartData,
  subscribeLiveChartUpdates,
  onChartUpdate,
  type ChartSchemaResponse,
  type ColumnValue,
  type ChartDataRequest,
  type EChartsDataResponse,
  type ChartUpdate,
} from "./backend";

// Props
interface Props {
  datasetId?: number;
}

const props = withDefaults(defineProps<Props>(), {
  datasetId: undefined,
});

// Template refs
const chart = useTemplateRef("chart");

// State
const schema = ref<ChartSchemaResponse | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

// Live update state
const liveUpdatesEnabled = ref(false);
const isLiveDataset = ref(false);
const lastUpdateTime = ref<Date | null>(null);
const updateCount = ref(0);
let chartUpdateUnlisten: (() => void) | null = null;

// Configuration state
const selectedXColumn = ref<string>("");
const selectedYColumns = ref<string[]>([]);
const indexFilters = ref<Record<string, ColumnValue>>({});

// Chart instance
let chartInstance: echarts.ECharts | null = null;
const observer = new ResizeObserver(() => {
  requestAnimationFrame(() => {
    chartInstance?.resize();
  });
});

// Computed properties
const indexColumns = computed(() => {
  return schema.value?.columns.filter((col) => col.is_index_column) ?? [];
});

const numericColumns = computed(() => {
  return (
    schema.value?.columns.filter((col) => col.data_type === "Numeric") ?? []
  );
});

const availableYColumns = computed(() => {
  // Y columns should be numeric and not the selected X column
  return numericColumns.value.filter(
    (col) => col.name !== selectedXColumn.value,
  );
});

const canGenerateChart = computed(() => {
  return (
    schema.value &&
    selectedXColumn.value &&
    selectedYColumns.value.length > 0 &&
    props.datasetId !== undefined
  );
});

// Chart generation
const chartData = ref<EChartsDataResponse | null>(null);

async function loadSchema() {
  if (!props.datasetId) return;

  loading.value = true;
  error.value = null;

  try {
    schema.value = await getChartSchema(props.datasetId);

    // Auto-select first index column as X-axis if available
    if (indexColumns.value.length > 0) {
      selectedXColumn.value = indexColumns.value[0].name;
    }

    // Auto-select first numeric column as Y-axis if available
    if (numericColumns.value.length > 0) {
      const firstY = numericColumns.value.find(
        (col) => col.name !== selectedXColumn.value,
      );
      if (firstY) {
        selectedYColumns.value = [firstY.name];
      }
    }

    // Auto-enable live updates for new datasets (optional)
    // This could be made configurable via user preferences
    if (canGenerateChart.value) {
      setTimeout(() => {
        void enableLiveUpdates();
      }, 500); // Small delay to let chart generate first
    }
  } catch (err) {
    error.value = err instanceof Error ? err.message : "Failed to load schema";
  } finally {
    loading.value = false;
  }
}

async function generateChart() {
  if (!canGenerateChart.value || !props.datasetId) return;

  loading.value = true;
  error.value = null;

  try {
    const request: ChartDataRequest = {
      dataset_id: props.datasetId,
      x_column: selectedXColumn.value,
      y_columns: selectedYColumns.value,
      index_column_filters: Object.entries(indexFilters.value)
        .filter(
          ([key, value]) =>
            key !== selectedXColumn.value && value !== undefined,
        )
        .map(([column, value]) => ({ column, value })),
    };

    chartData.value = await getChartData(request);
    updateChart();
  } catch (err) {
    error.value =
      err instanceof Error ? err.message : "Failed to generate chart";
  } finally {
    loading.value = false;
  }
}

// Live update functions
async function enableLiveUpdates() {
  if (!props.datasetId || liveUpdatesEnabled.value) return;

  try {
    // Subscribe to live chart updates
    await subscribeLiveChartUpdates(props.datasetId);

    // Set up event listener
    chartUpdateUnlisten = await onChartUpdate(handleChartUpdate);

    liveUpdatesEnabled.value = true;
    isLiveDataset.value = true;

    console.log(`Live updates enabled for dataset ${props.datasetId}`);
  } catch (err) {
    console.warn("Failed to enable live updates:", err);
    error.value = "Failed to enable live updates";
  }
}

function disableLiveUpdates() {
  if (chartUpdateUnlisten) {
    chartUpdateUnlisten();
    chartUpdateUnlisten = null;
  }

  liveUpdatesEnabled.value = false;
  console.log(`Live updates disabled for dataset ${props.datasetId}`);
}

function handleChartUpdate(update: ChartUpdate) {
  // Only handle updates for our dataset
  if (update.dataset_id !== props.datasetId) return;

  updateCount.value += 1;
  lastUpdateTime.value = new Date(update.timestamp);

  console.log(`Chart update received:`, update.update_type);

  // Refresh chart data on updates
  if (canGenerateChart.value) {
    generateChart();
  }

  // Handle dataset completion
  if (update.update_type === "DatasetCompleted") {
    isLiveDataset.value = false;
    setTimeout(() => {
      disableLiveUpdates();
    }, 1000); // Give a moment for final updates
  }
}

function updateChart() {
  if (!chartInstance || !chartData.value) return;

  const option: echarts.EChartsOption = {
    animation: false,
    dataset: {
      dimensions: chartData.value.dataset.dimensions,
      source: chartData.value.dataset.source.map((row) =>
        row.map((val) => {
          if (val.type === "Number") return val.value;
          if (val.type === "String") return val.value;
          if (val.type === "Boolean") return val.value ? 1 : 0;
          return val;
        }),
      ),
    },
    xAxis: {
      type: "category",
      name: selectedXColumn.value,
    },
    yAxis: {
      type: "value",
    },
    series: chartData.value.series.map((series) => ({
      name: series.name,
      type: "line" as const,
      encode: {
        x: 0, // First dimension is X
        y: series.data_group_id, // Use the series data group ID
      },
    })),
    legend: {
      data: chartData.value.series.map((s) => s.name),
    },
    tooltip: {
      trigger: "axis" as const,
    },
  };

  chartInstance.setOption(option, true);
}

function cleanup() {
  observer.disconnect();
  chartInstance?.dispose();
  chartInstance = null;

  // Clean up live updates
  disableLiveUpdates();
}

function initChart() {
  const chartDiv = chart.value;
  if (!chartDiv) return;
  chartInstance = echarts.init(chartDiv);
  observer.observe(chartDiv);

  if (chartData.value) {
    updateChart();
  }
}

function addYColumn() {
  const available = availableYColumns.value.filter(
    (col) => !selectedYColumns.value.includes(col.name),
  );
  if (available.length > 0) {
    selectedYColumns.value.push(available[0].name);
  }
}

function removeYColumn(index: number) {
  selectedYColumns.value.splice(index, 1);
}

function getColumnValueString(value: ColumnValue): string {
  if (value.type === "Number") return value.value.toString();
  if (value.type === "String") return value.value;
  if (value.type === "Boolean") return value.value ? "true" : "false";
  return "";
}

function setIndexFilter(column: string, value: ColumnValue | undefined) {
  if (value === undefined) {
    delete indexFilters.value[column];
  } else {
    indexFilters.value[column] = value;
  }
}

// Watchers
watch(() => props.datasetId, loadSchema, { immediate: true });
watch(chart, () => {
  cleanup();
  initChart();
});
watch([selectedXColumn, selectedYColumns, indexFilters], generateChart, {
  deep: true,
});

onMounted(() => {
  if (props.datasetId) {
    void loadSchema();
  }
});

onUnmounted(cleanup);
</script>

<template>
  <div class="chart-viewer h-full flex flex-col">
    <!-- Configuration Panel -->
    <div class="config-panel bg-gray-50 p-4 border-b">
      <div class="flex justify-between items-center mb-4">
        <h3 class="text-lg font-medium">Chart Configuration</h3>

        <!-- Live Update Controls -->
        <div v-if="props.datasetId" class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <button
              v-if="!liveUpdatesEnabled"
              class="px-3 py-1 bg-green-500 text-white rounded text-sm hover:bg-green-600"
              @click="enableLiveUpdates"
            >
              📊 Enable Live Updates
            </button>
            <button
              v-else
              class="px-3 py-1 bg-orange-500 text-white rounded text-sm hover:bg-orange-600"
              @click="disableLiveUpdates"
            >
              ⏸️ Disable Live Updates
            </button>
          </div>

          <!-- Live Update Status -->
          <div v-if="liveUpdatesEnabled" class="text-sm">
            <div class="flex items-center gap-2">
              <div
                class="w-2 h-2 bg-green-500 rounded-full animate-pulse"
              ></div>
              <span class="text-green-700">Live</span>
              <span v-if="isLiveDataset" class="text-gray-600">
                ({{ updateCount }} updates)
              </span>
              <span v-else class="text-blue-600">(Dataset Complete)</span>
            </div>
            <div v-if="lastUpdateTime" class="text-xs text-gray-500">
              Last update: {{ lastUpdateTime.toLocaleTimeString() }}
            </div>
          </div>
        </div>
      </div>

      <div v-if="loading" class="text-blue-600">Loading schema...</div>

      <div v-else-if="error" class="text-red-600">Error: {{ error }}</div>

      <div
        v-else-if="schema"
        class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4"
      >
        <!-- X-axis selection -->
        <div>
          <label class="block text-sm font-medium mb-2"
            >X-axis (Index Column):</label
          >
          <select v-model="selectedXColumn" class="w-full p-2 border rounded">
            <option value="">Select X column...</option>
            <option
              v-for="col in indexColumns"
              :key="col.name"
              :value="col.name"
            >
              {{ col.name }}
            </option>
          </select>
        </div>

        <!-- Y-axis selection -->
        <div>
          <label class="block text-sm font-medium mb-2"
            >Y-axis (Numeric Columns):</label
          >
          <div class="space-y-2">
            <div
              v-for="(yCol, index) in selectedYColumns"
              :key="index"
              class="flex items-center gap-2"
            >
              <select
                v-model="selectedYColumns[index]"
                class="flex-1 p-2 border rounded"
              >
                <option
                  v-for="col in availableYColumns"
                  :key="col.name"
                  :value="col.name"
                >
                  {{ col.name }}
                </option>
              </select>
              <button
                class="px-2 py-1 bg-red-500 text-white rounded text-sm"
                @click="removeYColumn(index)"
              >
                ×
              </button>
            </div>
            <button
              class="px-3 py-1 bg-blue-500 text-white rounded text-sm"
              @click="addYColumn"
            >
              + Add Y Column
            </button>
          </div>
        </div>

        <!-- Index filters -->
        <div v-if="indexColumns.length > 1">
          <label class="block text-sm font-medium mb-2">Index Filters:</label>
          <div class="space-y-2">
            <div v-for="col in indexColumns" :key="col.name">
              <div v-if="col.name !== selectedXColumn" class="">
                <label class="block text-xs text-gray-600 mb-1"
                  >{{ col.name }}:</label
                >
                <select
                  :value="
                    indexFilters[col.name]
                      ? getColumnValueString(indexFilters[col.name])
                      : ''
                  "
                  class="w-full p-1 border rounded text-sm"
                  @change="
                    (e) => {
                      const target = e.target as HTMLSelectElement;
                      if (target.value === '') {
                        setIndexFilter(col.name, undefined);
                      } else if (col.unique_values) {
                        const selected = col.unique_values.find(
                          (v) => getColumnValueString(v) === target.value,
                        );
                        setIndexFilter(col.name, selected);
                      }
                    }
                  "
                >
                  <option value="">All values</option>
                  <option
                    v-for="value in col.unique_values"
                    :key="getColumnValueString(value)"
                    :value="getColumnValueString(value)"
                  >
                    {{ getColumnValueString(value) }}
                  </option>
                </select>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="!props.datasetId" class="text-gray-500">
        No dataset selected
      </div>
    </div>

    <!-- Chart Display -->
    <div class="chart-container flex-1 p-4">
      <div
        v-if="loading"
        class="h-full flex items-center justify-center text-blue-600"
      >
        Generating chart...
      </div>

      <div
        v-else-if="error"
        class="h-full flex items-center justify-center text-red-600"
      >
        {{ error }}
      </div>

      <div
        v-else-if="!canGenerateChart"
        class="h-full flex items-center justify-center text-gray-500"
      >
        Select X and Y columns to generate chart
      </div>

      <div v-else ref="chart" class="w-full h-full"></div>
    </div>
  </div>
</template>

<style scoped>
.chart-viewer {
  min-height: 600px;
}

.config-panel {
  min-height: 200px;
}

.chart-container {
  min-height: 400px;
}
</style>
