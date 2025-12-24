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
  getDatasetDetail,
  fetchData,
  subscribeDatasetUpdate,
} from "@/backend.ts";
import {
  DataType,
  Float64,
  Struct,
  type StructRowProxy,
  type Table,
  tableToIPC,
  Vector,
} from "apache-arrow";
import { useThrottleFn, watchDebounced, watchThrottled } from "@vueuse/core";
import type { TypedArray } from "apache-arrow/interfaces";
import ChartWrapper, {
  type LinePlotOptions,
  type LineSeries,
} from "./components/ChartWrapper.vue";

const props = defineProps<{
  datasetId: number;
}>();

const datasetDetail = shallowRef<DatasetDetail>();
const indexTable = shallowRef<Table>();
let unsubscribe: (() => Promise<void>) | undefined;
watchThrottled(
  () => props.datasetId,
  async (newId) => {
    let aborted = false;
    onWatcherCleanup(() => (aborted = true));

    await unsubscribe?.();

    const newDetail = await getDatasetDetail(newId);
    if (aborted) return;

    const indexColumns = newDetail.columns.reduce((acc, c, i) => {
      if (c.isIndex) acc.push(i);
      return acc;
    }, [] as number[]);
    const newIndexTable = await fetchData(newId, {
      columns: indexColumns,
    });
    if (aborted) return;

    const updateCallback = useThrottleFn(async () => {
      const v = await fetchData(newId, {
        columns: indexColumns,
      });
      indexTable.value = v;
    }, 1000);
    unsubscribe = await subscribeDatasetUpdate(newId, updateCallback);
    datasetDetail.value = newDetail;
    indexTable.value = newIndexTable;
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

// Toggle for individual vs combined filter mode
const isIndividualFilterMode = ref(false);

// Store individual column selections when in individual mode
const individualColumnSelections = ref<Record<string, unknown[]>>({});

const filter = shallowRef<{ row: StructRowProxy; index: number }>();
const filterTable = computed(() => buildFilterTable());
watch(xColumn, () => {
  filter.value = filterTable.value?.rows[0];
  // Reset individual selections when x column changes
  individualColumnSelections.value = {};
});

// Check if we have multiple columns to show toggle button
const showFilterToggle = computed(() => {
  return filterTable.value && filterTable.value.fields.length > 1;
});

// Define type for column unique values
interface ColumnValueOption {
  value: unknown;
  displayValue: string;
}

// Compute unique values for each column when in individual mode
const columnUniqueValues = computed<Record<string, ColumnValueOption[]>>(() => {
  const filterTableValue = filterTable.value;
  if (!filterTableValue || !isIndividualFilterMode.value) return {};

  const uniqueValues: Record<string, ColumnValueOption[]> = {};

  filterTableValue.fields.forEach((field) => {
    const values = new Set<unknown>();
    filterTableValue.rows.forEach((row) => {
      const value = row.row[field.name] as unknown;
      values.add(value);
    });

    uniqueValues[field.name] = Array.from(values).map((value) => {
      let displayValue = "null";
      if (value !== null && value !== undefined) {
        displayValue =
          typeof value === "object" ? JSON.stringify(value) : value.toString(); // eslint-disable-line @typescript-eslint/no-base-to-string
      }
      return { value, displayValue };
    });
  });

  return uniqueValues;
});

function buildFilterTable() {
  const indexTableValue = indexTable.value;
  const xColumnName = xColumn.value?.name;
  if (!indexTableValue) return undefined;
  const columnsExceptX = indexTableValue.schema.fields
    .filter((c) => c.name !== xColumnName)
    .map((c) => c.name);

  const filteredTable = indexTableValue.select(columnsExceptX);

  // Get unique combinations of index values
  const rows = filteredTable.toArray() as StructRowProxy[];
  const uniqueRowsMap = new Map<
    string,
    { row: StructRowProxy; index: number }
  >();
  rows.forEach((row, i) => {
    const key = JSON.stringify(row);
    if (!uniqueRowsMap.has(key)) {
      uniqueRowsMap.set(key, { row, index: i });
    }
  });
  const uniqueRows = Array.from(uniqueRowsMap.values()) as {
    row: StructRowProxy;
    index: number;
  }[];
  return {
    table: filteredTable,
    rows: uniqueRows,
    fields: filteredTable.schema.fields,
  };
}

// Generate filter rows based on individual column selections
function generateFilterFromIndividualSelections() {
  if (!filterTable.value || !isIndividualFilterMode.value) return null;

  const fieldNames = filterTable.value.fields.map((f) => f.name);
  const selectedValues = fieldNames.map(
    (fieldName) => individualColumnSelections.value[fieldName] ?? [],
  );

  // Check if any column has selections
  if (selectedValues.every((values) => values.length === 0)) return null;

  // Find all rows that match the selected values in each column
  const matchingRows = filterTable.value.rows.filter((row) => {
    return fieldNames.every((fieldName, idx) => {
      const selectedForColumn = selectedValues[idx];
      if (selectedForColumn!.length === 0) return true; // No filter for this column
      return selectedForColumn!.includes(row.row[fieldName]);
    });
  });

  // If we have matching rows, use the first one as filter
  // In future, we might want to support multiple selections
  if (matchingRows.length > 0) {
    return matchingRows[0];
  }

  return null;
}

// Watch for individual selections and update filter when in individual mode
watch(
  [isIndividualFilterMode, individualColumnSelections],
  () => {
    if (isIndividualFilterMode.value && filterTable.value) {
      const individualFilter = generateFilterFromIndividualSelections();
      if (individualFilter) {
        filter.value = individualFilter;
      } else {
        // If no individual filter, reset to first row
        filter.value = filterTable.value.rows[0];
      }
    } else if (!isIndividualFilterMode.value && filterTable.value) {
      // When switching back to combined view, ensure we have a valid filter
      if (
        !filter.value ||
        !filterTable.value.rows.some((r) => r.index === filter.value?.index)
      ) {
        filter.value = filterTable.value.rows[0];
      }
    }
  },
  { deep: true },
);

const data = shallowRef<LinePlotOptions>();
watchDebounced(
  [
    datasetDetail,
    series,
    filter,
    filterTable,
    selectedComplexView,
    isIndividualFilterMode,
    individualColumnSelections,
  ],
  async () => {
    data.value = await getNewData();
  },
  { debounce: 50 },
);
async function getNewData() {
  const detailValue = datasetDetail.value;
  let indexRow = filter.value;
  const indexTableValue = filterTable.value;
  const datasetId = props.datasetId;
  const xColumnValue = xColumn.value;
  const seriesValue = series.value;

  const columns = detailValue?.columns;
  const indexTable = indexTableValue?.table;

  // If in individual mode, try to generate filter from individual selections
  if (isIndividualFilterMode.value) {
    const individualFilter = generateFilterFromIndividualSelections();
    if (individualFilter) {
      indexRow = individualFilter;
    }
  }

  if (
    !columns ||
    !indexTable ||
    !((indexTable.numCols > 0 && indexRow) || indexTable.numCols == 0) ||
    !datasetId ||
    !seriesValue
  )
    return undefined;

  let indexFilters: string | undefined;
  if (indexTable.numCols > 0 && indexRow) {
    const indexRowTable = indexTable.slice(indexRow.index, indexRow.index + 1);
    const buf = tableToIPC(indexRowTable);
    indexFilters = btoa(String.fromCharCode(...buf));
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
  <div class="size-full flex flex-col">
    <div class="p-2 flex gap-2 flex-wrap">
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
    <div class="p-2 flex gap-2 items-center">
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
    <Splitter class="flex-1 min-h-0" layout="vertical">
      <SplitterPanel>
        <ChartWrapper :data />
      </SplitterPanel>
      <SplitterPanel class="flex flex-col">
        <div v-if="showFilterToggle" class="p-2 flex gap-2 items-center">
          <ToggleSwitch
            v-model="isIndividualFilterMode"
            input-id="individual-filter-switch"
          />
          <label for="individual-filter-switch">Split columns</label>
        </div>

        <!-- Combined view (default) -->
        <DataTable
          v-if="!isIndividualFilterMode"
          v-model:selection="filter"
          size="small"
          :value="filterTable?.rows"
          data-key="index"
          scrollable
          scroll-height="flex"
          selection-mode="single"
          meta-key-selection
          :virtual-scroller-options="{ itemSize: 35, lazy: true }"
        >
          <Column
            v-for="col in filterTable?.fields"
            :key="col.name"
            :field="(x) => x.row[col.name]"
            :header="col.name"
          />
        </DataTable>

        <!-- Individual columns view -->
        <div v-else-if="filterTable" class="flex flex-col h-full">
          <div class="flex flex-1 overflow-hidden">
            <template
              v-for="(field, index) in filterTable.fields"
              :key="field.name"
            >
              <div class="min-w-0 flex-1">
                <DataTable
                  :value="columnUniqueValues[field.name]"
                  :selection="
                    columnUniqueValues[field.name]?.filter((item) =>
                      individualColumnSelections[field.name]?.includes(
                        item.value,
                      ),
                    ) ?? []
                  "
                  data-key="value"
                  scrollable
                  scroll-height="flex"
                  selection-mode="multiple"
                  size="small"
                  :meta-key-selection="true"
                  :virtual-scroller-options="{ itemSize: 35, lazy: true }"
                  @update:selection="
                    (selection: ColumnValueOption[]) => {
                      individualColumnSelections[field.name] = selection.map(
                        (s) => s.value,
                      );
                    }
                  "
                >
                  <Column field="displayValue" :header="field.name" />
                </DataTable>
              </div>
              <Divider
                v-if="index < filterTable.fields.length - 1"
                layout="vertical"
              />
            </template>
          </div>
        </div>
      </SplitterPanel>
    </Splitter>
  </div>
</template>
