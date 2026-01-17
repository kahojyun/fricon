<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, shallowRef, watch } from "vue";
import {
  type DatasetInfo,
  listDatasets,
  onDatasetCreated,
  updateDatasetFavorite,
} from "./backend";

const emit = defineEmits<{
  datasetSelected: [id: number];
}>();
const datasets = ref<DatasetInfo[]>([]);
const selectedDataset = shallowRef<DatasetInfo>();
const favoritesOnly = ref(false);
const searchQuery = ref("");
const selectedTags = ref<string[]>([]);

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

const loadDatasets = async () => {
  datasets.value = await listDatasets(searchQuery.value, selectedTags.value);
};

const handleDatasetCreated = (event: DatasetInfo) => {
  datasets.value.unshift(event);
  if (searchQuery.value.trim() || selectedTags.value.length > 0) {
    void loadDatasets();
  }
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
});

watch([searchQuery, selectedTags], () => {
  if (searchDebounce) {
    clearTimeout(searchDebounce);
  }
  searchDebounce = setTimeout(() => {
    void loadDatasets();
  }, 300);
});

function handleKeydown(event: KeyboardEvent) {
  if (event.metaKey || event.ctrlKey) {
    event.stopPropagation();
  }
}

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
      @keydown.capture="handleKeydown"
      @row-select="emit('datasetSelected', $event.data.id)"
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
