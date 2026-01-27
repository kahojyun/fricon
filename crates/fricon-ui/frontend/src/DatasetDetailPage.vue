<script setup lang="ts">
import ChartViewer from "./ChartViewer.vue";
import {
  getDatasetDetail,
  updateDatasetInfo,
  type DatasetDetail,
} from "./backend";
import { computed, ref, watch } from "vue";

const props = defineProps<{
  datasetId: number;
}>();

const emit = defineEmits<{
  datasetUpdated: [];
}>();

const detail = ref<DatasetDetail>();
const isLoading = ref(false);
const isSaving = ref(false);
const errorMessage = ref<string>();
const successMessage = ref<string>();

const formName = ref("");
const formDescription = ref("");
const formFavorite = ref(false);
const formTagsText = ref("");
const activeTab = ref("charts");

function tagsToText(tags: string[]): string {
  return tags.join(", ");
}

function normalizeTagList(tags: string[]): string[] {
  const trimmed = tags.map((tag) => tag.trim()).filter((tag) => tag.length > 0);
  return Array.from(new Set(trimmed)).sort((a, b) => a.localeCompare(b));
}

function parseTags(text: string): string[] {
  if (!text.trim()) return [];
  return normalizeTagList(text.split(","));
}

function setFormFromDetail(next: DatasetDetail) {
  formName.value = next.name;
  formDescription.value = next.description;
  formFavorite.value = next.favorite;
  formTagsText.value = tagsToText(next.tags);
}

async function loadDetail(id: number) {
  isLoading.value = true;
  errorMessage.value = undefined;
  successMessage.value = undefined;
  try {
    const next = await getDatasetDetail(id);
    detail.value = next;
    setFormFromDetail(next);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isLoading.value = false;
  }
}

const normalizedDetailTags = computed(() =>
  detail.value ? normalizeTagList(detail.value.tags) : [],
);

const normalizedFormTags = computed(() => parseTags(formTagsText.value));

const hasChanges = computed(() => {
  const current = detail.value;
  if (!current) return false;
  if (formName.value !== current.name) return true;
  if (formDescription.value !== current.description) return true;
  if (formFavorite.value !== current.favorite) return true;
  return (
    normalizedFormTags.value.join("|") !== normalizedDetailTags.value.join("|")
  );
});

async function saveChanges() {
  const current = detail.value;
  if (!current || !hasChanges.value) return;
  isSaving.value = true;
  errorMessage.value = undefined;
  successMessage.value = undefined;
  try {
    await updateDatasetInfo(props.datasetId, {
      name: formName.value,
      description: formDescription.value,
      favorite: formFavorite.value,
      tags: normalizedFormTags.value,
    });
    successMessage.value = "Dataset updated.";
    emit("datasetUpdated");
    await loadDetail(props.datasetId);
  } catch (error) {
    errorMessage.value = error instanceof Error ? error.message : String(error);
  } finally {
    isSaving.value = false;
  }
}

watch(
  () => props.datasetId,
  (id) => {
    void loadDetail(id);
  },
  { immediate: true },
);
</script>

<template>
  <div class="flex h-full flex-col gap-0 overflow-auto p-0">
    <div class="flex flex-1 flex-col overflow-hidden">
      <Tabs
        v-model:value="activeTab"
        class="dataset-tabs flex h-full flex-1 flex-col overflow-hidden"
        style="--p-tabs-tabpanel-padding: 0"
      >
        <TabList>
          <Tab value="charts">Charts</Tab>
          <Tab value="details">Details</Tab>
        </TabList>
        <TabPanels class="flex flex-1 flex-col overflow-hidden">
          <TabPanel value="charts" class="flex flex-1 flex-col overflow-hidden">
            <div class="flex flex-1 flex-col gap-0 overflow-hidden p-0">
              <div class="min-h-0 flex-1">
                <ChartViewer :dataset-id="props.datasetId" />
              </div>
            </div>
          </TabPanel>

          <TabPanel value="details" class="h-full overflow-auto">
            <div class="p-3">
              <div
                v-if="isLoading && !detail"
                class="text-surface-500 p-2 text-xs"
              >
                Loading dataset...
              </div>

              <div v-else-if="detail" class="flex flex-col gap-2">
                <div
                  class="grid gap-2 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]"
                >
                  <div
                    class="border-surface-200 flex flex-col gap-2 rounded border p-2"
                  >
                    <div class="flex items-center justify-between gap-2">
                      <h2 class="text-sm font-semibold">Dataset Details</h2>
                      <Button
                        label="Save"
                        icon="pi pi-save"
                        :disabled="!hasChanges || isSaving"
                        :loading="isSaving"
                        size="small"
                        @click="saveChanges"
                      />
                    </div>

                    <div v-if="errorMessage">
                      <Message severity="error" :closable="false" size="small">
                        {{ errorMessage }}
                      </Message>
                    </div>
                    <div v-else-if="successMessage">
                      <Message
                        severity="success"
                        :closable="false"
                        size="small"
                      >
                        {{ successMessage }}
                      </Message>
                    </div>

                    <div class="flex flex-col gap-2">
                      <label class="text-xs font-medium" for="dataset-name"
                        >Name</label
                      >
                      <InputText
                        id="dataset-name"
                        v-model="formName"
                        class="h-9"
                      />
                    </div>

                    <div class="flex flex-col gap-2">
                      <label
                        class="text-xs font-medium"
                        for="dataset-description"
                        >Description</label
                      >
                      <Textarea
                        id="dataset-description"
                        v-model="formDescription"
                        auto-resize
                        rows="4"
                      />
                    </div>

                    <div class="flex flex-col gap-2">
                      <label class="text-xs font-medium" for="dataset-tags"
                        >Tags</label
                      >
                      <InputText
                        id="dataset-tags"
                        v-model="formTagsText"
                        placeholder="Comma separated tags"
                        class="h-9"
                      />
                    </div>

                    <div class="flex items-center gap-2">
                      <ToggleSwitch
                        v-model="formFavorite"
                        input-id="dataset-favorite"
                      />
                      <label for="dataset-favorite">Favorite</label>
                    </div>
                  </div>

                  <div
                    class="border-surface-200 flex flex-col gap-2 rounded border p-2"
                  >
                    <h3 class="text-xs font-semibold">Metadata</h3>
                    <div class="text-xs">
                      <div>
                        <span class="font-medium">ID:</span> {{ detail.id }}
                      </div>
                      <div class="mt-1">
                        <span class="font-medium">Status:</span>
                        <Tag :value="detail.status" class="ml-2" />
                      </div>
                      <div class="mt-1">
                        <span class="font-medium">Created:</span>
                        {{ detail.createdAt.toLocaleString() }}
                      </div>
                    </div>

                    <div class="flex flex-col gap-2">
                      <div class="text-xs font-medium">Current Tags</div>
                      <div class="flex flex-wrap gap-1">
                        <Tag v-for="tag in detail.tags" :key="tag">{{
                          tag
                        }}</Tag>
                        <span
                          v-if="detail.tags.length === 0"
                          class="text-surface-500 text-xs"
                        >
                          No tags
                        </span>
                      </div>
                    </div>
                  </div>
                </div>

                <div class="border-surface-200 rounded border p-2">
                  <div class="mb-2 flex items-center justify-between">
                    <h3 class="text-xs font-semibold">Columns</h3>
                    <span class="text-surface-500 text-xs">
                      {{ detail.columns.length }} columns
                    </span>
                  </div>
                  <DataTable
                    :value="detail.columns"
                    size="small"
                    scrollable
                    scroll-height="240px"
                  >
                    <Column field="name" header="Name" />
                    <Column header="Index" class="w-20">
                      <template #body="slotProps">
                        <i
                          v-if="slotProps.data.isIndex"
                          class="pi pi-check text-green-500"
                          aria-label="Index column"
                        />
                      </template>
                    </Column>
                    <Column header="Type" class="w-40">
                      <template #body="slotProps">
                        <Tag v-if="slotProps.data.isTrace" value="Trace" />
                        <Tag
                          v-else-if="slotProps.data.isComplex"
                          value="Complex"
                          severity="info"
                        />
                        <Tag v-else value="Scalar" severity="secondary" />
                      </template>
                    </Column>
                  </DataTable>
                </div>
              </div>

              <div v-else class="text-surface-500 p-2 text-xs">
                Dataset not found.
              </div>
            </div>
          </TabPanel>
        </TabPanels>
      </Tabs>
    </div>
  </div>
</template>
