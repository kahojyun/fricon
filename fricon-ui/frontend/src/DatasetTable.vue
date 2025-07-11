<script setup lang="ts">
import { onMounted, ref } from "vue";
import { DataTable, Column } from "primevue";
import { getConnectionStatus } from "./backend";

const value = ref([
  { id: 1, name: "John Doe", age: 30 },
  { id: 2, name: "Jane Doe", age: 25 },
  { id: 3, name: "Bob Smith", age: 40 },
]);

const status = ref("");

onMounted(async () => {
  try {
    status.value = await getConnectionStatus();
  } catch (error) {
    console.error(error);
    status.value = "Error";
  }
});
</script>
<template>
  <p>{{ status }}</p>
  <DataTable :value="value" removable-sort>
    <Column field="id" header="ID" />
    <Column field="name" header="Name" />
    <Column field="age" header="Age" sortable />
  </DataTable>
</template>
