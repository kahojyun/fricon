<script setup lang="ts">
import {
  computed,
  onUnmounted,
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
  fetchData,
  subscribeDatasetUpdate,
} from "@/backend.ts";
import { Vector } from "apache-arrow";
import { useThrottleFn, watchDebounced, watchThrottled } from "@vueuse/core";
import type { TypedArray } from "apache-arrow/interfaces";
import ChartWrapper, {
  type ChartOptions,
  type ChartSeries,
} from "./components/ChartWrapper.vue";
import FilterTable from "./components/FilterTable.vue";
import RadioButton from "primevue/radiobutton";
import {
  extractTraceXAxis,
  getColumnIndices,
  extractComplexComponents,
  transformComplexView,
  createMockVector,
  type ComplexViewOption,
} from "@/composables/chartDataHelpers";

const props = defineProps<{
  datasetId: number;
}>();

// ============================================================================
// Dataset and Filter State
// ============================================================================

const datasetDetail = shallowRef<DatasetDetail>();
const filterTableData = shallowRef<FilterTableData>();
const excludeColumns = ref<string[]>([]);
let unsubscribe: (() => Promise<void>) | undefined;

// ============================================================================
// Chart Configuration
// ============================================================================

const chartType = ref<"line" | "heatmap">("line");
const availableChartTypes = computed(() => {
  if (!series.value) return [];
  return ["line", "heatmap"];
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

const isTraceSeries = computed(() => series.value?.isTrace ?? false);
const isComplexSeries = computed(() => series.value?.isComplex ?? false);

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

    await unsubscribe?.();

    const newDetail = await getDatasetDetail(newId);
    if (aborted) return;

    const newFilterTableData = await getFilterTableData(newId, {
      excludeColumns: excludeColumns.value,
    });
    if (aborted) return;

    const updateCallback = useThrottleFn(async () => {
      const v = await getFilterTableData(newId, {
        excludeColumns: excludeColumns.value,
      });
      filterTableData.value = v;
    }, 1000);

    unsubscribe = await subscribeDatasetUpdate(newId, updateCallback);
    datasetDetail.value = newDetail;
    filterTableData.value = newFilterTableData;
  },
  { throttle: 100, immediate: true },
);

onUnmounted(async () => {
  await unsubscribe?.();
});

// Update excludeColumns when axis or chart type changes
watch(
  [xColumn, yColumn, chartType, series],
  async ([newX, newY, newType, newSeries]) => {
    const excludes: string[] = [];
    if (newType === "line") {
      if (newX) excludes.push(newX.name);
    } else if (newSeries?.isTrace) {
      if (newY) excludes.push(newY.name);
    } else {
      if (newX) excludes.push(newX.name);
      if (newY) excludes.push(newY.name);
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
  ],
  async () => {
    data.value = await getNewData();
  },
  { debounce: 50 },
);

/** Determines which column indices to fetch based on chart type */
function buildSelectColumns(
  columns: ColumnInfo[],
  seriesValue: ColumnInfo,
  type: "line" | "heatmap",
  xName: string | undefined,
  yName: string | undefined,
): number[] | null {
  const seriesIndex = columns.findIndex((c) => c.name === seriesValue.name);
  if (seriesIndex === -1) return null;

  if (type === "line") {
    if (seriesValue.isTrace) {
      return [seriesIndex];
    }
    const indices = getColumnIndices(columns, [xName]);
    return indices ? [...indices, seriesIndex] : null;
  }

  // Heatmap
  if (seriesValue.isTrace) {
    const indices = getColumnIndices(columns, [yName]);
    return indices ? [...indices, seriesIndex] : null;
  }
  const indices = getColumnIndices(columns, [xName, yName]);
  return indices ? [...indices, seriesIndex] : null;
}

/** Processes data for line charts */
function processLineChartData(
  newData: import("apache-arrow").Table,
  seriesValue: ColumnInfo,
  seriesVector: Vector,
): { x: number[] | TypedArray; rawY: Vector } | null {
  if (seriesValue.isTrace) {
    if (newData.numRows !== 1) {
      console.error(
        "Trace series should fetch exactly 1 row, actual:",
        newData.numRows,
      );
      return null;
    }
    const result = extractTraceXAxis(seriesVector, 0);
    if (!result) return null;
    return { x: result.x, rawY: result.y };
  }

  return {
    x: newData.getChildAt(0)!.toArray() as TypedArray,
    rawY: seriesVector,
  };
}

/** Processes data for heatmap charts */
function processHeatmapData(
  newData: import("apache-arrow").Table,
  seriesValue: ColumnInfo,
  seriesVector: Vector,
): { x: number[] | TypedArray; y: number[] | TypedArray; rawY: Vector } | null {
  if (seriesValue.isTrace) {
    const yVector = newData.getChildAt(0)!;
    const flatX: number[] = [];
    const flatY: number[] = [];
    const accumulatedZ: (number | { real: number; imag: number })[] = [];

    for (let r = 0; r < newData.numRows; r++) {
      const rowY = yVector.get(r);
      if (rowY === null) continue;

      const result = extractTraceXAxis(seriesVector, r);
      if (!result) continue;

      for (let i = 0; i < result.y.length; i++) {
        flatX.push(result.x[i]!);
        flatY.push(rowY);
        accumulatedZ.push(result.y.get(i));
      }
    }

    return {
      x: Float64Array.from(flatX),
      y: Float64Array.from(flatY),
      rawY: createMockVector(accumulatedZ) as unknown as Vector,
    };
  }

  // Scalar Heatmap
  return {
    x: newData.getChildAt(0)!.toArray() as TypedArray,
    y: newData.getChildAt(1)!.toArray() as TypedArray,
    rawY: seriesVector,
  };
}

/** Builds chart series with optional complex number transformation */
function buildChartSeries(
  rawY: Vector,
  seriesName: string,
  isComplex: boolean,
  type: "line" | "heatmap",
  complexViewOptions: ComplexViewOption[],
  complexViewSingle: ComplexViewOption,
): ChartSeries[] {
  if (!isComplex) {
    return [{ name: seriesName, data: rawY.toArray() as TypedArray }];
  }

  const components = extractComplexComponents(rawY);
  const options = type === "heatmap" ? [complexViewSingle] : complexViewOptions;

  return options.map((option) => ({
    name: `${seriesName} (${option})`,
    data: transformComplexView(components, option),
  }));
}

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

  // Validation
  const columns = detailValue?.columns;
  if (!filterTableDataValue || !columns || !datasetId || !seriesValue) {
    return undefined;
  }

  const filterFields = filterTableDataValue.fields;
  const hasFilters = filterFields.length > 0;
  if (hasFilters && !indexRow) {
    return undefined;
  }

  // Build column selection
  const selectColumns = buildSelectColumns(
    columns,
    seriesValue,
    type,
    xColumnValue?.name,
    yColumnValue?.name,
  );
  if (!selectColumns) return undefined;

  // Fetch data
  const newData = await fetchData(datasetId, {
    indexFilters: hasFilters ? indexRow!.valueIndices : undefined,
    excludeColumns: excludeColumns.value,
    columns: selectColumns,
  });

  const seriesVector = newData.getChild(seriesValue.name);
  if (!seriesVector) {
    console.error("No series column returned", seriesValue);
    return undefined;
  }

  // Process based on chart type
  let finalX: number[] | TypedArray;
  let finalY: number[] | TypedArray | undefined;
  let rawYColumn: Vector;

  if (type === "line") {
    const result = processLineChartData(newData, seriesValue, seriesVector);
    if (!result) return undefined;
    finalX = result.x;
    rawYColumn = result.rawY;
  } else {
    const result = processHeatmapData(newData, seriesValue, seriesVector);
    if (!result) return undefined;
    finalX = result.x;
    finalY = result.y;
    rawYColumn = result.rawY;
  }

  // Build series
  const seriesData = buildChartSeries(
    rawYColumn,
    seriesValue.name,
    isComplexSeries.value,
    type,
    selectedComplexView.value,
    selectedComplexViewSingle.value,
  );

  return {
    type,
    x: finalX,
    xName:
      xColumnValue?.name ??
      (seriesValue.isTrace && type === "line"
        ? `${seriesValue.name} - X`
        : "X"),
    y: finalY,
    yName: yColumnValue?.name,
    series: seriesData,
  };
}
</script>

<template>
  <div class="flex size-full flex-col">
    <div class="flex flex-wrap gap-2 p-2">
      <IftaLabel>
        <Select
          v-model="series"
          :options="seriesOptions"
          option-label="name"
          input-id="main-series-select"
          fluid
        />
        <label for="main-series-select">Series</label>
      </IftaLabel>
      <IftaLabel>
        <Select
          v-model="chartType"
          :options="availableChartTypes"
          input-id="chart-type-select"
          fluid
        />
        <label for="chart-type-select">Chart Type</label>
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
    </div>
    <div class="flex items-center gap-2 p-2">
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
        <ChartWrapper :data />
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
