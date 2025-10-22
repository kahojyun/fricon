<script setup lang="ts">
import { Splitter, SplitterPanel } from "primevue";
import DatasetTable from "./DatasetTable.vue";
import ChartViewer from "./ChartViewer.vue";
import { ref } from "vue";

const datasetId = ref<number | null>(null);
</script>
<template>
  <Splitter class="size-full select-none">
    <SplitterPanel>
      <DatasetTable @dataset-selected="(id) => (datasetId = id)" />
    </SplitterPanel>
    <SplitterPanel>
      <ChartViewer :dataset-id />
    </SplitterPanel>
  </Splitter>
</template>
