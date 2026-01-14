<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type {
  FilterTableData,
  FilterTableRow,
  ColumnUniqueValue,
} from "@/backend";

interface Props {
  filterTableData: FilterTableData | undefined;
  datasetId: string;
}

const props = defineProps<Props>();
const model = defineModel<FilterTableRow>();

// Toggle for individual vs combined filter mode
const isIndividualFilterMode = ref(false);

// Store individual column selections when in individual mode
const individualColumnSelections = ref<Record<string, unknown[]>>({});

// Track previous dataset ID for preservation logic
const previousDatasetId = ref<string | undefined>(undefined);

// Check if we have multiple columns to show toggle button
const showFilterToggle = computed(() => {
  return props.filterTableData && props.filterTableData.fields.length > 1;
});

// Check if filter table is empty
const isFilterTableEmpty = computed(() => {
  return !props.filterTableData || props.filterTableData.rows.length === 0;
});

// Get column unique values from FilterTableData (already computed by backend)
const columnUniqueValues = computed<Record<string, ColumnUniqueValue[]>>(() => {
  if (!isIndividualFilterMode.value || !props.filterTableData) return {};
  return props.filterTableData.columnUniqueValues;
});

// Helper to get row value by field index
function getRowValue(row: FilterTableRow, fieldIndex: number): unknown {
  return row.values[fieldIndex];
}

// Find matching row from individual column selections
function findMatchingRowFromSelections(
  filterTableData: FilterTableData,
  selections: Record<string, unknown[]>,
): FilterTableRow | null {
  const fieldNames = filterTableData.fields;
  const selectedValues = fieldNames.map(
    (fieldName) => selections[fieldName] ?? [],
  );

  if (selectedValues.every((values) => values.length === 0)) return null;

  const matchingRows = filterTableData.rows.filter((row) => {
    return fieldNames.every((fieldName, idx) => {
      const selectedForColumn = selectedValues[idx];
      if (selectedForColumn!.length === 0) return true;
      const rowValue = row.values[idx];
      return selectedForColumn!.some(
        (sel) => JSON.stringify(sel) === JSON.stringify(rowValue),
      );
    });
  });

  if (matchingRows.length > 0) {
    return matchingRows[0]!;
  }

  return null;
}

watch(
  () => props.filterTableData,
  (newFilterTableData) => {
    const datasetChanged = previousDatasetId.value !== props.datasetId;

    if (!newFilterTableData || newFilterTableData.rows.length === 0) {
      model.value = undefined;
      if (datasetChanged) {
        individualColumnSelections.value = {};
      }
      previousDatasetId.value = props.datasetId;
      return;
    }

    if (datasetChanged) {
      model.value = newFilterTableData.rows[0];
      individualColumnSelections.value = {};
    } else {
      const currentSelection = model.value;
      if (currentSelection) {
        const preservedRow = newFilterTableData.rows.find(
          (row) => row.index === currentSelection.index,
        );
        if (preservedRow) {
          model.value = preservedRow;
        } else {
          model.value = newFilterTableData.rows[0];
          individualColumnSelections.value = {};
        }
      } else {
        model.value = newFilterTableData.rows[0];
      }
    }

    previousDatasetId.value = props.datasetId;
  },
  { immediate: true },
);

watch(
  [isIndividualFilterMode, individualColumnSelections],
  () => {
    if (isIndividualFilterMode.value && props.filterTableData) {
      const individualFilter = findMatchingRowFromSelections(
        props.filterTableData,
        individualColumnSelections.value,
      );
      if (individualFilter) {
        model.value = individualFilter;
      } else {
        model.value = props.filterTableData.rows[0];
      }
    } else if (!isIndividualFilterMode.value && props.filterTableData) {
      const currentFilter = model.value;
      if (
        !currentFilter ||
        !props.filterTableData.rows.some((r) => r.index === currentFilter.index)
      ) {
        model.value = props.filterTableData.rows[0];
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
      :value="filterTableData?.rows"
      :selection="model"
      size="small"
      data-key="index"
      scrollable
      scroll-height="flex"
      selection-mode="single"
      meta-key-selection
      :virtual-scroller-options="{ itemSize: 35, lazy: true }"
      @update:selection="model = $event as FilterTableRow | undefined"
    >
      <Column
        v-for="(field, fieldIndex) in filterTableData?.fields"
        :key="field"
        :field="(x: FilterTableRow) => String(getRowValue(x, fieldIndex))"
        :header="field"
      />
    </DataTable>

    <div
      v-if="isIndividualFilterMode && isFilterTableEmpty"
      class="text-color-secondary flex h-full items-center justify-center text-sm"
    >
      No data available
    </div>
    <div
      v-else-if="isIndividualFilterMode && filterTableData"
      class="flex h-full flex-col"
    >
      <div class="flex flex-1 overflow-hidden">
        <template v-for="(field, index) in filterTableData.fields" :key="field">
          <div class="min-w-0 flex-1">
            <DataTable
              :value="columnUniqueValues[field]"
              :selection="
                columnUniqueValues[field]?.filter((item) =>
                  individualColumnSelections[field]?.some(
                    (sel) => JSON.stringify(sel) === JSON.stringify(item.value),
                  ),
                ) ?? []
              "
              data-key="displayValue"
              scrollable
              scroll-height="flex"
              selection-mode="multiple"
              size="small"
              :meta-key-selection="true"
              :virtual-scroller-options="{ itemSize: 35, lazy: true }"
              @update:selection="
                (selection: ColumnUniqueValue[]) => {
                  individualColumnSelections[field] = selection.map(
                    (s) => s.value,
                  );
                }
              "
            >
              <Column field="displayValue" :header="field" />
            </DataTable>
          </div>
          <Divider
            v-if="index < filterTableData.fields.length - 1"
            layout="vertical"
          />
        </template>
      </div>
    </div>
  </div>
</template>
