import type { ReactElement } from "react";
import { Trash2 } from "lucide-react";
import type { DatasetDeleteResult, DatasetInfo } from "../api/types";
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

interface DatasetTableRowActionsProps {
  dataset: DatasetInfo;
  selectedDatasets: DatasetInfo[];
  allTags: string[];
  isUpdatingTags: boolean;
  onDatasetSelected: (id?: number) => void;
  openDeleteDialog: (ids: number[]) => void;
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
  selectedDatasets,
  allTags,
  isUpdatingTags,
  onDatasetSelected,
  openDeleteDialog,
  batchAddTags,
  batchRemoveTags,
  children,
}: DatasetTableRowActionsProps) {
  const target = deriveDatasetTagMenuTarget(dataset, selectedDatasets);
  const selectedCount = selectedDatasets.length;
  const includesDataset = selectedDatasets.some(
    (selectedDataset) => selectedDataset.id === dataset.id,
  );

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
        <ContextMenuItem
          variant="destructive"
          onClick={() => openDeleteDialog([dataset.id])}
        >
          <Trash2 data-icon="inline-start" />
          Delete
        </ContextMenuItem>
        {selectedCount > 1 && includesDataset && (
          <ContextMenuItem
            variant="destructive"
            onClick={() =>
              openDeleteDialog(
                selectedDatasets.map((selectedDataset) => selectedDataset.id),
              )
            }
          >
            <Trash2 data-icon="inline-start" />
            Delete Selected ({selectedCount})
          </ContextMenuItem>
        )}
      </ContextMenuContent>
    </ContextMenu>
  );
}
