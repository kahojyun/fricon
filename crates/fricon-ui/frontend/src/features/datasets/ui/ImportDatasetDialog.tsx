import { AlertTriangle, DownloadCloud } from "lucide-react";
import type { UiPreviewImportResult } from "../api/types";
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
  previewResult: UiPreviewImportResult | null;
  isImporting: boolean;
  onConfirm: () => void;
}

export function ImportDatasetDialog({
  open,
  onOpenChange,
  previewResult,
  isImporting,
  onConfirm,
}: ImportDatasetDialogProps) {
  if (!previewResult) {
    return null;
  }

  const { metadata, conflict } = previewResult.preview;
  const hasConflict = conflict !== null;

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent className="max-w-md">
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <DownloadCloud className="h-5 w-5" />
            Import Dataset
          </AlertDialogTitle>
          <AlertDialogDescription>
            Are you sure you want to import this dataset?
          </AlertDialogDescription>
        </AlertDialogHeader>

        <div className="text-sm my-4 space-y-2">
          <div className="rounded-md bg-muted p-3">
            <p><strong>Name:</strong> {metadata.name}</p>
            <p><strong>UUID:</strong> <span className="text-xs font-mono">{metadata.uid}</span></p>
            <p className="text-muted-foreground">{metadata.description}</p>
          </div>

          {hasConflict && (
            <Alert variant="destructive" className="mt-4">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>Conflict Detected</AlertTitle>
              <AlertDescription className="mt-2 space-y-2">
                <p>
                  A dataset with this UUID already exists. Continuing will overwrite
                  the existing dataset's metadata and data.
                </p>
                {conflict.diffs.length > 0 && (
                  <div className="bg-destructive/10 text-destructive rounded-sm p-2 text-xs font-mono max-h-32 overflow-y-auto">
                    {conflict.diffs.map((diff) => (
                      <div key={`diff-${diff.field}`} className="mb-2 last:mb-0">
                        <div className="font-semibold">{diff.field}:</div>
                        <div className="text-red-700/80 dark:text-red-400 line-through">- {diff.existingValue}</div>
                        <div className="text-green-700/80 dark:text-green-400">+ {diff.incomingValue}</div>
                      </div>
                    ))}
                  </div>
                )}
              </AlertDescription>
            </Alert>
          )}
        </div>

        <AlertDialogFooter>
          <AlertDialogCancel disabled={isImporting}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={(event) => {
              event.preventDefault();
              onConfirm();
            }}
            disabled={isImporting}
            className={hasConflict ? "bg-destructive text-destructive-foreground hover:bg-destructive/90" : ""}
          >
            {isImporting ? "Importing..." : hasConflict ? "Overwrite Dataset" : "Import Dataset"}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
