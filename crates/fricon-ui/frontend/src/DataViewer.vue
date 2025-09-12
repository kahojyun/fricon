<script setup lang="ts">
import { ref, type Ref } from "vue";
import { Splitter, SplitterPanel } from "primevue";
import DatasetTable from "./DatasetTable.vue";
import PlotConfigViewer from "./PlotConfigViewer.vue";
import type { DatasetInfo } from "./backend";

const selectedDataset: Ref<DatasetInfo | null> = ref(null);
</script>

<template>
  <Splitter class="size-full select-none">
    <SplitterPanel :size="50">
      <DatasetTable v-model:selected-dataset="selectedDataset" />
    </SplitterPanel>
    <SplitterPanel :size="50">
      <PlotConfigViewer :dataset-id="selectedDataset?.id ?? null" />
    </SplitterPanel>
  </Splitter>
</template>
