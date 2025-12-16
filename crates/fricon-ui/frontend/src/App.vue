<script setup lang="ts">
import { onMounted, ref } from "vue";
import { getWorkspaceInfo } from "@/backend.ts";
import AppLink from "@/components/AppLink.vue";

// Try to read workspace path from Tauri (if available at runtime)
const workspacePath = ref<string>("(no workspace)");

onMounted(async () => {
  workspacePath.value = (await getWorkspaceInfo()).path;
});
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex flex-1 overflow-hidden">
      <!-- Left sidebar (like VS Code) -->
      <aside class="w-14 bg-surface-700 flex flex-col items-center py-2 gap-2">
        <AppLink to="/" icon="pi pi-database" label="Data" />
        <AppLink to="/credits" icon="pi pi-info-circle" label="Credits" />
      </aside>

      <!-- Main content -->
      <main class="flex-1 relative overflow-hidden">
        <RouterView />
      </main>
    </div>

    <!-- Bottom status bar -->
    <footer class="h-8 bg-surface-800 text-sm px-3 flex items-center">
      <div class="truncate">Workspace: {{ workspacePath }}</div>
    </footer>
  </div>
</template>
