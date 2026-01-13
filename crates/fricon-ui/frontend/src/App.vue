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
  <div class="flex h-full flex-col">
    <div class="flex flex-1 overflow-hidden">
      <!-- Left sidebar (like VS Code) -->
      <aside class="bg-surface-700 flex w-14 flex-col items-center gap-2 py-2">
        <AppLink to="/" icon="pi pi-database" label="Data" />
        <AppLink to="/credits" icon="pi pi-info-circle" label="Credits" />
      </aside>

      <!-- Main content -->
      <main class="relative flex-1 overflow-y-auto">
        <RouterView />
      </main>
    </div>

    <!-- Bottom status bar -->
    <footer class="bg-surface-800 flex h-8 items-center px-3 text-sm">
      <div class="truncate">Workspace: {{ workspacePath }}</div>
    </footer>
  </div>
</template>
