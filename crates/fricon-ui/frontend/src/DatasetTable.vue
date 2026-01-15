<script setup lang="ts">
import { onMounted, onUnmounted, ref, shallowRef } from "vue";
import { type DatasetInfo, listDatasets, onDatasetCreated } from "./backend";

const emit = defineEmits<{
  datasetSelected: [id: number];
}>();
const datasets = ref<DatasetInfo[]>([]);
const selectedDataset = shallowRef<DatasetInfo>();

let unsubscribe: (() => void) | null = null;

const loadDatasets = async () => {
  datasets.value = await listDatasets();
};

const handleDatasetCreated = (event: DatasetInfo) => {
  datasets.value.unshift(event);
};

onMounted(async () => {
  await loadDatasets();

  // Listen for dataset created events
  unsubscribe = await onDatasetCreated(handleDatasetCreated);
});

onUnmounted(() => {
  unsubscribe?.();
});

function handleKeydown(event: KeyboardEvent) {
  if (event.metaKey || event.ctrlKey) {
    event.stopPropagation();
  }
}
</script>
<template>
  <DataTable
    v-model:selection="selectedDataset"
    :value="datasets"
    size="small"
    data-key="id"
    selection-mode="single"
    removable-sort
    scrollable
    scroll-height="flex"
    @keydown.capture="handleKeydown"
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
