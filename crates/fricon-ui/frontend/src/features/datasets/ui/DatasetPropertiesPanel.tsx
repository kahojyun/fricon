import { useState } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { updateDatasetInfo } from "../api/client";
import { datasetKeys } from "../api/queryKeys";
import type {
  DatasetDetail,
  DatasetInfoUpdate,
} from "../api/types";
import { Alert, AlertDescription, AlertTitle } from "@/shared/ui/alert";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/shared/ui/card";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldGroup,
  FieldLabel,
} from "@/shared/ui/field";
import { Input } from "@/shared/ui/input";
import { Separator } from "@/shared/ui/separator";
import { Switch } from "@/shared/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/shared/ui/table";
import { Textarea } from "@/shared/ui/textarea";

interface DatasetPropertiesPanelProps {
  datasetId: number;
  detail: DatasetDetail | null;
  isLoading: boolean;
  loadErrorMessage: string | null;
  onDatasetUpdated?: () => void;
}

export function DatasetPropertiesPanel({
  datasetId,
  detail,
  isLoading,
  loadErrorMessage,
  onDatasetUpdated,
}: DatasetPropertiesPanelProps) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 overflow-auto p-1">
      {isLoading && !detail ? (
        <div className="text-xs text-muted-foreground">Loading dataset...</div>
      ) : null}

      {detail ? (
        <DatasetDetailEditor
          key={buildDatasetDetailEditorKey(datasetId, detail)}
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
  );
}

interface DatasetDetailEditorProps {
  datasetId: number;
  detail: DatasetDetail;
  onDatasetUpdated?: () => void;
}

interface DatasetDetailEditorDraft {
  name: string;
  description: string;
  favorite: boolean;
  tagsText: string;
  normalizedTags: string[];
}

function DatasetDetailEditor({
  datasetId,
  detail,
  onDatasetUpdated,
}: DatasetDetailEditorProps) {
  const queryClient = useQueryClient();
  const initialDraft = createDatasetDetailEditorDraft(detail);
  const [saveErrorMessage, setSaveErrorMessage] = useState<string | null>(null);
  const [successMessage, setSuccessMessage] = useState<string | null>(null);
  const [formName, setFormName] = useState(initialDraft.name);
  const [formDescription, setFormDescription] = useState(
    initialDraft.description,
  );
  const [formFavorite, setFormFavorite] = useState(initialDraft.favorite);
  const [formTagsText, setFormTagsText] = useState(() => initialDraft.tagsText);

  const normalizedFormTags = parseTags(formTagsText);

  const hasChanges =
    formName !== initialDraft.name ||
    formDescription !== initialDraft.description ||
    formFavorite !== initialDraft.favorite ||
    normalizedFormTags.join("|") !== initialDraft.normalizedTags.join("|");

  const updateMutation = useMutation({
    mutationFn: (update: DatasetInfoUpdate) =>
      updateDatasetInfo(datasetId, update),
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
        queryKey: datasetKeys.detail(datasetId),
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

function createDatasetDetailEditorDraft(
  detail: DatasetDetail,
): DatasetDetailEditorDraft {
  const normalizedTags = normalizeTagList(detail.tags);

  return {
    name: detail.name,
    description: detail.description,
    favorite: detail.favorite,
    tagsText: tagsToText(normalizedTags),
    normalizedTags,
  };
}

function buildDatasetDetailEditorKey(
  datasetId: number,
  detail: DatasetDetail,
): string {
  const draft = createDatasetDetailEditorDraft(detail);

  return JSON.stringify({
    datasetId,
    name: draft.name,
    description: draft.description,
    favorite: draft.favorite,
    tags: draft.normalizedTags,
  });
}

function normalizeTagList(tags: string[]): string[] {
  const trimmed = tags.map((tag) => tag.trim()).filter((tag) => tag.length > 0);
  return Array.from(new Set(trimmed)).sort((a, b) => a.localeCompare(b));
}

function parseTags(text: string): string[] {
  if (!text.trim()) return [];
  return normalizeTagList(text.split(","));
}
