<script setup lang="ts">
import DatasetTable from "./DatasetTable.vue";
import ChartViewer from "./ChartViewer.vue";
import { ref } from "vue";

const datasetId = ref<number>();
</script>
<template>
  <Splitter class="size-full overflow-hidden select-none">
    <SplitterPanel>
      <DatasetTable @dataset-selected="(id) => (datasetId = id)" />
    </SplitterPanel>
    <SplitterPanel>
      <p v-if="datasetId == null">No dataset selected</p>
      <ChartViewer v-else :dataset-id />
    </SplitterPanel>
  </Splitter>
</template>
