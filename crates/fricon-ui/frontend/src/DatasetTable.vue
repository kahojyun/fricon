<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from "vue";
import {
  type DatasetInfo,
  type DatasetStatus,
  DATASET_PAGE_SIZE,
  listDatasets,
  onDatasetCreated,
  updateDatasetFavorite,
} from "./backend";
import { useRouter } from "vue-router";

const props = defineProps<{
  selectedDatasetId?: number;
}>();
const emit = defineEmits<{
  datasetSelected: [id: number];
}>();
const datasets = ref<DatasetInfo[]>([]);
const selectedDataset = shallowRef<DatasetInfo>();
const favoritesOnly = ref(false);
const searchQuery = ref("");
const selectedTags = ref<string[]>([]);
const isLoading = ref(false);
const router = useRouter();

const syncSelectedDataset = () => {
  if (props.selectedDatasetId == null) {
    selectedDataset.value = undefined;
    return;
  }
  selectedDataset.value = datasets.value.find(
    (dataset) => dataset.id === props.selectedDatasetId,
  );
};

const tagOptions = computed(() => {
  const tagSet = new Set<string>();
  datasets.value.forEach((dataset) => {
    dataset.tags.forEach((tag) => tagSet.add(tag));
  });
  return Array.from(tagSet).sort((a, b) => a.localeCompare(b));
});

const filteredDatasets = computed(() =>
  favoritesOnly.value
    ? datasets.value.filter((dataset) => dataset.favorite)
    : datasets.value,
);

let unsubscribe: (() => void) | null = null;
let searchDebounce: ReturnType<typeof setTimeout> | undefined;
let statusRefreshTimer: ReturnType<typeof setInterval> | null = null;

const loadDatasets = async ({ append = false } = {}) => {
  if (isLoading.value) return;
  isLoading.value = true;
  try {
    const offset = append ? datasets.value.length : 0;
    const next = await listDatasets(
      searchQuery.value,
      selectedTags.value,
      DATASET_PAGE_SIZE,
      offset,
    );
    datasets.value = append ? [...datasets.value, ...next] : next;
    syncSelectedDataset();
  } finally {
    isLoading.value = false;
  }
};

const refreshDatasets = async () => {
  if (isLoading.value) return;
  isLoading.value = true;
  try {
    const limit = Math.max(datasets.value.length, DATASET_PAGE_SIZE);
    datasets.value = await listDatasets(
      searchQuery.value,
      selectedTags.value,
      limit,
      0,
    );
    syncSelectedDataset();
  } finally {
    isLoading.value = false;
  }
};

const statusSeverity = (status: DatasetStatus) => {
  switch (status) {
    case "Writing":
      return "info";
    case "Completed":
      return "success";
    case "Aborted":
      return "danger";
    default:
      return "secondary";
  }
};

const startStatusPolling = () => {
  if (statusRefreshTimer) return;
  statusRefreshTimer = setInterval(() => {
    void refreshDatasets();
  }, 2000);
};

const stopStatusPolling = () => {
  if (!statusRefreshTimer) return;
  clearInterval(statusRefreshTimer);
  statusRefreshTimer = null;
};

const handleDatasetCreated = (event: DatasetInfo) => {
  datasets.value.unshift(event);
  if (searchQuery.value.trim() || selectedTags.value.length > 0) {
    void loadDatasets();
  }
  syncSelectedDataset();
};

onMounted(async () => {
  await loadDatasets();

  // Listen for dataset created events
  unsubscribe = await onDatasetCreated(handleDatasetCreated);
});

onUnmounted(() => {
  unsubscribe?.();
  if (searchDebounce) {
    clearTimeout(searchDebounce);
  }
  stopStatusPolling();
});

watch([searchQuery, selectedTags], () => {
  if (searchDebounce) {
    clearTimeout(searchDebounce);
  }
  searchDebounce = setTimeout(() => {
    void loadDatasets();
  }, 300);
});

watch(
  () => props.selectedDatasetId,
  () => {
    syncSelectedDataset();
  },
  { immediate: true },
);

const handleLazyLoad = (event: { first: number; last: number }) => {
  if (event.last <= datasets.value.length) return;
  void loadDatasets({ append: true });
};

watch(
  datasets,
  (nextDatasets) => {
    const hasWriting = nextDatasets.some(
      (dataset) => dataset.status === "Writing",
    );
    if (hasWriting) {
      startStatusPolling();
    } else {
      stopStatusPolling();
    }
  },
  { deep: true },
);

function handleKeydown(event: KeyboardEvent) {
  if (event.metaKey || event.ctrlKey) {
    event.stopPropagation();
  }
}

const selectDataset = (id: number) => {
  emit("datasetSelected", id);
  if (router.currentRoute.value.params.id !== String(id)) {
    void router.push({ name: "dataset", params: { id } });
  }
};

const toggleFavorite = async (dataset: DatasetInfo) => {
  const nextFavorite = !dataset.favorite;
  dataset.favorite = nextFavorite;
  try {
    await updateDatasetFavorite(dataset.id, nextFavorite);
  } catch (error) {
    dataset.favorite = !nextFavorite;
    throw error;
  }
};
</script>
<template>
  <div class="flex h-full flex-col">
    <div class="flex flex-wrap items-center gap-2 p-2">
      <div class="flex items-center gap-2">
        <ToggleSwitch v-model="favoritesOnly" input-id="favorites-only" />
        <label for="favorites-only">Favorites only</label>
      </div>
      <InputText
        v-model="searchQuery"
        placeholder="Search by name"
        class="h-8 w-full sm:max-w-64"
      />
      <MultiSelect
        v-model="selectedTags"
        :options="tagOptions"
        placeholder="Filter by tags"
        class="h-8 w-full sm:max-w-72"
        display="chip"
        filter
      />
    </div>
    <DataTable
      v-model:selection="selectedDataset"
      :value="filteredDatasets"
      size="small"
      data-key="id"
      selection-mode="single"
      removable-sort
      scrollable
      scroll-height="flex"
      :virtual-scroller-options="{
        itemSize: 40,
        lazy: true,
        onLazyLoad: handleLazyLoad,
      }"
      @keydown.capture="handleKeydown"
      @row-select="selectDataset($event.data.id)"
    >
      <Column header="Favorite" class="w-24">
        <template #body="slotProps">
          <Button
            :icon="slotProps.data.favorite ? 'pi pi-star-fill' : 'pi pi-star'"
            :aria-label="
              slotProps.data.favorite
                ? 'Unfavorite dataset'
                : 'Favorite dataset'
            "
            text
            rounded
            @click.stop="toggleFavorite(slotProps.data)"
          />
        </template>
      </Column>
      <Column field="id" header="ID" />
      <Column field="name" header="Name" />
      <Column field="status" header="Status" class="w-36">
        <template #body="slotProps">
          <Tag
            :value="slotProps.data.status"
            :severity="statusSeverity(slotProps.data.status)"
          />
        </template>
      </Column>
      <Column field="tags" header="Tags">
        <template #body="slotProps">
          <Tag
            v-for="(tag, index) in slotProps.data.tags"
            :key="index"
            class="mr-1 mb-1"
          >
            {{ tag }}
          </Tag>
        </template>
      </Column>
      <Column field="createdAt" header="Created At" sortable>
        <template #body="slotProps">
          {{ slotProps.data.createdAt.toLocaleString() }}
        </template>
      </Column>
    </DataTable>
  </div>
</template>
