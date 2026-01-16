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
import { DataType, Float64, Struct, Vector } from "apache-arrow";
import { useThrottleFn, watchDebounced, watchThrottled } from "@vueuse/core";
import type { TypedArray } from "apache-arrow/interfaces";
import ChartWrapper, {
  type ChartOptions,
  type ChartSeries,
} from "./components/ChartWrapper.vue";
import FilterTable from "./components/FilterTable.vue";
import RadioButton from "primevue/radiobutton";

const props = defineProps<{
  datasetId: number;
}>();

const datasetDetail = shallowRef<DatasetDetail>();
const filterTableData = shallowRef<FilterTableData>();
const excludeColumns = ref<string[]>([]);
let unsubscribe: (() => Promise<void>) | undefined;

const chartType = ref<"line" | "heatmap">("line");
const availableChartTypes = computed(() => {
  if (!series.value) return [];
  // For now, allow both line and heatmap for all types, logic differentiates requirements
  return ["line", "heatmap"];
});

watchThrottled(
  () => props.datasetId,
  async (newId) => {
    let aborted = false;
    onWatcherCleanup(() => (aborted = true));

    await unsubscribe?.();

    const newDetail = await getDatasetDetail(newId);
    if (aborted) return;

    // Get initial filter table data
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
watch(yColumnOptions, updateSelectionFn(yColumn));

// Update excludeColumns when X or Y or ChartType changes
watch(
  [xColumn, yColumn, chartType, series],
  async ([newX, newY, newType, newSeries]) => {
    const excludes: string[] = [];
    if (newType === "line") {
      if (newX) excludes.push(newX.name);
    } else {
      // Heatmap
      if (newSeries?.isTrace) {
        // Trace Heatmap: exclude Y (variation axis)
        if (newY) excludes.push(newY.name);
      } else {
        // Scalar Heatmap: exclude X and Y
        if (newX) excludes.push(newX.name);
        if (newY) excludes.push(newY.name);
      }
    }
    excludeColumns.value = excludes;

    if (datasetDetail.value && props.datasetId) {
      const newFilterTableData = await getFilterTableData(props.datasetId, {
        excludeColumns: excludes,
      });
      filterTableData.value = newFilterTableData;
    }
  },
);

function updateSelectionFn(optionRef: Ref<ColumnInfo | undefined>) {
  return (newOptions: ColumnInfo[]) => {
    const currentName = optionRef.value?.name;
    optionRef.value =
      newOptions.find((col) => col.name === currentName) ?? newOptions[0];
  };
}

const complexSeriesOptions = ["real", "imag", "mag", "arg"];
const selectedComplexView = ref(["real", "imag"]);
const selectedComplexViewSingle = ref("mag"); // For single select

const isTraceSeries = computed(() => series.value?.isTrace ?? false);
const isComplexSeries = computed(() => series.value?.isComplex ?? false);

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
async function getNewData() {
  const detailValue = datasetDetail.value;
  const indexRow = filter.value;
  const filterTableDataValue = filterTableData.value;
  const datasetId = props.datasetId;
  const xColumnValue = xColumn.value;
  const yColumnValue = yColumn.value;
  const seriesValue = series.value;
  const type = chartType.value;

  const columns = detailValue?.columns;
  if (!filterTableDataValue) return undefined;

  // Build filter fields from filterTableData (columns except excluded ones)
  const filterFields = filterTableDataValue.fields;

  if (
    !columns ||
    !((filterFields.length > 0 && indexRow) || filterFields.length == 0) ||
    !datasetId ||
    !seriesValue
  )
    return undefined;

  let indexFilters: number[] | undefined;
  if (filterFields.length > 0 && indexRow) {
    indexFilters = indexRow.valueIndices;
  }

  const seriesIndex = columns.findIndex((c) => c.name === seriesValue.name);
  if (seriesIndex === -1) return undefined;

  let selectColumns: number[];

  if (type === "line") {
    if (seriesValue.isTrace) {
      selectColumns = [seriesIndex];
    } else {
      if (!xColumnValue) return undefined;
      const xIndex = columns.findIndex((c) => c.name === xColumnValue.name);
      if (xIndex === -1) return undefined;
      selectColumns = [xIndex, seriesIndex];
    }
  } else {
    // Heatmap
    if (seriesValue.isTrace) {
      if (!yColumnValue) return undefined;
      const yIndex = columns.findIndex((c) => c.name === yColumnValue.name);
      if (yIndex === -1) return undefined;
      selectColumns = [yIndex, seriesIndex];
      // Logic for excluded columns handled in watcher, here just fetching
    } else {
      // Scalar Heatmap
      if (!xColumnValue || !yColumnValue) return undefined;
      const xIndex = columns.findIndex((c) => c.name === xColumnValue.name);
      const yIndex = columns.findIndex((c) => c.name === yColumnValue.name);
      if (xIndex === -1 || yIndex === -1) return undefined;
      selectColumns = [xIndex, yIndex, seriesIndex];
    }
  }

  const newData = await fetchData(datasetId, {
    indexFilters,
    excludeColumns: excludeColumns.value,
    columns: selectColumns,
  });

  const seriesVector = newData.getChild(seriesValue.name);
  if (seriesVector == null) {
    console.error("No series column returned", seriesValue);
    return undefined;
  }

  let finalX: number[] | TypedArray;
  let finalY: number[] | TypedArray | undefined;
  let rawYColumn: Vector; // This is the Z values source

  if (type === "line") {
    if (seriesValue.isTrace) {
      if (newData.numRows !== 1) {
        // If we have > 1 row for Trace Line, maybe warn or default to first?
        // Existing logic errors.
        console.error(
          "Trace series should fetch exactly 1 row, actual: ",
          newData.numRows,
        );
        return undefined;
      }
      if (DataType.isList(seriesVector.type)) {
        rawYColumn = seriesVector.get(0) as Vector;
        finalX = Int32Array.from({ length: rawYColumn.length }, (_, i) => i);
      } else {
        rawYColumn = seriesVector.getChild("y")!.get(0) as Vector;
        if (seriesVector.numChildren === 2) {
          finalX = (
            seriesVector.getChild("x")!.get(0) as Vector
          ).toArray() as TypedArray;
        } else {
          const firstRow = (
            seriesVector as Vector<Struct<{ x0: Float64; step: Float64 }>>
          ).get(0)!;
          const x0 = firstRow.x0;
          const step = firstRow.step;
          finalX = Float64Array.from(
            { length: rawYColumn.length },
            (_, i) => x0 + i * step,
          );
        }
      }
    } else {
      finalX = newData.getChildAt(0)!.toArray() as TypedArray;
      rawYColumn = seriesVector;
    }
  } else {
    // Heatmap
    if (seriesValue.isTrace) {
      // Y column is column 0 (selectColumns=[y, series])
      const yVector = newData.getChildAt(0)!;
      const traceVector = seriesVector;

      const numRows = newData.numRows;
      // Flatten
      const flatX: number[] = [];
      const flatY: number[] = [];
      // For Complex Trace, rawYColumn will be composed later
      // But here we need to extract from Trace Vector

      // We need to know Trace length. Assuming uniform?
      // Apache Arrow Lists?
      // Just iterate.
      // Wait, for Complex support, we need to extract "Real" or "Imag" from Trace.
      // So rawYColumn needs to be the flattened Z values.

      // Let's build flatZ array(s) depending on complexity later.
      // Here just build X and Y.
      // And build a "virtual" rawYColumn which is the concatenation of all traces.

      // We can't easily concatenate Vectors in JS if they are Structs/Lists efficiently without copying?
      // Actually, if we just push to array.

      const accumulatedZ: (number | { real: number; imag: number })[] = []; // Temporary holding

      for (let r = 0; r < numRows; r++) {
        const rowY = yVector.get(r);
        if (rowY === null) continue;

        let rowTraceY: Vector;
        let rowTraceX: number[] | TypedArray;

        if (DataType.isList(traceVector.type)) {
          const vec = traceVector.get(r) as Vector | null;
          if (!vec) continue;
          rowTraceY = vec;
          rowTraceX = Int32Array.from(
            { length: rowTraceY.length },
            (_, i) => i,
          );
        } else {
          const vecY = traceVector.getChild("y")?.get(r) as Vector | null;
          if (!vecY) continue;
          rowTraceY = vecY;

          if (traceVector.numChildren === 2) {
            const vecX = traceVector.getChild("x")?.get(r) as Vector | null;
            if (!vecX) {
              rowTraceX = Int32Array.from(
                { length: rowTraceY.length },
                (_, i) => i,
              );
            } else {
              rowTraceX = vecX.toArray() as TypedArray;
            }
          } else {
            const rowStruct = (
              traceVector as Vector<Struct<{ x0: Float64; step: Float64 }>>
            ).get(r);
            if (!rowStruct) {
              rowTraceX = Int32Array.from(
                { length: rowTraceY.length },
                (_, i) => i,
              );
            } else {
              const x0 = rowStruct.x0;
              const step = rowStruct.step;
              rowTraceX = Float64Array.from(
                { length: rowTraceY.length },
                (_, i) => x0 + i * step,
              );
            }
          }
        }

        for (let i = 0; i < rowTraceY.length; i++) {
          flatX.push(rowTraceX[i]!);
          flatY.push(rowY);
          accumulatedZ.push(rowTraceY.get(i));
        }
      }
      finalX = Float64Array.from(flatX);
      finalY = Float64Array.from(flatY);

      // Reconstruct rawYColumn from accumulatedZ
      // This is slow. But necessary for "ChartWrapper" which expects Arrays.

      // If "isComplex", accumulatedZ contains Objects {real, imag}.
      // If scalar, numbers.

      if (isComplexSeries.value) {
        // Mock a Vector-like object or just process accumulatedZ
        // Line 211 uses `rawYColumn as Vector`.
        // We can just skip 'rawYColumn' usage and build seriesData directly?
        // But existing logic uses existing complex logic.
        // Let's adapt.
      }

      // To reuse logic, let's make rawYColumn an array-like object
      rawYColumn = {
        toArray: () => accumulatedZ,
        length: accumulatedZ.length,
        get: (i: number) => accumulatedZ[i],
      } as any;
    } else {
      // Scalar Heatmap
      finalX = newData.getChildAt(0)!.toArray() as TypedArray;
      finalY = newData.getChildAt(1)!.toArray() as TypedArray;
      rawYColumn = seriesVector;
    }
  }

  // Handle complex data
  let seriesData: ChartSeries[];

  if (isComplexSeries.value) {
    seriesData = [];
    const complexYColumn = rawYColumn as any; // Vector or Array

    const options =
      type === "heatmap"
        ? [selectedComplexViewSingle.value]
        : selectedComplexView.value;

    for (const option of options) {
      let transformedY: number[] | TypedArray;

      // We need to handle if complexYColumn is Vector or Array (from Heatmap Trace flattening)
      let reals: any;
      let imags: any;

      if (Array.isArray(complexYColumn.toArray())) {
        // If we mocked it as above "toArray() => accumulatedZ"
        // actually Vector.toArray() returns array of struct/values
        const arr = complexYColumn.toArray() as {
          real: number;
          imag: number;
        }[];
        reals = new Float64Array(arr.length);
        imags = new Float64Array(arr.length);
        for (let i = 0; i < arr.length; i++) {
          reals[i] = arr[i]!.real;
          imags[i] = arr[i]!.imag;
        }
      } else {
        // It's a proper Arrow Vector
        reals = complexYColumn.getChild("real")!.toArray();
        imags = complexYColumn.getChild("imag")!.toArray();
      }

      switch (option) {
        case "real":
          transformedY = reals;
          break;
        case "imag":
          transformedY = imags;
          break;
        case "mag":
          transformedY = new Float64Array(reals.length);
          for (let i = 0; i < reals.length; i++)
            transformedY[i] = Math.sqrt(
              reals[i] * reals[i] + imags[i] * imags[i],
            );
          break;
        case "arg":
          transformedY = new Float64Array(reals.length);
          for (let i = 0; i < reals.length; i++)
            transformedY[i] = Math.atan2(imags[i], reals[i]);
          break;
        default:
          console.warn("Unexpected complex view", option);
          continue;
      }
      seriesData.push({
        name: `${seriesValue.name} (${option})`,
        data: transformedY,
      });
    }
  } else {
    // Scalar
    const rawY = rawYColumn.toArray() as TypedArray; // works for Arrow Vector and my Mock
    seriesData = [{ name: seriesValue.name, data: rawY }];
  }

  return {
    type: type,
    x: finalX!,
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
