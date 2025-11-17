<script setup lang="ts">
import { computed, onWatcherCleanup, ref, shallowRef, watch } from "vue";
import {
  type ColumnInfo,
  type DatasetDetail,
  datasetDetail,
  fetchData,
} from "@/backend.ts";
import { type StructRowProxy, type Table, tableToIPC } from "apache-arrow";
import { watchThrottled, computedAsync } from "@vueuse/core";
import type { TypedArray } from "apache-arrow/interfaces";
import ChartWrapper from "./components/ChartWrapper.vue";

const props = defineProps<{
  datasetId: number;
}>();

const detail = shallowRef<DatasetDetail | null>(null);
const indexTable = shallowRef<Table | null>(null);
watchThrottled(
  () => props.datasetId,
  async (newId) => {
    let aborted = false;
    onWatcherCleanup(() => (aborted = true));

    const newDetail = await datasetDetail(newId);
    if (aborted) return;

    const index_columns = newDetail.columns.reduce((acc, c, i) => {
      if (c.isIndex) acc.push(i);
      return acc;
    }, [] as number[]);
    const newIndexTable = await fetchData(newId, {
      columns: index_columns,
    });
    if (aborted) return;

    detail.value = newDetail;
    indexTable.value = newIndexTable;
    selectedRowIndex.value = undefined;
  },
  { throttle: 100, immediate: true },
);
const nonIndexColumns = computed(
  () => detail.value?.columns.filter((c) => !c.isIndex) ?? [],
);
function updateSelection(
  newOptions: ColumnInfo[],
  currentOption: ColumnInfo | undefined,
): ColumnInfo | undefined {
  const currentName = currentOption?.name;
  return newOptions.find((col) => col.name === currentName) ?? newOptions[0];
}

const selectedSeries = ref<ColumnInfo>();
watch(nonIndexColumns, (newOptions) => {
  selectedSeries.value = updateSelection(newOptions, selectedSeries.value);
});

const chartType = ref("line");
const chartTypes = ref([
  { name: "Line", value: "line" },
  { name: "Scatter", value: "scatter" },
  { name: "Heatmap", value: "heatmap" },
]);

const xColumn = ref<ColumnInfo>();
const xColumnOptions = computed(
  () => detail.value?.columns.filter((c) => c.isIndex) ?? [],
);
watch(xColumnOptions, (newOptions) => {
  xColumn.value = updateSelection(newOptions, xColumn.value);
});

const indexSelectionTable = computed(() => {
  const indexTableValue = indexTable.value;
  const xColumnName = xColumn.value?.name;
  if (!indexTableValue) return null;
  const columnsExceptX = indexTableValue.schema.fields
    .filter((c) => c.name !== xColumnName)
    .map((c) => c.name);

  const filteredTable = indexTableValue.select(columnsExceptX);

  // Get unique combinations of index values
  const rows = filteredTable.toArray() as StructRowProxy[];
  const uniqueRows = Array.from(
    new Map(
      rows.map((row, index) => [JSON.stringify(row), { row, index }]),
    ).values(),
  );

  return {
    table: filteredTable,
    rows: uniqueRows,
    fields: filteredTable.schema.fields,
  };
});
const selectedRowIndex = shallowRef<{ row: StructRowProxy; index: number }>();
const data = computedAsync(async () => {
  const detailValue = detail.value;
  const indexRow = selectedRowIndex.value;
  const indexTableValue = indexSelectionTable.value;
  const datasetId = props.datasetId;
  const xColumnValue = xColumn.value;
  const yColumn = selectedSeries.value;

  const columns = detailValue?.columns;
  const indexTable = indexTableValue?.table;

  if (
    !columns ||
    !indexTable ||
    !((indexTable.numCols > 0 && indexRow) || indexTable.numCols == 0) ||
    !datasetId ||
    !xColumnValue ||
    !yColumn
  )
    return undefined;

  const xIndex = columns.findIndex((c) => c.name === xColumnValue.name);
  const yIndex = columns.findIndex((c) => c.name === yColumn.name);
  let newData: Table;
  if (indexTable.numCols > 0 && indexRow) {
    const indexRowTable = indexTable.slice(indexRow.index, indexRow.index + 1);
    const buf = tableToIPC(indexRowTable);
    const buf_base64 = btoa(String.fromCharCode(...buf));
    newData = await fetchData(datasetId, {
      indexFilters: buf_base64,
      columns: [xIndex, yIndex],
    });
  } else {
    newData = await fetchData(datasetId, {
      columns: [xIndex, yIndex],
    });
  }
  const x = newData.getChildAt(0)!.toArray() as TypedArray;
  const y = newData.getChildAt(1)!.toArray() as TypedArray;
  return {
    x,
    xName: xColumnValue.name,
    series: [{ name: yColumn.name, data: y }],
  };
}, undefined);
</script>

<template>
  <div class="size-full flex flex-col">
    <div class="p-2 flex">
      <Select
        v-model="chartType"
        :options="chartTypes"
        option-label="name"
        option-value="value"
        placeholder="Select a Chart Type"
        class="mr-2"
      />
      <IftaLabel>
        <Select
          v-model="selectedSeries"
          :options="nonIndexColumns"
          option-label="name"
          input-id="main-series-select"
          fluid
        />
        <label for="main-series-select">Select Series</label>
      </IftaLabel>
      <IftaLabel>
        <Select
          v-model="xColumn"
          :options="xColumnOptions"
          option-label="name"
          input-id="x-column-select"
          fluid
        />
        <label for="x-column-select">X</label>
      </IftaLabel>
    </div>
    <Splitter class="flex-1 min-h-0" layout="vertical">
      <SplitterPanel>
        <ChartWrapper :data />
      </SplitterPanel>
      <SplitterPanel>
        <DataTable
          v-model:selection="selectedRowIndex"
          size="small"
          :value="indexSelectionTable?.rows"
          data-key="index"
          scrollable
          scroll-height="flex"
          selection-mode="single"
        >
          <Column
            v-for="col in indexSelectionTable?.fields"
            :key="col.name"
            :field="(x) => x.row[col.name]"
            :header="col.name"
          />
        </DataTable>
      </SplitterPanel>
    </Splitter>
  </div>
</template>
