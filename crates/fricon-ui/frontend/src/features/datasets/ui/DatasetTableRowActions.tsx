import type { ReactElement } from "react";
import { Download, RotateCcw, Trash2 } from "lucide-react";
import type {
  DatasetDeleteResult,
  DatasetInfo,
  DatasetViewMode,
} from "../api/types";
import { DatasetRowTagMenus } from "./DatasetTableTagMenu";
import {
  deriveDatasetTagMenuTarget,
  runDatasetTagMutation,
} from "../model/datasetTableTagMenuLogic";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/shared/ui/context-menu";
import { toast } from "sonner";
import { commands } from "@/shared/lib/bindings";

interface DatasetTableRowActionsProps {
  dataset: DatasetInfo;
  viewMode: DatasetViewMode;
  selectedDatasets: DatasetInfo[];
  allTags: string[];
  isUpdatingTags: boolean;
  onDatasetSelected: (id?: number) => void;
  onTrash: (ids: number[]) => void;
  onRestore: (ids: number[]) => void;
  onPermanentDelete: (ids: number[]) => void;
  batchAddTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
  batchRemoveTags: (
    ids: number[],
    tags: string[],
  ) => Promise<DatasetDeleteResult[]>;
  children: ReactElement;
}

export function DatasetTableRowActions({
  dataset,
  viewMode,
  selectedDatasets,
  allTags,
  isUpdatingTags,
  onDatasetSelected,
  onTrash,
  onRestore,
  onPermanentDelete,
  batchAddTags,
  batchRemoveTags,
  children,
}: DatasetTableRowActionsProps) {
  const target = deriveDatasetTagMenuTarget(dataset, selectedDatasets);
  const selectedCount = selectedDatasets.length;
  const includesDataset = selectedDatasets.some(
    (selectedDataset) => selectedDataset.id === dataset.id,
  );
  const targetIds = includesDataset
    ? selectedDatasets.map((selectedDataset) => selectedDataset.id)
    : [dataset.id];

  const handleBatchTagMutation = async (
    operation: "add" | "remove",
    tag: string,
  ) => {
    await runDatasetTagMutation({
      operation,
      targetIds: target.targetIds,
      tag,
      batchAddTags,
      batchRemoveTags,
      notify: toast,
    });
  };

  const handleExport = () => {
    commands
      .exportDatasetDialog(dataset.id)
      .then((result) => {
        if (result.status === "ok" && result.data) {
          toast.success(`Dataset exported to ${result.data}`);
        } else if (result.status === "error") {
          toast.error(`Export failed: ${result.error.message}`);
        }
      })
      .catch((e) => {
        toast.error(
          `Export failed: ${e instanceof Error ? e.message : String(e)}`,
        );
      });
  };

  return (
    <ContextMenu>
      <ContextMenuTrigger render={children} />
      <ContextMenuContent className="w-64">
        <ContextMenuItem onClick={() => onDatasetSelected(dataset.id)}>
          View Details
        </ContextMenuItem>
        <ContextMenuSeparator />
        <DatasetRowTagMenus
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          target={target}
          onAddTag={(tag) => {
            void handleBatchTagMutation("add", tag);
          }}
          onRemoveTag={(tag) => {
            void handleBatchTagMutation("remove", tag);
          }}
        />
        <ContextMenuSeparator />
        <ContextMenuItem onClick={() => void handleExport()}>
          <Download data-icon="inline-start" />
          Export Dataset
        </ContextMenuItem>
        <ContextMenuSeparator />
        {viewMode === "trash" ? (
          <>
            <ContextMenuItem onClick={() => onRestore(targetIds)}>
              <RotateCcw data-icon="inline-start" />
              {selectedCount > 1 && includesDataset
                ? `Restore Selected (${selectedCount})`
                : "Restore"}
            </ContextMenuItem>
            <ContextMenuItem
              variant="destructive"
              onClick={() => onPermanentDelete(targetIds)}
            >
              <Trash2 data-icon="inline-start" />
              {selectedCount > 1 && includesDataset
                ? `Permanently Delete Selected (${selectedCount})`
                : "Permanently Delete"}
            </ContextMenuItem>
          </>
        ) : (
          <ContextMenuItem
            variant="destructive"
            onClick={() => onTrash(targetIds)}
          >
            <Trash2 data-icon="inline-start" />
            {selectedCount > 1 && includesDataset
              ? `Move Selected to Trash (${selectedCount})`
              : "Move to Trash"}
          </ContextMenuItem>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
}
