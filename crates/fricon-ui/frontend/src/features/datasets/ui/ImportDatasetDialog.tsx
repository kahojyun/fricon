import { AlertTriangle, DownloadCloud } from "lucide-react";
import type { UiPreviewImportResult } from "../api/types";
import type { DuplicateBatchConflict } from "./useDatasetImportFlow";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/shared/ui/alert-dialog";
import { Alert, AlertDescription, AlertTitle } from "@/shared/ui/alert";

interface ImportDatasetDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  previewResults: UiPreviewImportResult[];
  duplicateBatchConflicts: DuplicateBatchConflict[];
  isImporting: boolean;
  onConfirm: () => void;
}

function fileNameFromPath(path: string) {
  return path.split(/[/\\]/).at(-1) ?? path;
}

export function ImportDatasetDialog({
  open,
  onOpenChange,
  previewResults,
  duplicateBatchConflicts,
  isImporting,
  onConfirm,
}: ImportDatasetDialogProps) {
  if (!previewResults || previewResults.length === 0) {
    return null;
  }

  const conflicts = previewResults.filter((r) => r.preview.conflict !== null);
  const hasConflict = conflicts.length > 0;
  const hasDuplicateBatchConflicts = duplicateBatchConflicts.length > 0;

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent className="max-w-md max-h-[80vh] flex flex-col">
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <DownloadCloud className="h-5 w-5" />
            Import Datasets
          </AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to import {previewResults.length} dataset(s)?
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="text-sm my-4 space-y-4 overflow-y-auto flex-1 pr-2">
          {hasDuplicateBatchConflicts && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>Duplicate Dataset UUIDs</AlertTitle>
              <AlertDescription className="mt-2 space-y-2">
                <p>
                  Multiple selected archives target the same dataset UUID. Remove
                  duplicate archives before importing this batch.
                </p>
                <div className="space-y-2">
                  {duplicateBatchConflicts.map((conflict) => (
                    <div
                      key={conflict.uid}
                      className="rounded-sm bg-destructive/10 p-2 text-xs"
                    >
                      <div className="font-semibold break-all">
                        UUID: {conflict.uid}
                      </div>
                      <div className="mt-1 text-destructive/90">
                        {conflict.entries
                          .map((entry) => fileNameFromPath(entry.archivePath))
                          .join(", ")}
                      </div>
                    </div>
                  ))}
                </div>
              </AlertDescription>
            </Alert>
          )}

          {previewResults.map((result) => {
            const { metadata, conflict } = result.preview;
            return (
              <div key={metadata.uid} className="space-y-2">
                <div className="rounded-md bg-muted p-3">
                  <p>
                    <strong>Name:</strong>{" "}
                    <span className="break-all">{metadata.name}</span>
                  </p>
                  <p>
                    <strong>UUID:</strong>{" "}
                    <span className="text-xs font-mono break-all">
                      {metadata.uid}
                    </span>
                  </p>
                  <p className="text-muted-foreground break-all">
                    {metadata.description}
                  </p>
                </div>

                {conflict && (
                  <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertTitle>Conflict Detected</AlertTitle>
                    <AlertDescription className="mt-2 space-y-2">
                      <p>
                        A dataset with this UUID already exists. Continuing will
                        overwrite the existing dataset&apos;s metadata and data.
                      </p>
                      {conflict.diffs.length > 0 && (
                        <div className="bg-destructive/10 text-destructive rounded-sm p-2 text-xs font-mono max-h-32 overflow-y-auto">
                          {conflict.diffs.map((diff) => (
                            <div key={`diff-${diff.field}`} className="mb-2 last:mb-0">
                              <div className="font-semibold">{diff.field}:</div>
                              <div className="text-red-700/80 dark:text-red-400 line-through break-all">
                                - {diff.existingValue}
                              </div>
                              <div className="text-green-700/80 dark:text-green-400 break-all">
                                + {diff.incomingValue}
                              </div>
                            </div>
                          ))}
                        </div>
                      )}
                    </AlertDescription>
                  </Alert>
                )}
              </div>
            );
          })}
        </div>

        <AlertDialogFooter>
          <AlertDialogCancel disabled={isImporting}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={(event) => {
              event.preventDefault();
              onConfirm();
            }}
            disabled={isImporting || hasDuplicateBatchConflicts}
            className={
              hasConflict
                ? "bg-destructive text-destructive-foreground hover:bg-destructive/90"
                : ""
            }
          >
            {isImporting
              ? "Importing..."
              : hasConflict
                ? "Overwrite & Import"
                : "Import Dataset(s)"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
