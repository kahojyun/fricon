<script setup lang="ts">
import { onMounted, ref } from "vue";
import DataViewer from "./DataViewer.vue";
import { getWorkspaceInfo, type WorkspaceInfo } from "./backend";

const workspaceInfo = ref<WorkspaceInfo | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    workspaceInfo.value = await getWorkspaceInfo();
  } catch (err) {
    error.value = `Failed to get workspace info: ${String(err)}`;
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div v-if="loading" class="flex justify-center items-center h-full">
    <div>Loading workspace...</div>
  </div>
  <div
    v-else-if="error"
    class="flex flex-col gap-4 justify-center items-center h-full"
  >
    <div class="text-red-500">{{ error }}</div>
  </div>
  <div v-else class="h-full flex flex-col">
    <div class="p-4">
      <h1 class="text-lg font-semibold">Fricon Workspace</h1>
      <p class="text-sm">{{ workspaceInfo?.path }}</p>
    </div>
    <DataViewer />
  </div>
</template>
