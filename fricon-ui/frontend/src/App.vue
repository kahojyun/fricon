<script setup lang="ts">
import { ref, watch } from "vue";
import Splitter from "primevue/splitter";
import SplitterPanel from "primevue/splitterpanel";
import ToggleSwitch from "primevue/toggleswitch";
import ChartViewer from "./ChartViewer.vue";
import AppCredits from "./AppCredits.vue";
import DatasetTable from "./DatasetTable.vue";
import AppFolderPicker from "./AppFolderPicker.vue";

const showChart = ref(true);
const folderName = ref("");
watch(folderName, (newVal) => {
  console.log("Folder name changed:", newVal);
});
</script>

<template>
  <Splitter class="w-full h-full">
    <SplitterPanel>
      <DatasetTable />
    </SplitterPanel>
    <SplitterPanel class="flex flex-col items-center">
      <AppFolderPicker v-model="folderName" class="flex-none p-4" />
      <AppCredits class="flex-none" />
      <label class="flex-none flex items-center gap-4 p-4">
        Toggle Chart 开关
        <ToggleSwitch v-model="showChart" />
      </label>
      <ChartViewer v-if="showChart" class="flex-auto" />
    </SplitterPanel>
  </Splitter>
</template>

<style scoped></style>
