import { useEffect, useMemo, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDatasetInfo } from "@/lib/backend";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ChartViewer } from "@/components/chart-viewer";
import { useDatasetDetailQuery } from "@/hooks/useDatasetDetailQuery";

interface DatasetDetailPageProps {
  datasetId: number;
  onDatasetUpdated?: () => void;
}

export function DatasetDetailPage({
  datasetId,
  onDatasetUpdated,
}: DatasetDetailPageProps) {
  const queryClient = useQueryClient();
  const detailQuery = useDatasetDetailQuery(datasetId);
  const detail = detailQuery.data ?? null;
  const isLoading = detailQuery.isLoading;

  const [saveErrorMessage, setSaveErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [formName, setFormName] = useState("");
  const [formDescription, setFormDescription] = useState("");
  const [formFavorite, setFormFavorite] = useState(false);
  const [formTagsText, setFormTagsText] = useState("");

  const loadErrorMessage =
    detailQuery.error instanceof Error ? detailQuery.error.message : null;
  const errorMessage = saveErrorMessage ?? loadErrorMessage;

  const normalizedDetailTags = useMemo(() => {
    if (!detail) return [];
    return normalizeTagList(detail.tags);
  }, [detail]);

  const normalizedFormTags = useMemo(
    () => parseTags(formTagsText),
    [formTagsText],
  );

  const hasChanges = useMemo(() => {
    if (!detail) return false;
    if (formName !== detail.name) return true;
    if (formDescription !== detail.description) return true;
    if (formFavorite !== detail.favorite) return true;
    return normalizedFormTags.join("|") !== normalizedDetailTags.join("|");
  }, [
    detail,
    formName,
    formDescription,
    formFavorite,
    normalizedFormTags,
    normalizedDetailTags,
  ]);

  useEffect(() => {
    setSaveErrorMessage(null);
    setSuccessMessage(null);
  }, [datasetId]);

  useEffect(() => {
    if (!detail) return;
    setFormName(detail.name);
    setFormDescription(detail.description);
    setFormFavorite(detail.favorite);
    setFormTagsText(tagsToText(detail.tags));
  }, [detail]);

  const updateMutation = useMutation({
    mutationFn: (update: {
      name: string;
      description: string;
      favorite: boolean;
      tags: string[];
    }) => updateDatasetInfo(datasetId, update),
  });

  const isSaving = updateMutation.isPending;

  const handleSave = async () => {
    if (!detail || !hasChanges) return;
    setSaveErrorMessage(null);
    setSuccessMessage(null);
    try {
      await updateMutation.mutateAsync({
        name: formName,
        description: formDescription,
        favorite: formFavorite,
        tags: normalizedFormTags,
      });
      await queryClient.invalidateQueries({
        queryKey: ["datasetDetail", datasetId],
      });
      setSuccessMessage("Dataset updated.");
      onDatasetUpdated?.();
    } catch (error) {
      setSaveErrorMessage(error instanceof Error ? error.message : String(error));
    }
  };

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <Tabs defaultValue="charts" className="flex h-full min-h-0 flex-col">
        <TabsList>
          <TabsTrigger value="charts">Charts</TabsTrigger>
          <TabsTrigger value="details">Details</TabsTrigger>
        </TabsList>

        <TabsContent
          value="charts"
          className="flex min-h-0 flex-1 flex-col overflow-hidden"
        >
          <div className="min-h-0 flex-1 overflow-hidden">
            <ChartViewer datasetId={datasetId} />
          </div>
        </TabsContent>

        <TabsContent
          value="details"
          className="flex min-h-0 flex-1 flex-col overflow-hidden"
        >
          <div className="min-h-0 flex-1 space-y-4 overflow-auto">
            {isLoading && !detail ? (
              <div className="text-muted-foreground text-xs">
                Loading dataset...
              </div>
            ) : null}

            {detail ? (
              <>
                <div className="grid gap-4 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
                  <div className="rounded-md border p-3">
                    <div className="flex items-center justify-between gap-2">
                      <h2 className="text-sm font-semibold">Dataset Details</h2>
                      <Button
                        type="button"
                        size="sm"
                        disabled={!hasChanges || isSaving}
                        onClick={() => void handleSave()}
                      >
                        {isSaving ? "Saving..." : "Save"}
                      </Button>
                    </div>

                    {errorMessage ? (
                      <div className="border-destructive/30 bg-destructive/10 text-destructive mt-2 rounded-md border px-3 py-2 text-xs">
                        {errorMessage}
                      </div>
                    ) : null}
                    {successMessage ? (
                      <div className="mt-2 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-700">
                        {successMessage}
                      </div>
                    ) : null}

                    <div className="mt-4 space-y-3">
                      <div className="space-y-1">
                        <label
                          className="text-xs font-medium"
                          htmlFor="dataset-name"
                        >
                          Name
                        </label>
                        <Input
                          id="dataset-name"
                          value={formName}
                          onChange={(event) => setFormName(event.target.value)}
                        />
                      </div>

                      <div className="space-y-1">
                        <label
                          className="text-xs font-medium"
                          htmlFor="dataset-description"
                        >
                          Description
                        </label>
                        <Textarea
                          id="dataset-description"
                          rows={4}
                          value={formDescription}
                          onChange={(event) =>
                            setFormDescription(event.target.value)
                          }
                        />
                      </div>

                      <div className="space-y-1">
                        <label
                          className="text-xs font-medium"
                          htmlFor="dataset-tags"
                        >
                          Tags
                        </label>
                        <Input
                          id="dataset-tags"
                          placeholder="Comma separated tags"
                          value={formTagsText}
                          onChange={(event) =>
                            setFormTagsText(event.target.value)
                          }
                        />
                      </div>

                      <div className="flex items-center gap-2">
                        <Switch
                          checked={formFavorite}
                          onCheckedChange={setFormFavorite}
                        />
                        <span className="text-sm">Favorite</span>
                      </div>
                    </div>
                  </div>

                  <div className="rounded-md border p-3">
                    <h3 className="text-xs font-semibold">Metadata</h3>
                    <div className="mt-2 text-xs">
                      <div>
                        <span className="font-medium">ID:</span> {detail.id}
                      </div>
                      <div className="mt-1 flex items-center gap-2">
                        <span className="font-medium">Status:</span>
                        <Badge variant="secondary">{detail.status}</Badge>
                      </div>
                      <div className="mt-1">
                        <span className="font-medium">Created:</span>{" "}
                        {detail.createdAt.toLocaleString()}
                      </div>
                    </div>

                    <div className="mt-4 space-y-2">
                      <div className="text-xs font-medium">Current Tags</div>
                      <div className="flex flex-wrap gap-1">
                        {detail.tags.length > 0 ? (
                          detail.tags.map((tag) => (
                            <Badge key={tag} variant="secondary">
                              {tag}
                            </Badge>
                          ))
                        ) : (
                          <span className="text-muted-foreground text-xs">
                            No tags
                          </span>
                        )}
                      </div>
                    </div>
                  </div>
                </div>

                <div className="rounded-md border p-3">
                  <div className="mb-2 flex items-center justify-between">
                    <h3 className="text-xs font-semibold">Columns</h3>
                    <span className="text-muted-foreground text-xs">
                      {detail.columns.length} columns
                    </span>
                  </div>
                  <div className="overflow-hidden rounded-md border">
                    <table className="w-full text-xs">
                      <thead className="bg-muted/40 text-muted-foreground">
                        <tr>
                          <th className="px-2 py-2 text-left font-semibold">
                            Name
                          </th>
                          <th className="px-2 py-2 text-left font-semibold">
                            Index
                          </th>
                          <th className="px-2 py-2 text-left font-semibold">
                            Type
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        {detail.columns.map((column) => (
                          <tr
                            key={column.name}
                            className="text-foreground border-t"
                          >
                            <td className="px-2 py-2">{column.name}</td>
                            <td className="px-2 py-2">
                              {column.isIndex ? "✓" : ""}
                            </td>
                            <td className="px-2 py-2">
                              {column.isTrace ? (
                                <Badge variant="secondary">Trace</Badge>
                              ) : column.isComplex ? (
                                <Badge variant="outline">Complex</Badge>
                              ) : (
                                <Badge variant="secondary">Scalar</Badge>
                              )}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              </>
            ) : null}

            {!detail && !isLoading ? (
              <div className="text-muted-foreground text-xs">
                Dataset not found.
              </div>
            ) : null}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

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
