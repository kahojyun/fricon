<script setup lang="ts">
import { ref } from "vue";
import { DataTable, Column } from "primevue";

const value = ref([
  { id: 1, name: "John Doe", age: 30 },
  { id: 2, name: "Jane Doe", age: 25 },
  { id: 3, name: "Bob Smith", age: 40 },
]);
</script>
<template>
  <DataTable :value="value" removable-sort>
    <Column field="id" header="ID" />
    <Column field="name" header="Name" />
    <Column field="age" header="Age" sortable />
  </DataTable>
</template>
