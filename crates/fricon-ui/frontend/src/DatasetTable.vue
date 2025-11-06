<script setup lang="ts">
import { onMounted, onUnmounted, ref, type Ref } from "vue";
import { Column, DataTable, Tag } from "primevue";
import {
  type DatasetCreatedEvent,
  type DatasetInfo,
  listDatasets,
  onDatasetCreated,
} from "./backend";

const emit = defineEmits<{
  datasetSelected: [id: number];
}>();
const value: Ref<DatasetInfo[]> = ref([]);

let unsubscribe: (() => void) | null = null;

const loadDatasets = async () => {
  value.value = await listDatasets();
};

const handleDatasetCreated = (event: DatasetCreatedEvent) => {
  // Add the new dataset to the list
  const newDataset: DatasetInfo = {
    id: event.id,
    name: event.name,
    description: event.description,
    tags: event.tags,
    createdAt: new Date(),
  };
  value.value.unshift(newDataset);
};

onMounted(async () => {
  await loadDatasets();

  // Listen for dataset created events
  unsubscribe = await onDatasetCreated(handleDatasetCreated);
});

onUnmounted(() => {
  unsubscribe?.();
});
</script>
<template>
  <DataTable
    :value="value"
    size="small"
    data-key="id"
    selection-mode="single"
    removable-sort
    scrollable
    scroll-height="flex"
    @row-select="emit('datasetSelected', $event.data.id)"
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
    <Column field="createdAt" header="Created At" sortable>
      <template #body="slotProps">
        {{ slotProps.data.createdAt.toLocaleString() }}
      </template>
    </Column>
  </DataTable>
</template>
