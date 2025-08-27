<script setup lang="ts">
import { onMounted, onUnmounted, ref, type Ref } from "vue";
import { DataTable, Column, Tag } from "primevue";
import {
  listDatasets,
  onDatasetCreated,
  type DatasetInfo,
  type DatasetCreatedEvent,
} from "./backend";

const emit = defineEmits<{
  "dataset-selected": [dataset: DatasetInfo];
}>();

const value: Ref<DatasetInfo[]> = ref([]);
const selectedDataset = ref<DatasetInfo | null>(null);

let unsubscribe: (() => void) | null = null;

const loadDatasets = async () => {
  const datasets = await listDatasets();
  value.value = datasets;
};

const handleDatasetCreated = (event: DatasetCreatedEvent) => {
  // Add the new dataset to the list
  const newDataset: DatasetInfo = {
    id: event.id,
    name: event.name,
    description: event.description,
    tags: event.tags,
    created_at: new Date(),
  };
  value.value.unshift(newDataset);
};

const onSelectionChange = (event: { data: DatasetInfo }) => {
  selectedDataset.value = event.data;
  emit("dataset-selected", event.data);
};

onMounted(async () => {
  await loadDatasets();

  // Listen for dataset created events
  unsubscribe = await onDatasetCreated(handleDatasetCreated);
});

onUnmounted(() => {
  if (unsubscribe) {
    unsubscribe();
  }
});
</script>
<template>
  <DataTable
    v-model:selection="selectedDataset"
    :value="value"
    removable-sort
    selection-mode="single"
    @row-select="onSelectionChange"
  >
    <Column field="id" header="ID" />
    <Column field="name" header="Name" />
    <Column field="tags" header="Tags">
      <template #body="slotProps">
        <Tag
          v-for="(tag, index) in slotProps.data.tags"
          :key="index"
          class="mr-1 mb-1"
        >
          {{ tag }}
        </Tag>
      </template>
    </Column>
    <Column field="created_at" header="Created At" sortable>
      <template #body="slotProps">
        {{ slotProps.data.created_at.toLocaleString() }}
      </template>
    </Column>
  </DataTable>
</template>
