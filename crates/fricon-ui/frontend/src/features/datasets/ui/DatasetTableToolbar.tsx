import type { Table } from "@tanstack/react-table";
import type { DatasetInfo, DatasetStatus, DatasetViewMode } from "../api/types";
import { DatasetStatusFilter } from "./DatasetStatusFilter";
import { DatasetTableViewOptions } from "./DatasetTableViewOptions";
import { DatasetTagFilter } from "./DatasetTagFilter";
import { Toggle } from "@/shared/ui/toggle";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { X } from "lucide-react";

interface DatasetTableToolbarProps {
  table: Table<DatasetInfo>;
  viewMode: DatasetViewMode;
  hasActiveFilters: boolean;
  activeTags: string[];
  activeStatuses: DatasetStatus[];
  showFavoritesOnly: boolean;
  searchInput: string;
  allTags: string[];
  isUpdatingTags: boolean;
  isMutatingDatasets: boolean;
  setShowFavoritesOnly: (next: boolean) => void;
  setViewMode: (next: DatasetViewMode) => void;
  setSearchInput: (next: string) => void;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  clearFilters: () => void;
  resetColumnVisibilityToDefault: () => void;
  showAllColumns: () => void;
  onColumnVisibilityChange: (columnId: string, visible: boolean) => void;
  onDeleteTag: (tag: string) => Promise<void>;
  onRenameTag: (oldName: string, newName: string) => Promise<void>;
  onMergeTag: (source: string, target: string) => Promise<void>;
  onEmptyTrash: () => void;
}

export function DatasetTableToolbar({
  table,
  viewMode,
  hasActiveFilters,
  activeTags,
  activeStatuses,
  showFavoritesOnly,
  searchInput,
  allTags,
  isUpdatingTags,
  isMutatingDatasets,
  setShowFavoritesOnly,
  setViewMode,
  setSearchInput,
  handleTagToggle,
  handleStatusToggle,
  clearFilters,
  resetColumnVisibilityToDefault,
  showAllColumns,
  onColumnVisibilityChange,
  onDeleteTag,
  onRenameTag,
  onMergeTag,
  onEmptyTrash,
}: DatasetTableToolbarProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-1.5 border-b px-2.5 py-1.5">
      <div className="flex flex-1 flex-wrap items-center gap-1.5">
        <div className="flex items-center gap-1 rounded-md border p-0.5">
          <Button
            variant={viewMode === "active" ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setViewMode("active")}
          >
            Datasets
          </Button>
          <Button
            variant={viewMode === "trash" ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setViewMode("trash")}
          >
            Trash
          </Button>
        </div>

        <Input
          placeholder={
            viewMode === "trash"
              ? "Filter trashed datasets..."
              : "Filter datasets..."
          }
          value={searchInput}
          onChange={(event) => setSearchInput(event.target.value)}
          className="w-full max-w-62.5 min-w-37.5"
        />

        <DatasetTagFilter
          activeTags={activeTags}
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          onToggleTag={handleTagToggle}
          onDeleteTag={onDeleteTag}
          onRenameTag={onRenameTag}
          onMergeTag={onMergeTag}
        />

        <DatasetStatusFilter
          activeStatuses={activeStatuses}
          onToggleStatus={handleStatusToggle}
        />

        <Toggle
          variant="outline"
          pressed={showFavoritesOnly}
          className="border-dashed"
          onPressedChange={setShowFavoritesOnly}
        >
          Favorites Only
        </Toggle>

        {hasActiveFilters && (
          <Button
            variant="ghost"
            onClick={clearFilters}
            className="shrink-0 lg:px-3"
          >
            Reset
            <X data-icon="inline-end" />
          </Button>
        )}

        {viewMode === "trash" && (
          <Button
            variant="outline"
            onClick={onEmptyTrash}
            disabled={isMutatingDatasets}
            className="shrink-0"
          >
            Empty Trash
          </Button>
        )}
      </div>

      <DatasetTableViewOptions
        table={table}
        resetColumnVisibilityToDefault={resetColumnVisibilityToDefault}
        showAllColumns={showAllColumns}
        onColumnVisibilityChange={onColumnVisibilityChange}
      />
    </div>
  );
}
