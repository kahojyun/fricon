import { useMemo, useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDatasetInfo, type DatasetDetail } from "@/lib/backend";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
  const detailQuery = useDatasetDetailQuery(datasetId);
  const detail = detailQuery.data ?? null;
  const isLoading = detailQuery.isLoading;

  const loadErrorMessage =
    detailQuery.error instanceof Error ? detailQuery.error.message : null;

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <Tabs defaultValue="charts" className="flex h-full min-h-0 flex-col">
        <TabsList>
          <TabsTrigger value="charts">Charts</TabsTrigger>
          <TabsTrigger value="properties">Properties</TabsTrigger>
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
          value="properties"
          className="flex min-h-0 flex-1 flex-col overflow-hidden"
        >
          <div className="min-h-0 flex-1 space-y-3 overflow-auto">
            {isLoading && !detail ? (
              <div className="text-xs text-muted-foreground">
                Loading dataset...
              </div>
            ) : null}

            {detail ? (
              <DatasetDetailEditor
                key={datasetId}
                datasetId={datasetId}
                detail={detail}
                onDatasetUpdated={onDatasetUpdated}
              />
            ) : null}

            {!detail && !isLoading ? (
              <div className="text-xs text-muted-foreground">
                {loadErrorMessage ?? "Dataset not found."}
              </div>
            ) : null}
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}

interface DatasetDetailEditorProps {
  datasetId: number;
  detail: DatasetDetail;
  onDatasetUpdated?: () => void;
}

function DatasetDetailEditor({
  datasetId,
  detail,
  onDatasetUpdated,
}: DatasetDetailEditorProps) {
  const queryClient = useQueryClient();
  const [saveErrorMessage, setSaveErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [formName, setFormName] = useState(detail.name);
  const [formDescription, setFormDescription] = useState(detail.description);
  const [formFavorite, setFormFavorite] = useState(detail.favorite);
  const [formTagsText, setFormTagsText] = useState(() =>
    tagsToText(detail.tags),
  );

  const normalizedDetailTags = useMemo(
    () => normalizeTagList(detail.tags),
    [detail.tags],
  );
  const normalizedFormTags = useMemo(
    () => parseTags(formTagsText),
    [formTagsText],
  );

  const hasChanges = useMemo(() => {
    if (formName !== detail.name) return true;
    if (formDescription !== detail.description) return true;
    if (formFavorite !== detail.favorite) return true;
    return normalizedFormTags.join("|") !== normalizedDetailTags.join("|");
  }, [
    detail.description,
    detail.favorite,
    detail.name,
    formDescription,
    formFavorite,
    formName,
    normalizedDetailTags,
    normalizedFormTags,
  ]);

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
    if (!hasChanges) return;
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
      setSaveErrorMessage(
        error instanceof Error ? error.message : String(error),
      );
    }
  };

  return (
    <>
      <div className="grid gap-3 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
        <div className="rounded-md border p-2.5">
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

          {saveErrorMessage ? (
            <div className="mt-2 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive">
              {saveErrorMessage}
            </div>
          ) : null}
          {successMessage ? (
            <div className="mt-2 rounded-md border border-emerald-500/30 bg-emerald-500/10 px-3 py-2 text-xs text-emerald-700">
              {successMessage}
            </div>
          ) : null}

          <div className="mt-3 space-y-2">
            <div className="space-y-1">
              <Label htmlFor="dataset-name">Name</Label>
              <Input
                id="dataset-name"
                value={formName}
                onChange={(event) => setFormName(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="dataset-description">Description</Label>
              <Textarea
                id="dataset-description"
                rows={4}
                value={formDescription}
                onChange={(event) => setFormDescription(event.target.value)}
              />
            </div>

            <div className="space-y-1">
              <Label htmlFor="dataset-tags">Tags</Label>
              <Input
                id="dataset-tags"
                placeholder="Comma separated tags"
                value={formTagsText}
                onChange={(event) => setFormTagsText(event.target.value)}
              />
            </div>

            <div className="flex items-center gap-2">
              <Switch
                id="dataset-favorite"
                checked={formFavorite}
                onCheckedChange={setFormFavorite}
              />
              <Label htmlFor="dataset-favorite" className="whitespace-nowrap">
                Favorite
              </Label>
            </div>
          </div>
        </div>

        <div className="rounded-md border bg-muted/30 p-2.5">
          <h3 className="text-xs font-semibold">Metadata</h3>
          <div className="mt-1.5 text-xs">
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

          <div className="mt-3 space-y-1.5">
            <div className="text-xs font-medium">Current Tags</div>
            <div className="flex flex-wrap gap-1">
              {detail.tags.length > 0 ? (
                detail.tags.map((tag) => (
                  <Badge key={tag} variant="secondary">
                    {tag}
                  </Badge>
                ))
              ) : (
                <span className="text-xs text-muted-foreground">No tags</span>
              )}
            </div>
          </div>
        </div>
      </div>

      <div className="rounded-md border p-2.5">
        <div className="mb-2 flex items-center justify-between">
          <h3 className="text-xs font-semibold">Columns</h3>
          <span className="text-xs text-muted-foreground">
            {detail.columns.length} columns
          </span>
        </div>
        <div className="overflow-hidden rounded-md border">
          <Table>
            <TableHeader className="bg-muted/40 text-muted-foreground">
              <TableRow>
                <TableHead>Name</TableHead>
                <TableHead>Index</TableHead>
                <TableHead>Type</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {detail.columns.map((column) => (
                <TableRow key={column.name}>
                  <TableCell>{column.name}</TableCell>
                  <TableCell>{column.isIndex ? "✓" : ""}</TableCell>
                  <TableCell>
                    {column.isTrace ? (
                      <Badge variant="secondary">Trace</Badge>
                    ) : column.isComplex ? (
                      <Badge variant="outline">Complex</Badge>
                    ) : (
                      <Badge variant="secondary">Scalar</Badge>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      </div>
    </>
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
