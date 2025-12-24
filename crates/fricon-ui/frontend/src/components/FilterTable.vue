<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { StructRowProxy, Table } from "apache-arrow";

interface Props {
  indexTable: Table | undefined;
  xColumnName?: string;
}

interface ColumnValueOption {
  value: unknown;
  displayValue: string;
}

const props = defineProps<Props>();
const model = defineModel<{ row: StructRowProxy; index: number }>();

// Toggle for individual vs combined filter mode
const isIndividualFilterMode = ref(false);

// Store individual column selections when in individual mode
const individualColumnSelections = ref<Record<string, unknown[]>>({});

const filterTable = computed(() => buildFilterTable());

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
  const indexTableValue = props.indexTable;
  const xColumnName = props.xColumnName;
  if (!indexTableValue) return undefined;
  const columnsExceptX = indexTableValue.schema.fields
    .filter((c) => c.name !== xColumnName)
    .map((c) => c.name);

  const filteredTable = indexTableValue.select(columnsExceptX);

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

function generateFilterFromIndividualSelections() {
  if (!filterTable.value || !isIndividualFilterMode.value) return null;

  const fieldNames = filterTable.value.fields.map((f) => f.name);
  const selectedValues = fieldNames.map(
    (fieldName) => individualColumnSelections.value[fieldName] ?? [],
  );

  if (selectedValues.every((values) => values.length === 0)) return null;

  const matchingRows = filterTable.value.rows.filter((row) => {
    return fieldNames.every((fieldName, idx) => {
      const selectedForColumn = selectedValues[idx];
      if (selectedForColumn!.length === 0) return true;
      return selectedForColumn!.includes(row.row[fieldName]);
    });
  });

  if (matchingRows.length > 0) {
    return matchingRows[0];
  }

  return null;
}

watch(filterTable, () => {
  model.value = filterTable.value?.rows[0];
  individualColumnSelections.value = {};
});

watch(
  [isIndividualFilterMode, individualColumnSelections],
  () => {
    if (isIndividualFilterMode.value && filterTable.value) {
      const individualFilter = generateFilterFromIndividualSelections();
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
  <div class="flex flex-col h-full">
    <div v-if="showFilterToggle" class="p-2 flex gap-2 items-center">
      <ToggleSwitch
        v-model="isIndividualFilterMode"
        input-id="individual-filter-switch"
      />
      <label for="individual-filter-switch">Split columns</label>
    </div>

    <div
      v-if="!isIndividualFilterMode && isFilterTableEmpty"
      class="flex items-center justify-center h-full text-sm text-color-secondary"
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
      class="flex items-center justify-center h-full text-sm text-color-secondary"
    >
      No data available
    </div>
    <div
      v-else-if="isIndividualFilterMode && filterTable"
      class="flex flex-col h-full"
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
