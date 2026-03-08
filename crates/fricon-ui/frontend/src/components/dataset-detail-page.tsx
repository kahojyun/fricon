import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDatasetInfo, type DatasetDetail } from "@/lib/backend";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Separator } from "@/components/ui/separator";
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
          <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto">
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

  const normalizedDetailTags = normalizeTagList(detail.tags);
  const normalizedFormTags = parseTags(formTagsText);

  const hasChanges =
    formName !== detail.name ||
    formDescription !== detail.description ||
    formFavorite !== detail.favorite ||
    normalizedFormTags.join("|") !== normalizedDetailTags.join("|");

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
    <div className="flex flex-col gap-3">
      <div className="grid gap-3 lg:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
        <Card className="ring-border">
          <CardHeader className="border-b">
            <CardTitle>Dataset Details</CardTitle>
            <CardDescription>
              Update the display metadata used across the workspace.
            </CardDescription>
            <CardAction>
              <Button
                type="button"
                disabled={!hasChanges || isSaving}
                onClick={() => void handleSave()}
              >
                {isSaving ? "Saving..." : "Save"}
              </Button>
            </CardAction>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            {saveErrorMessage ? (
              <Alert variant="destructive">
                <AlertTitle>Save failed</AlertTitle>
                <AlertDescription>{saveErrorMessage}</AlertDescription>
              </Alert>
            ) : null}
            {successMessage ? (
              <Alert>
                <AlertTitle>Saved</AlertTitle>
                <AlertDescription>{successMessage}</AlertDescription>
              </Alert>
            ) : null}

            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="dataset-name">Name</FieldLabel>
                <FieldContent>
                  <Input
                    id="dataset-name"
                    value={formName}
                    onChange={(event) => setFormName(event.target.value)}
                  />
                </FieldContent>
              </Field>

              <Field>
                <FieldLabel htmlFor="dataset-description">
                  Description
                </FieldLabel>
                <FieldContent>
                  <Textarea
                    id="dataset-description"
                    rows={4}
                    value={formDescription}
                    onChange={(event) => setFormDescription(event.target.value)}
                  />
                </FieldContent>
              </Field>

              <Field>
                <FieldLabel htmlFor="dataset-tags">Tags</FieldLabel>
                <FieldContent>
                  <Input
                    id="dataset-tags"
                    placeholder="Comma separated tags"
                    value={formTagsText}
                    onChange={(event) => setFormTagsText(event.target.value)}
                  />
                  <FieldDescription>
                    Tags are normalized, deduplicated, and sorted on save.
                  </FieldDescription>
                </FieldContent>
              </Field>

              <Field
                orientation="horizontal"
                className="items-center justify-between rounded-md border px-3 py-2"
              >
                <FieldContent>
                  <FieldLabel htmlFor="dataset-favorite">Favorite</FieldLabel>
                  <FieldDescription>
                    Keep this dataset pinned for quick access.
                  </FieldDescription>
                </FieldContent>
                <Switch
                  id="dataset-favorite"
                  checked={formFavorite}
                  onCheckedChange={setFormFavorite}
                />
              </Field>
            </FieldGroup>
          </CardContent>
        </Card>

        <Card className="bg-muted/30 ring-border">
          <CardHeader className="border-b">
            <CardTitle>Metadata</CardTitle>
            <CardDescription>
              Current backend state for this dataset.
            </CardDescription>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <div className="grid gap-1.5 text-xs">
              <div>
                <span className="font-medium">ID:</span> {detail.id}
              </div>
              <div className="flex items-center gap-2">
                <span className="font-medium">Status:</span>
                <Badge variant={statusVariantMap[detail.status]}>
                  {detail.status}
                </Badge>
              </div>
              <div>
                <span className="font-medium">Created:</span>{" "}
                {detail.createdAt.toLocaleString()}
              </div>
            </div>

            <Separator />

            <div className="flex flex-col gap-1.5">
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
          </CardContent>
        </Card>
      </div>

      <Card className="ring-border">
        <CardHeader className="border-b">
          <CardTitle>Columns</CardTitle>
          <CardDescription>
            {detail.columns.length} columns detected in this dataset.
          </CardDescription>
        </CardHeader>
        <CardContent>
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
        </CardContent>
      </Card>
    </div>
  );
}

const statusVariantMap: Record<
  DatasetDetail["status"],
  "default" | "secondary" | "destructive"
> = {
  Writing: "secondary",
  Completed: "default",
  Aborted: "destructive",
};

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
