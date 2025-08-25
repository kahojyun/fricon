<script setup lang="ts">
import { Splitter, SplitterPanel } from "primevue";
import DatasetTable from "./DatasetTable.vue";
import ChartViewer from "./ChartViewer.vue";
</script>
<template>
  <Splitter class="size-full">
    <SplitterPanel>
      <DatasetTable />
    </SplitterPanel>
    <SplitterPanel>
      <ChartViewer />
    </SplitterPanel>
  </Splitter>
</template>
