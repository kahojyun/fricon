<script setup lang="ts">
import { onMounted, ref } from "vue";
import DataViewer from "./DataViewer.vue";
import {
  getConnectionStatus,
  selectWorkspace as selectWorkspaceFromBackend,
} from "./backend";
import { Button } from "primevue";

enum WorkspaceStatus {
  LOADING,
  NO_SELECTION,
  SELECTED,
}

const workspaceStatus = ref(WorkspaceStatus.LOADING);

onMounted(async () => {
  const status = await getConnectionStatus();
  workspaceStatus.value =
    status === "connected"
      ? WorkspaceStatus.SELECTED
      : WorkspaceStatus.NO_SELECTION;
});

async function selectWorkspace() {
  try {
    await selectWorkspaceFromBackend();
    workspaceStatus.value = WorkspaceStatus.SELECTED;
  } catch (error) {
    console.error("Error selecting workspace:", error);
  }
}
</script>

<template>
  <div v-if="workspaceStatus === WorkspaceStatus.LOADING">Loading...</div>
  <div
    v-else-if="workspaceStatus === WorkspaceStatus.NO_SELECTION"
    class="flex flex-col gap-4 justify-center items-center h-full"
  >
    <Button label="Select workspace folder" @click="selectWorkspace" />
  </div>
  <DataViewer v-else-if="workspaceStatus === WorkspaceStatus.SELECTED" />
</template>
