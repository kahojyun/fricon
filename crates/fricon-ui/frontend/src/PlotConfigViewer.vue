<script setup lang="ts">
import { ref, watch } from "vue";
import { Card, DataTable, Column } from "primevue";
import { getDatasetPlotConfig, type DatasetPlotConfig } from "./backend";

const props = defineProps<{
  datasetId: number | null;
}>();

const plotConfig = ref<DatasetPlotConfig | null>(null);
const loading = ref(false);
const error = ref<string | null>(null);

const loadPlotConfig = async () => {
  if (props.datasetId === null) {
    plotConfig.value = null;
    return;
  }

  loading.value = true;
  error.value = null;

  try {
    plotConfig.value = await getDatasetPlotConfig(props.datasetId);
  } catch (err) {
    error.value =
      err instanceof Error ? err.message : "Failed to load plot configuration";
    plotConfig.value = null;
  } finally {
    loading.value = false;
  }
};

watch(() => props.datasetId, loadPlotConfig, { immediate: true });
</script>

<template>
  <div class="h-full w-full flex flex-col">
    <div v-if="loading" class="flex items-center justify-center h-full">
      <i class="pi pi-spin pi-spinner" style="font-size: 2rem"></i>
    </div>

    <div v-else-if="error" class="flex items-center justify-center h-full">
      <div class="text-red-500">{{ error }}</div>
    </div>

    <div v-else-if="plotConfig" class="flex flex-col h-full">
      <Card class="mb-4">
        <template #title>
          <div class="flex items-center">
            <i class="pi pi-chart-bar mr-2"></i>
            <span>Plot Configuration for {{ plotConfig.dataset_name }}</span>
          </div>
        </template>
        <template #content>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <h3 class="font-bold mb-2">Dataset Settings</h3>
              <div v-if="Object.keys(plotConfig.settings).length > 0">
                <div
                  v-for="(value, key) in plotConfig.settings"
                  :key="key"
                  class="mb-1"
                >
                  <span class="font-semibold">{{ key }}:</span> {{ value }}
                </div>
              </div>
              <div v-else class="text-gray-500">No dataset settings</div>
            </div>

            <div>
              <h3 class="font-bold mb-2">Multi-Index Information</h3>
              <div v-if="plotConfig.multi_index">
                <div class="mb-1">
                  <span class="font-semibold">Level Indices:</span>
                  {{ plotConfig.multi_index.level_indices.join(", ") }}
                </div>
                <div class="mb-1">
                  <span class="font-semibold">Level Names:</span>
                  {{ plotConfig.multi_index.level_names.join(", ") }}
                </div>
                <div class="mb-1">
                  <span class="font-semibold">Deepest Level Column:</span>
                  {{
                    plotConfig.multi_index.deepest_level_col !== null
                      ? plotConfig.multi_index.deepest_level_col
                      : "None"
                  }}
                </div>
              </div>
              <div v-else class="text-gray-500">No multi-index information</div>
            </div>
          </div>
        </template>
      </Card>

      <Card class="flex-grow">
        <template #title>
          <div class="flex items-center">
            <i class="pi pi-table mr-2"></i>
            <span>Column Configurations</span>
          </div>
        </template>
        <template #content>
          <DataTable
            :value="plotConfig.columns"
            striped-rows
            table-style="min-width: 50rem"
            class="h-full"
          >
            <Column field="name" header="Name" />
            <Column field="data_type" header="Data Type" />
            <Column field="can_be_x_axis" header="Can be X-Axis">
              <template #body="slotProps">
                <i
                  :class="
                    slotProps.data.can_be_x_axis
                      ? 'pi pi-check text-green-500'
                      : 'pi pi-times text-red-500'
                  "
                ></i>
              </template>
            </Column>
            <Column field="can_be_y_axis" header="Can be Y-Axis">
              <template #body="slotProps">
                <i
                  :class="
                    slotProps.data.can_be_y_axis
                      ? 'pi pi-check text-green-500'
                      : 'pi pi-times text-red-500'
                  "
                ></i>
              </template>
            </Column>
            <Column field="suggested_plot_types" header="Suggested Plot Types">
              <template #body="slotProps">
                <span
                  v-for="(type, index) in slotProps.data.suggested_plot_types"
                  :key="index"
                  class="inline-block bg-blue-100 text-blue-800 text-xs font-medium mr-2 px-2.5 py-0.5 rounded"
                >
                  {{ type }}
                </span>
              </template>
            </Column>
            <Column header="Settings">
              <template #body="slotProps">
                <div
                  v-for="(value, key) in slotProps.data.settings"
                  :key="key"
                  class="text-xs"
                >
                  <span class="font-semibold">{{ key }}:</span> {{ value }}
                </div>
              </template>
            </Column>
          </DataTable>
        </template>
      </Card>
    </div>

    <div v-else class="flex items-center justify-center h-full text-gray-500">
      Select a dataset to view its plot configuration
    </div>
  </div>
</template>
