<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import Button from "primevue/button";
import { getWorkspaceInfo } from "@/backend.ts";

// Try to read workspace path from Tauri (if available at runtime)
const workspacePath = ref<string>("(no workspace)");

const router = useRouter();
const route = useRoute();

function goTo(name: string) {
  void router.push({ name });
}

onMounted(async () => {
  workspacePath.value = (await getWorkspaceInfo()).path;
});
</script>

<template>
  <div class="h-full flex flex-col">
    <div class="flex flex-1 overflow-hidden">
      <!-- Left sidebar (like VS Code) -->
      <aside class="w-14 bg-surface-700 flex flex-col items-center py-2 gap-2">
        <Button
          icon="pi pi-database"
          :aria-label="'Data'"
          :outlined="route.name !== 'data'"
          @click="goTo('data')"
        />
        <Button
          icon="pi pi-info-circle"
          :aria-label="'Credits'"
          :outlined="route.name !== 'credits'"
          @click="goTo('credits')"
        />
      </aside>

      <!-- Main content -->
      <main class="flex-1 relative overflow-hidden">
        <router-view />
      </main>
    </div>

    <!-- Bottom status bar -->
    <footer class="h-8 bg-surface-800 text-sm px-3 flex items-center">
      <div class="truncate">Workspace: {{ workspacePath }}</div>
    </footer>
  </div>
</template>
