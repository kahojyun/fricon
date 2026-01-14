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
  type LinePlotOptions,
  type LineSeries,
} from "./components/ChartWrapper.vue";
import FilterTable from "./components/FilterTable.vue";

const props = defineProps<{
  datasetId: number;
}>();

const datasetDetail = shallowRef<DatasetDetail>();
const filterTableData = shallowRef<FilterTableData>();
const xColumnName = ref<string | undefined>();
let unsubscribe: (() => Promise<void>) | undefined;

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
      xColumnName: xColumnName.value,
    });
    if (aborted) return;

    const updateCallback = useThrottleFn(async () => {
      const v = await getFilterTableData(newId, {
        xColumnName: xColumnName.value,
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

// Update xColumnName when xColumn changes, and refetch filter table data
watch(xColumn, async (newXColumn) => {
  xColumnName.value = newXColumn?.name;
  if (datasetDetail.value && props.datasetId) {
    const newFilterTableData = await getFilterTableData(props.datasetId, {
      xColumnName: newXColumn?.name,
    });
    filterTableData.value = newFilterTableData;
  }
});

function updateSelectionFn(optionRef: Ref<ColumnInfo | undefined>) {
  return (newOptions: ColumnInfo[]) => {
    const currentName = optionRef.value?.name;
    optionRef.value =
      newOptions.find((col) => col.name === currentName) ?? newOptions[0];
  };
}

const complexSeriesOptions = ["real", "imag", "mag", "arg"];
const selectedComplexView = ref(["real", "imag"]);

const isTraceSeries = computed(() => series.value?.isTrace ?? false);
const isComplexSeries = computed(() => series.value?.isComplex ?? false);

const filter = shallowRef<FilterTableRow>();

const data = shallowRef<LinePlotOptions>();
watchDebounced(
  [datasetDetail, series, filter, selectedComplexView],
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
  const seriesValue = series.value;

  const columns = detailValue?.columns;
  if (!filterTableDataValue) return undefined;

  // Build filter fields from filterTableData (columns except X)
  const filterFields = filterTableDataValue.fields;

  if (
    !columns ||
    !((filterFields.length > 0 && indexRow) || filterFields.length == 0) ||
    !datasetId ||
    !seriesValue
  )
    return undefined;

  let indexFilters: Record<string, unknown> | undefined;
  if (filterFields.length > 0 && indexRow) {
    // Build a simple JSON object mapping field names to filter values
    indexFilters = {};
    for (let i = 0; i < filterFields.length; i++) {
      const fieldName = filterFields[i]!;
      indexFilters[fieldName] = indexRow.values[i];
    }
  }

  const seriesIndex = columns.findIndex((c) => c.name === seriesValue.name);
  if (seriesIndex === -1) return undefined;
  let selectColumns: number[];
  if (seriesValue.isTrace) {
    selectColumns = [seriesIndex];
  } else {
    if (!xColumnValue) return undefined;
    const xIndex = columns.findIndex((c) => c.name === xColumnValue.name);
    if (xIndex === -1) return undefined;
    selectColumns = [xIndex, seriesIndex];
  }

  const newData = await fetchData(datasetId, {
    indexFilters,
    columns: selectColumns,
  });

  const seriesVector = newData.getChild(seriesValue.name);
  if (seriesVector == null) {
    console.error("No series column returned", seriesValue);
    return undefined;
  }
  let x: number[] | TypedArray;
  let rawYColumn: Vector;
  if (seriesValue.isTrace) {
    if (newData.numRows !== 1) {
      console.error(
        "Trace series should fetch exactly 1 row, actual: ",
        newData.numRows,
      );
      return undefined;
    }
    if (DataType.isList(seriesVector.type)) {
      rawYColumn = seriesVector.get(0) as Vector;
      x = Int32Array.from({ length: rawYColumn.length }, (_, i) => i);
    } else {
      rawYColumn = seriesVector.getChild("y")!.get(0) as Vector;
      if (seriesVector.numChildren === 2) {
        x = (
          seriesVector.getChild("x")!.get(0) as Vector
        ).toArray() as TypedArray;
      } else {
        const firstRow = (
          seriesVector as Vector<Struct<{ x0: Float64; step: Float64 }>>
        ).get(0)!;
        const x0 = firstRow.x0;
        const step = firstRow.step;
        x = Float64Array.from(
          { length: rawYColumn.length },
          (_, i) => x0 + i * step,
        );
      }
    }
  } else {
    x = newData.getChildAt(0)!.toArray() as TypedArray;
    rawYColumn = seriesVector;
  }

  // Handle complex data if needed
  let seriesData: LineSeries[];
  if (isComplexSeries.value) {
    seriesData = [];
    const complexYColumn = rawYColumn as Vector<
      Struct<{ real: Float64; imag: Float64 }>
    >;

    for (const option of selectedComplexView.value) {
      let transformedY: number[] | TypedArray;
      switch (option) {
        case "real":
          transformedY = complexYColumn.getChildAt(0)?.toArray() as TypedArray;
          break;
        case "imag":
          transformedY = complexYColumn.getChildAt(1)?.toArray() as TypedArray;
          break;
        case "mag":
          transformedY = complexYColumn
            .toArray()
            .map(({ real, imag }) => Math.sqrt(real * real + imag * imag));
          break;
        case "arg":
          transformedY = complexYColumn
            .toArray()
            .map(({ real, imag }) => Math.atan2(imag, real));
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
    const rawY = rawYColumn.toArray() as TypedArray;
    seriesData = [{ name: seriesValue.name, data: rawY }];
  }

  return {
    x,
    xName: xColumnValue?.name ?? `${seriesValue.name} - X`,
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
          v-model="xColumn"
          :options="xColumnOptions"
          :disabled="isTraceSeries"
          option-label="name"
          input-id="x-column-select"
          fluid
        />
        <label for="x-column-select">X</label>
      </IftaLabel>
    </div>
    <div class="flex items-center gap-2 p-2">
      <span class="text-sm font-medium">Complex:</span>
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
