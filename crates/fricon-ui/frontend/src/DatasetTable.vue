<script setup lang="ts">
import { onMounted, ref, type Ref } from "vue";
import { DataTable, Column } from "primevue";
import { listDatasets, type DatasetInfo } from "./backend";

const value: Ref<DatasetInfo[]> = ref([]);

onMounted(async () => {
  const datasets = await listDatasets();
  value.value = datasets;
});
</script>
<template>
  <DataTable :value="value" removable-sort>
    <Column field="id" header="ID" />
    <Column field="name" header="Name" />
    <Column field="description" header="Description" />
    <Column field="tags" header="Tags">
      <template #body="slotProps">
        <span
          v-for="(tag, index) in slotProps.data.tags"
          :key="index"
          class="inline-block bg-primary-100 text-primary-800 text-xs font-medium px-2 py-1 rounded-full mr-1 mb-1"
        >
          {{ tag }}
        </span>
      </template>
    </Column>
    <Column field="created_at" header="Created At" sortable>
      <template #body="slotProps">
        {{ slotProps.data.created_at.toLocaleString() }}
      </template>
    </Column>
  </DataTable>
</template>
