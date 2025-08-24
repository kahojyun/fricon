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
    <Column field="tags" header="Tags" />
    <Column field="created_at" header="Created At" sortable />
  </DataTable>
</template>
