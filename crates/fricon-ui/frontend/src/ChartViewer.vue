<script setup lang="ts">
import {
  computed,
  onWatcherCleanup,
  ref,
  shallowRef,
  watch,
  type Ref,
} from "vue";
import {
  type ColumnInfo,
  type DatasetDetail,
  type FilterTableData,
  type FilterTableRow,
  getDatasetDetail,
  getFilterTableData,
  fetchChartData,
  getDatasetWriteStatus,
} from "@/backend.ts";
import { watchDebounced, watchThrottled } from "@vueuse/core";
import ChartWrapper from "./components/ChartWrapper.vue";
import FilterTable from "./components/FilterTable.vue";
import Checkbox from "primevue/checkbox";
import RadioButton from "primevue/radiobutton";
import Select from "primevue/select";
import Splitter from "primevue/splitter";
import SplitterPanel from "primevue/splitterpanel";
import type {
  ChartOptions,
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/types/chart";

const props = defineProps<{
  datasetId: number;
}>();

// ============================================================================
// Dataset and Filter State
// ============================================================================

const datasetDetail = shallowRef<DatasetDetail>();
const filterTableData = shallowRef<FilterTableData>();
const excludeColumns = ref<string[]>([]);
const datasetUpdateTick = ref(0);

// ============================================================================
// Chart Configuration
// ============================================================================

const chartType = ref<ChartType>("line");
const availableChartTypes = computed(() => {
  const columns = datasetDetail.value?.columns ?? [];
  if (columns.length === 0) return [];
  const hasSeries = columns.some((column) => !column.isIndex);
  const hasIndex = columns.some((column) => column.isIndex);
  const hasComplex = columns.some(
    (column) => !column.isIndex && column.isComplex,
  );
  const realColumns = columns.filter(
    (column) => !column.isIndex && !column.isComplex && !column.isTrace,
  );
  const realTraceColumns = columns.filter(
    (column) => !column.isIndex && !column.isComplex && column.isTrace,
  );
  const canScatter =
    hasComplex || realColumns.length >= 2 || realTraceColumns.length >= 2;
  const types: ChartType[] = [];
  if (hasSeries) types.push("line");
  if (hasSeries && hasIndex) types.push("heatmap");
  if (canScatter) types.push("scatter");
  return types;
});

const complexSeriesOptions: ComplexViewOption[] = [
  "real",
  "imag",
  "mag",
  "arg",
];
const selectedComplexView = ref<ComplexViewOption[]>(["real", "imag"]);
const selectedComplexViewSingle = ref<ComplexViewOption>("mag");

// ============================================================================
// Column Selection
// ============================================================================

const seriesOptions = computed(
  () => datasetDetail.value?.columns.filter((c) => !c.isIndex) ?? [],
);
const series = ref<ColumnInfo>();
watch(seriesOptions, updateSelectionFn(series));

const xColumnOptions = computed(() =>
  series.value?.isTrace
    ? []
    : (datasetDetail.value?.columns.filter((c) => c.isIndex) ?? []),
);
const xColumn = ref<ColumnInfo>();
watch(xColumnOptions, updateSelectionFn(xColumn));

const yColumnOptions = computed(
  () => datasetDetail.value?.columns.filter((c) => c.isIndex) ?? [],
);
const yColumn = ref<ColumnInfo>();
watch(yColumnOptions, updateSelectionFn(yColumn, 1));

const scatterMode = ref<ScatterMode>("complex");
const scatterComplexOptions = computed(
  () =>
    datasetDetail.value?.columns.filter(
      (column) => !column.isIndex && column.isComplex,
    ) ?? [],
);
const scatterTraceXYOptions = computed(
  () =>
    datasetDetail.value?.columns.filter(
      (column) => !column.isIndex && !column.isComplex && column.isTrace,
    ) ?? [],
);
const scatterXYOptions = computed(
  () =>
    datasetDetail.value?.columns.filter(
      (column) => !column.isIndex && !column.isComplex && !column.isTrace,
    ) ?? [],
);
const scatterSeries = ref<ColumnInfo>();
watch(scatterComplexOptions, updateSelectionFn(scatterSeries));
const scatterTraceXColumn = ref<ColumnInfo>();
watch(scatterTraceXYOptions, updateSelectionFn(scatterTraceXColumn));
const scatterTraceYColumn = ref<ColumnInfo>();
watch(scatterTraceXYOptions, updateSelectionFn(scatterTraceYColumn, 1));
const scatterXColumn = ref<ColumnInfo>();
watch(scatterXYOptions, updateSelectionFn(scatterXColumn));
const scatterYColumn = ref<ColumnInfo>();
watch(scatterXYOptions, updateSelectionFn(scatterYColumn, 1));

const scatterIsTraceBased = computed(() => {
  if (scatterMode.value === "trace_xy") {
    return true;
  }
  return scatterMode.value === "complex" && scatterSeries.value?.isTrace;
});
const scatterBinColumnOptions = computed(() => {
  const columns = datasetDetail.value?.columns ?? [];
  const excludedNames = new Set(
    [
      scatterSeries.value?.name,
      scatterXColumn.value?.name,
      scatterYColumn.value?.name,
      scatterTraceXColumn.value?.name,
      scatterTraceYColumn.value?.name,
    ].filter((name): name is string => Boolean(name)),
  );
  return columns.filter(
    (column) => column.isIndex && !excludedNames.has(column.name),
  );
});
const scatterBinColumn = ref<ColumnInfo>();
const updateScatterBinSelection = updateSelectionFn(scatterBinColumn);
watch([scatterBinColumnOptions, scatterMode], ([newOptions, mode]) => {
  if (mode === "xy") {
    updateScatterBinSelection(newOptions);
  } else {
    scatterBinColumn.value = undefined;
  }
});

const isTraceSeries = computed(() => series.value?.isTrace ?? false);
const isComplexSeries = computed(() => series.value?.isComplex ?? false);
const hasIndexColumn = computed(
  () => datasetDetail.value?.columns.some((column) => column.isIndex) ?? false,
);
const canUseScatterComplex = computed(
  () => scatterComplexOptions.value.length > 0,
);
const canUseScatterTraceXY = computed(
  () => scatterTraceXYOptions.value.length >= 2,
);
const canUseScatterXY = computed(
  () => scatterXYOptions.value.length >= 2 && hasIndexColumn.value,
);

const scatterModeOptions = computed(() => {
  const options: { label: string; value: ScatterMode }[] = [];
  if (canUseScatterComplex.value) {
    options.push({ label: "Complex (real/imag)", value: "complex" });
  }
  if (canUseScatterTraceXY.value) {
    options.push({ label: "Trace X/Y", value: "trace_xy" });
  }
  if (canUseScatterXY.value) {
    options.push({ label: "X/Y columns", value: "xy" });
  }
  return options;
});

watch(availableChartTypes, (types) => {
  if (types.length === 0) return;
  if (!types.includes(chartType.value)) {
    chartType.value = types[0]!;
  }
});

watch(
  [canUseScatterComplex, canUseScatterTraceXY, canUseScatterXY],
  ([hasComplex, hasTraceXY, hasXY]) => {
    if (!hasComplex && scatterMode.value === "complex") {
      scatterMode.value = hasTraceXY ? "trace_xy" : "xy";
    }
    if (!hasTraceXY && scatterMode.value === "trace_xy") {
      scatterMode.value = hasComplex ? "complex" : "xy";
    }
    if (!hasXY && scatterMode.value === "xy") {
      scatterMode.value = hasComplex ? "complex" : "trace_xy";
    }
  },
);

watch(scatterIsTraceBased, (isTraceBased) => {
  if (isTraceBased) {
    scatterBinColumn.value = undefined;
  }
});

/** Updates a column selection ref when options change */
function updateSelectionFn(
  optionRef: Ref<ColumnInfo | undefined>,
  defaultIndex = 0,
) {
  return (newOptions: ColumnInfo[]) => {
    const currentName = optionRef.value?.name;
    const found = newOptions.find((col) => col.name === currentName);
    optionRef.value = found ?? newOptions[defaultIndex] ?? newOptions[0];
  };
}

// ============================================================================
// Data Subscription
// ============================================================================

watchThrottled(
  () => props.datasetId,
  async (newId) => {
    let aborted = false;
    onWatcherCleanup(() => (aborted = true));

    const newDetail = await getDatasetDetail(newId);
    if (aborted) return;

    const newFilterTableData = await getFilterTableData(newId, {
      excludeColumns: excludeColumns.value,
    });
    if (aborted) return;

    datasetDetail.value = newDetail;
    filterTableData.value = newFilterTableData;

    // Start polling for updates if dataset is still being written
    const poll = async () => {
      while (!aborted) {
        const { isComplete } = await getDatasetWriteStatus(newId);
        if (aborted) return;

        if (isComplete) {
          // Final refresh and stop polling
          const pollDetail = await getDatasetDetail(newId);
          if (aborted) return;
          const pollFilter = await getFilterTableData(newId, {
            excludeColumns: excludeColumns.value,
          });
          if (aborted) return;
          datasetDetail.value = pollDetail;
          filterTableData.value = pollFilter;
          datasetUpdateTick.value += 1;
          break;
        }

        // Refresh data and continue polling
        const pollFilter = await getFilterTableData(newId, {
          excludeColumns: excludeColumns.value,
        });
        if (aborted) return;
        filterTableData.value = pollFilter;
        datasetUpdateTick.value += 1;

        await new Promise((resolve) => setTimeout(resolve, 1000));
      }
    };
    poll();
  },
  { throttle: 100, immediate: true },
);

// Update excludeColumns when axis or chart type changes
watch(
  [xColumn, yColumn, chartType, series, scatterMode, scatterBinColumn],
  async ([newX, newY, newType, newSeries, newScatterMode, newBinColumn]) => {
    const excludes: string[] = [];
    if (newType === "line") {
      if (newX) excludes.push(newX.name);
    } else if (newType === "heatmap") {
      if (newSeries?.isTrace) {
        if (newY) excludes.push(newY.name);
      } else {
        if (newX) excludes.push(newX.name);
        if (newY) excludes.push(newY.name);
      }
    } else if (newType === "scatter") {
      if (newScatterMode === "xy" && newBinColumn?.isIndex) {
        excludes.push(newBinColumn.name);
      }
    }
    excludeColumns.value = excludes;

    if (datasetDetail.value && props.datasetId) {
      filterTableData.value = await getFilterTableData(props.datasetId, {
        excludeColumns: excludes,
      });
    }
  },
);

// ============================================================================
// Chart Data Processing
// ============================================================================

const filter = shallowRef<FilterTableRow>();
const data = shallowRef<ChartOptions>();
const scatterError = ref<string | null>(null);

watchDebounced(
  [
    datasetDetail,
    series,
    filter,
    selectedComplexView,
    selectedComplexViewSingle,
    chartType,
    xColumn,
    yColumn,
    scatterMode,
    scatterSeries,
    scatterTraceXColumn,
    scatterTraceYColumn,
    scatterXColumn,
    scatterYColumn,
    scatterBinColumn,
    datasetUpdateTick,
  ],
  async () => {
    data.value = await getNewData();
  },
  { debounce: 50 },
);

/** Main data fetching and processing function */
async function getNewData(): Promise<ChartOptions | undefined> {
  const detailValue = datasetDetail.value;
  const indexRow = filter.value;
  const filterTableDataValue = filterTableData.value;
  const datasetId = props.datasetId;
  const xColumnValue = xColumn.value;
  const yColumnValue = yColumn.value;
  const seriesValue = series.value;
  const type = chartType.value;
  const scatterModeValue = scatterMode.value;
  const scatterSeriesValue = scatterSeries.value;
  const scatterTraceXValue = scatterTraceXColumn.value;
  const scatterTraceYValue = scatterTraceYColumn.value;
  const scatterXValue = scatterXColumn.value;
  const scatterYValue = scatterYColumn.value;
  const scatterBinColumnValue = scatterBinColumn.value;

  // Validation
  const columns = detailValue?.columns;
  if (!filterTableDataValue || !columns || !datasetId) {
    return undefined;
  }

  const filterFields = filterTableDataValue.fields;
  const hasFilters = filterFields.length > 0;
  if (hasFilters && !indexRow) {
    return undefined;
  }

  scatterError.value = null;

  if (type === "scatter") {
    if (!scatterModeValue) return undefined;
    if (scatterModeValue === "complex" && !scatterSeriesValue) return undefined;
    if (
      scatterModeValue === "trace_xy" &&
      (!scatterTraceXValue || !scatterTraceYValue)
    ) {
      return undefined;
    }
    if (scatterModeValue === "xy" && (!scatterXValue || !scatterYValue)) {
      return undefined;
    }
  } else {
    if (!seriesValue) return undefined;
    if (type === "line") {
      if (!seriesValue.isTrace && !xColumnValue) return undefined;
    } else if (type === "heatmap") {
      if (!yColumnValue) return undefined;
      if (!seriesValue.isTrace && !xColumnValue) return undefined;
    }
  }

  try {
    return await fetchChartData(datasetId, {
      chartType: type,
      series: seriesValue?.name,
      xColumn: xColumnValue?.name,
      yColumn: yColumnValue?.name,
      scatterMode: scatterModeValue,
      scatterSeries: scatterSeriesValue?.name,
      scatterXColumn: scatterXValue?.name,
      scatterYColumn: scatterYValue?.name,
      scatterTraceXColumn: scatterTraceXValue?.name,
      scatterTraceYColumn: scatterTraceYValue?.name,
      scatterBinColumn: scatterBinColumnValue?.name,
      complexViews: selectedComplexView.value,
      complexViewSingle: selectedComplexViewSingle.value,
      indexFilters: hasFilters ? indexRow!.valueIndices : undefined,
      excludeColumns: excludeColumns.value,
    });
  } catch (error) {
    if (type === "scatter") {
      scatterError.value =
        error instanceof Error
          ? error.message
          : "Scatter data error. Please check trace lengths.";
    }
    return undefined;
  }
}

// ============================================================================
// Template
// ============================================================================
</script>

<template>
  <div class="flex size-full flex-col">
    <div class="flex flex-wrap gap-2 p-2">
      <IftaLabel>
        <Select
          v-model="chartType"
          :options="availableChartTypes"
          input-id="chart-type-select"
          fluid
        />
        <label for="chart-type-select">Chart Type</label>
      </IftaLabel>
      <IftaLabel v-if="chartType !== 'scatter'">
        <Select
          v-model="series"
          :options="seriesOptions"
          option-label="name"
          input-id="main-series-select"
          fluid
        />
        <label for="main-series-select">Series</label>
      </IftaLabel>
      <IftaLabel
        v-if="
          chartType === 'line' || (chartType === 'heatmap' && !series?.isTrace)
        "
      >
        <Select
          v-model="xColumn"
          :options="xColumnOptions"
          :disabled="isTraceSeries && chartType === 'line'"
          option-label="name"
          input-id="x-column-select"
          fluid
        />
        <label for="x-column-select">X</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'heatmap'">
        <Select
          v-model="yColumn"
          :options="yColumnOptions"
          option-label="name"
          input-id="y-column-select"
          fluid
        />
        <label for="y-column-select">Y</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter'">
        <Select
          v-model="scatterMode"
          :options="scatterModeOptions"
          option-label="label"
          option-value="value"
          input-id="scatter-mode-select"
          fluid
        />
        <label for="scatter-mode-select">Point Cloud Source</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter' && scatterMode === 'complex'">
        <Select
          v-model="scatterSeries"
          :options="scatterComplexOptions"
          option-label="name"
          input-id="scatter-series-select"
          fluid
        />
        <label for="scatter-series-select">Complex Series</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter' && scatterMode === 'xy'">
        <Select
          v-model="scatterXColumn"
          :options="scatterXYOptions"
          option-label="name"
          input-id="scatter-x-select"
          fluid
        />
        <label for="scatter-x-select">X Column</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter' && scatterMode === 'xy'">
        <Select
          v-model="scatterYColumn"
          :options="scatterXYOptions"
          option-label="name"
          input-id="scatter-y-select"
          fluid
        />
        <label for="scatter-y-select">Y Column</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter' && scatterMode === 'trace_xy'">
        <Select
          v-model="scatterTraceXColumn"
          :options="scatterTraceXYOptions"
          option-label="name"
          input-id="scatter-trace-x-select"
          fluid
        />
        <label for="scatter-trace-x-select">Trace X</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter' && scatterMode === 'trace_xy'">
        <Select
          v-model="scatterTraceYColumn"
          :options="scatterTraceXYOptions"
          option-label="name"
          input-id="scatter-trace-y-select"
          fluid
        />
        <label for="scatter-trace-y-select">Trace Y</label>
      </IftaLabel>
      <IftaLabel v-if="chartType === 'scatter' && scatterMode === 'xy'">
        <Select
          v-model="scatterBinColumn"
          :options="scatterBinColumnOptions"
          option-label="name"
          input-id="scatter-bin-column-select"
          fluid
        />
        <label for="scatter-bin-column-select">Index Column (excluded)</label>
      </IftaLabel>
    </div>
    <div
      v-if="chartType === 'scatter' && scatterError"
      class="px-2 text-sm text-red-600"
    >
      {{ scatterError }}
    </div>
    <div v-if="chartType !== 'scatter'" class="flex items-center gap-2 p-2">
      <span class="text-sm font-medium">Complex:</span>
      <template v-if="chartType === 'heatmap'">
        <div
          v-for="option in complexSeriesOptions"
          :key="option"
          class="flex items-center gap-1"
        >
          <RadioButton
            v-model="selectedComplexViewSingle"
            :input-id="`complex-${option}`"
            :disabled="!isComplexSeries"
            :value="option"
          />
          <label :for="`complex-${option}`" class="text-sm">{{ option }}</label>
        </div>
      </template>
      <template v-else>
        <div
          v-for="option in complexSeriesOptions"
          :key="option"
          class="flex items-center gap-1"
        >
          <Checkbox
            v-model="selectedComplexView"
            :input-id="`complex-${option}`"
            :disabled="!isComplexSeries"
            name="selectedComplexView"
            :value="option"
          />
          <label :for="`complex-${option}`" class="text-sm">{{ option }}</label>
        </div>
      </template>
    </div>
    <Splitter class="min-h-0 flex-1" layout="vertical">
      <SplitterPanel>
        <ChartWrapper :data="data" />
      </SplitterPanel>
      <SplitterPanel>
        <FilterTable
          v-model="filter"
          :filter-table-data="filterTableData"
          :dataset-id="String(datasetId)"
        />
      </SplitterPanel>
    </Splitter>
  </div>
</template>
