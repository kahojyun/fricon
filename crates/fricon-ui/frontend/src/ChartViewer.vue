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
const datasetUpdateTick = ref(0);
let unsubscribe: (() => Promise<void>) | undefined;

// ============================================================================
// Chart Configuration
// ============================================================================

type ChartType = "line" | "heatmap" | "scatter";
type ScatterMode = "complex" | "trace_xy" | "xy";

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

function buildScatterSelectColumns(
  columns: ColumnInfo[],
  mode: ScatterMode,
  seriesValue: ColumnInfo | undefined,
  xValue: ColumnInfo | undefined,
  yValue: ColumnInfo | undefined,
  traceXValue: ColumnInfo | undefined,
  traceYValue: ColumnInfo | undefined,
  binColumnValue: ColumnInfo | undefined,
): number[] | null {
  const indices: number[] = [];
  const isTraceBased =
    mode === "trace_xy" || (mode === "complex" && seriesValue?.isTrace);
  if (mode === "complex") {
    if (!seriesValue) return null;
    const seriesIndex = columns.findIndex((c) => c.name === seriesValue.name);
    if (seriesIndex === -1) return null;
    indices.push(seriesIndex);
  } else if (mode === "trace_xy") {
    const columnIndices = getColumnIndices(columns, [
      traceXValue?.name,
      traceYValue?.name,
    ]);
    if (!columnIndices) return null;
    indices.push(...columnIndices);
  } else {
    const columnIndices = getColumnIndices(columns, [
      xValue?.name,
      yValue?.name,
    ]);
    if (!columnIndices) return null;
    indices.push(...columnIndices);
  }

  if (!isTraceBased && binColumnValue) {
    const binIndex = columns.findIndex((c) => c.name === binColumnValue.name);
    if (binIndex === -1) return null;
    if (!indices.includes(binIndex)) {
      indices.push(binIndex);
    }
  }
  return indices;
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
      datasetUpdateTick.value += 1;
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

function pushScatterPoint(
  map: Map<string, [number, number][]>,
  key: string,
  point: [number, number],
) {
  const bucket = map.get(key);
  if (bucket) {
    bucket.push(point);
    return;
  }
  map.set(key, [point]);
}

function buildScatterSeriesFromMap(
  map: Map<string, [number, number][]>,
): ChartSeries[] {
  return Array.from(map.entries()).map(([name, points]) => ({
    name,
    data: points,
  }));
}

function processScatterData(
  newData: import("apache-arrow").Table,
  mode: ScatterMode,
  seriesValue: ColumnInfo | undefined,
  xValue: ColumnInfo | undefined,
  yValue: ColumnInfo | undefined,
  traceXValue: ColumnInfo | undefined,
  traceYValue: ColumnInfo | undefined,
): { xName: string; yName: string; series: ChartSeries[] } | null {
  const seriesMap = new Map<string, [number, number][]>();

  if (mode === "complex") {
    if (!seriesValue) return null;
    const seriesVector = newData.getChild(seriesValue.name);
    if (!seriesVector) return null;

    if (seriesValue.isTrace) {
      for (let r = 0; r < newData.numRows; r++) {
        const traceResult = extractTraceXAxis(seriesVector, r);
        if (!traceResult) continue;
        const components = extractComplexComponents(traceResult.y);
        const { reals, imags } = components;
        const binKey = seriesValue.name;

        for (let i = 0; i < reals.length; i++) {
          pushScatterPoint(seriesMap, binKey, [reals[i]!, imags[i]!]);
        }
      }
    } else {
      const components = extractComplexComponents(seriesVector);
      const { reals, imags } = components;
      for (let i = 0; i < reals.length; i++) {
        const binKey = seriesValue.name;
        pushScatterPoint(seriesMap, binKey, [reals[i]!, imags[i]!]);
      }
    }

    return {
      xName: `${seriesValue.name} (real)`,
      yName: `${seriesValue.name} (imag)`,
      series: buildScatterSeriesFromMap(seriesMap),
    };
  }

  if (mode === "trace_xy") {
    if (!traceXValue || !traceYValue) return null;
    const xVector = newData.getChild(traceXValue.name);
    const yVector = newData.getChild(traceYValue.name);
    if (!xVector || !yVector) return null;

    for (let r = 0; r < newData.numRows; r++) {
      const xTrace = extractTraceXAxis(xVector, r);
      const yTrace = extractTraceXAxis(yVector, r);
      if (!xTrace || !yTrace) continue;
      if (xTrace.y.length !== yTrace.y.length) {
        throw new Error(
          `Trace length mismatch at row ${r + 1}: ${traceXValue.name} (${xTrace.y.length}) vs ${traceYValue.name} (${yTrace.y.length})`,
        );
      }
      const binKey = `${traceXValue.name} vs ${traceYValue.name}`;
      for (let i = 0; i < xTrace.y.length; i++) {
        const xPoint = xTrace.y.get(i);
        const yPoint = yTrace.y.get(i);
        if (xPoint === null || yPoint === null) continue;
        pushScatterPoint(seriesMap, binKey, [xPoint, yPoint]);
      }
    }

    return {
      xName: traceXValue.name,
      yName: traceYValue.name,
      series: buildScatterSeriesFromMap(seriesMap),
    };
  }

  if (!xValue || !yValue) return null;
  const xVector = newData.getChild(xValue.name);
  const yVector = newData.getChild(yValue.name);
  if (!xVector || !yVector) return null;
  const xArray = xVector.toArray() as TypedArray;
  const yArray = yVector.toArray() as TypedArray;

  for (let i = 0; i < xArray.length; i++) {
    const xPoint = xArray[i];
    const yPoint = yArray[i];
    if (xPoint === undefined || yPoint === undefined) continue;
    const binKey = `${xValue.name} vs ${yValue.name}`;
    pushScatterPoint(seriesMap, binKey, [xPoint, yPoint]);
  }

  return {
    xName: xValue.name,
    yName: yValue.name,
    series: buildScatterSeriesFromMap(seriesMap),
  };
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

  // Build column selection
  if (type === "scatter") {
    scatterError.value = null;
    const selectColumns = buildScatterSelectColumns(
      columns,
      scatterModeValue,
      scatterSeriesValue,
      scatterXValue,
      scatterYValue,
      scatterTraceXValue,
      scatterTraceYValue,
      scatterBinColumnValue,
    );
    if (!selectColumns) return undefined;

    const newData = await fetchData(datasetId, {
      indexFilters: hasFilters ? indexRow!.valueIndices : undefined,
      excludeColumns: excludeColumns.value,
      columns: selectColumns,
    });

    let scatterResult: {
      xName: string;
      yName: string;
      series: ChartSeries[];
    } | null;
    try {
      scatterResult = processScatterData(
        newData,
        scatterModeValue,
        scatterSeriesValue,
        scatterXValue,
        scatterYValue,
        scatterTraceXValue,
        scatterTraceYValue,
      );
    } catch (error) {
      scatterError.value =
        error instanceof Error
          ? error.message
          : "Scatter data error. Please check trace lengths.";
      return undefined;
    }
    if (!scatterResult) return undefined;

    return {
      type: "scatter",
      xName: scatterResult.xName,
      yName: scatterResult.yName,
      series: scatterResult.series,
    };
  }

  if (!seriesValue) {
    return undefined;
  }

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

  if (type === "line") {
    return {
      type: "line",
      x: finalX,
      xName:
        xColumnValue?.name ??
        (seriesValue.isTrace ? `${seriesValue.name} - X` : "X"),
      series: seriesData,
    };
  }

  if (!finalY || !yColumnValue?.name) {
    return undefined;
  }

  return {
    type: "heatmap",
    x: finalX,
    xName: xColumnValue?.name ?? "X",
    y: finalY,
    yName: yColumnValue.name,
    series: seriesData,
  };
}
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
