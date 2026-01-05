<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { StructRowProxy, Table } from "apache-arrow";
import {
  buildFilterTableData,
  getColumnUniqueValues,
  findMatchingRowFromSelections,
  type ColumnValueOption,
} from "../utils/filterTableUtils";

interface Props {
  indexTable: Table | undefined;
  xColumnName?: string;
  datasetId: string;
}

const props = defineProps<Props>();
const model = defineModel<{ row: StructRowProxy; index: number }>();

// Toggle for individual vs combined filter mode
const isIndividualFilterMode = ref(false);

// Store individual column selections when in individual mode
const individualColumnSelections = ref<Record<string, unknown[]>>({});

// Track previous dataset ID and x column name for preservation logic
const previousDatasetState = ref<{
  datasetId: string | undefined;
  xColumnName: string | undefined;
}>({
  datasetId: undefined,
  xColumnName: undefined,
});

const filterTable = computed(() => {
  return buildFilterTableData(props.indexTable, props.xColumnName);
});

// Check if we have multiple columns to show toggle button
const showFilterToggle = computed(() => {
  return filterTable.value && filterTable.value.fields.length > 1;
});

// Check if filter table is empty
const isFilterTableEmpty = computed(() => {
  return !filterTable.value || filterTable.value.rows.length === 0;
});

// Compute unique values for each column when in individual mode
const columnUniqueValues = computed<Record<string, ColumnValueOption[]>>(() => {
  if (!isIndividualFilterMode.value) return {};
  return getColumnUniqueValues(filterTable.value);
});

watch(
  filterTable,
  (newFilterTable) => {
    const datasetChanged =
      previousDatasetState.value.datasetId !== props.datasetId;
    const xColumnChanged =
      previousDatasetState.value.xColumnName !== props.xColumnName;
    const contextChanged = datasetChanged || xColumnChanged;

    if (!newFilterTable || newFilterTable.rows.length === 0) {
      model.value = undefined;
      if (contextChanged) {
        individualColumnSelections.value = {};
      }
      previousDatasetState.value = {
        datasetId: props.datasetId,
        xColumnName: props.xColumnName,
      };
      return;
    }

    if (contextChanged) {
      model.value = newFilterTable.rows[0];
      individualColumnSelections.value = {};
    } else {
      const currentSelection = model.value;
      if (currentSelection) {
        const preservedRow = newFilterTable.rows.find(
          (row) => row.index === currentSelection.index,
        );
        if (preservedRow) {
          model.value = preservedRow;
        } else {
          model.value = newFilterTable.rows[0];
          individualColumnSelections.value = {};
        }
      } else {
        model.value = newFilterTable.rows[0];
      }
    }

    previousDatasetState.value = {
      datasetId: props.datasetId,
      xColumnName: props.xColumnName,
    };
  },
  { immediate: true },
);

watch(
  [isIndividualFilterMode, individualColumnSelections],
  () => {
    if (isIndividualFilterMode.value && filterTable.value) {
      const individualFilter = findMatchingRowFromSelections(
        filterTable.value,
        individualColumnSelections.value,
      );
      if (individualFilter) {
        model.value = individualFilter;
      } else {
        model.value = filterTable.value.rows[0];
      }
    } else if (!isIndividualFilterMode.value && filterTable.value) {
      const currentFilter = model.value;
      if (
        !currentFilter ||
        !filterTable.value.rows.some((r) => r.index === currentFilter.index)
      ) {
        model.value = filterTable.value.rows[0];
      }
    }
  },
  { deep: true },
);
</script>

<template>
  <div class="flex h-full flex-col">
    <div v-if="showFilterToggle" class="flex items-center gap-2 p-2">
      <ToggleSwitch
        v-model="isIndividualFilterMode"
        input-id="individual-filter-switch"
      />
      <label for="individual-filter-switch">Split columns</label>
    </div>

    <div
      v-if="!isIndividualFilterMode && isFilterTableEmpty"
      class="text-color-secondary flex h-full items-center justify-center text-sm"
    >
      No data available
    </div>
    <DataTable
      v-else-if="!isIndividualFilterMode"
      :value="filterTable?.rows"
      :selection="model"
      size="small"
      data-key="index"
      scrollable
      scroll-height="flex"
      selection-mode="single"
      meta-key-selection
      :virtual-scroller-options="{ itemSize: 35, lazy: true }"
      @update:selection="
        model = $event as { row: StructRowProxy; index: number } | undefined
      "
    >
      <Column
        v-for="col in filterTable?.fields"
        :key="col.name"
        :field="(x: { row: StructRowProxy; index: number }) => x.row[col.name]"
        :header="col.name"
      />
    </DataTable>

    <div
      v-if="isIndividualFilterMode && isFilterTableEmpty"
      class="text-color-secondary flex h-full items-center justify-center text-sm"
    >
      No data available
    </div>
    <div
      v-else-if="isIndividualFilterMode && filterTable"
      class="flex h-full flex-col"
    >
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
                  individualColumnSelections[field.name]?.includes(item.value),
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
  </div>
</template>
