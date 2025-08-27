<script setup lang="ts">
import { ref } from "vue";
import { Splitter, SplitterPanel } from "primevue";
import DatasetTable from "./DatasetTable.vue";
import ChartViewer from "./ChartViewer.vue";
import type { DatasetInfo } from "./backend";

const selectedDataset = ref<DatasetInfo | null>(null);

function onDatasetSelected(dataset: DatasetInfo) {
  selectedDataset.value = dataset;
}
</script>
<template>
  <Splitter class="size-full select-none">
    <SplitterPanel>
      <DatasetTable @dataset-selected="onDatasetSelected" />
    </SplitterPanel>
    <SplitterPanel>
      <ChartViewer :dataset-id="selectedDataset?.id" />
    </SplitterPanel>
  </Splitter>
</template>
