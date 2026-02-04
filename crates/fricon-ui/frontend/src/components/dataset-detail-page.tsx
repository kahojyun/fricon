import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  getDatasetDetail,
  updateDatasetInfo,
  type DatasetDetail,
} from "@/lib/backend";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";

interface DatasetDetailPageProps {
  datasetId: number;
  onDatasetUpdated?: () => void;
}

export function DatasetDetailPage({
  datasetId,
  onDatasetUpdated,
}: DatasetDetailPageProps) {
  const [detail, setDetail] = useState<DatasetDetail | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [formName, setFormName] = useState("");
  const [formDescription, setFormDescription] = useState("");
  const [formFavorite, setFormFavorite] = useState(false);
  const [formTagsText, setFormTagsText] = useState("");
  const requestTokenRef = useRef(0);

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

  const loadDetail = useCallback(
    async (id: number) => {
      const token = ++requestTokenRef.current;
      setIsLoading(true);
      setErrorMessage(null);
      setSuccessMessage(null);
      try {
        const next = await getDatasetDetail(id);
        if (token !== requestTokenRef.current || datasetId !== id) return;
        setDetail(next);
        setFormName(next.name);
        setFormDescription(next.description);
        setFormFavorite(next.favorite);
        setFormTagsText(tagsToText(next.tags));
      } catch (error) {
        if (token !== requestTokenRef.current) return;
        setErrorMessage(error instanceof Error ? error.message : String(error));
      } finally {
        if (token === requestTokenRef.current) {
          setIsLoading(false);
        }
      }
    },
    [datasetId],
  );

  useEffect(() => {
    void loadDetail(datasetId);
  }, [datasetId, loadDetail]);

  const handleSave = async () => {
    if (!detail || !hasChanges) return;
    setIsSaving(true);
    setErrorMessage(null);
    setSuccessMessage(null);
    try {
      await updateDatasetInfo(datasetId, {
        name: formName,
        description: formDescription,
        favorite: formFavorite,
        tags: normalizedFormTags,
      });
      setSuccessMessage("Dataset updated.");
      onDatasetUpdated?.();
      await loadDetail(datasetId);
    } catch (error) {
      setErrorMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <Tabs defaultValue="charts" className="flex h-full flex-col">
        <TabsList>
          <TabsTrigger value="charts">Charts</TabsTrigger>
          <TabsTrigger value="details">Details</TabsTrigger>
        </TabsList>

        <TabsContent value="charts" className="flex-1">
          <div className="text-muted-foreground text-sm">
            Chart viewer placeholder.
          </div>
        </TabsContent>

        <TabsContent value="details" className="flex-1">
          <div className="space-y-4">
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
