<script setup lang="ts">
import DatasetTable from "./DatasetTable.vue";
import DatasetDetailPage from "./DatasetDetailPage.vue";
import { ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";

const datasetId = ref<number>();
const datasetTableKey = ref(0);
const route = useRoute();
const router = useRouter();

watch(
  () => route.params.id,
  (idParam) => {
    const parsed =
      typeof idParam === "string" && idParam.trim()
        ? Number.parseInt(idParam, 10)
        : Number.NaN;
    datasetId.value = Number.isFinite(parsed) ? parsed : undefined;
  },
  { immediate: true },
);

const handleDatasetSelected = (id: number) => {
  datasetId.value = id;
  if (route.params.id !== String(id)) {
    void router.push({ name: "dataset", params: { id } });
  }
};

const handleDatasetUpdated = () => {
  datasetTableKey.value += 1;
};
</script>
<template>
  <Splitter class="size-full overflow-hidden select-none">
    <SplitterPanel>
      <DatasetTable
        :key="datasetTableKey"
        :selected-dataset-id="datasetId"
        @dataset-selected="handleDatasetSelected"
      />
    </SplitterPanel>
    <SplitterPanel>
      <p v-if="datasetId == null">No dataset selected</p>
      <DatasetDetailPage
        v-else
        :dataset-id="datasetId"
        @dataset-updated="handleDatasetUpdated"
      />
    </SplitterPanel>
  </Splitter>
</template>
