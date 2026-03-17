import type { Table } from "@tanstack/react-table";
import type { DatasetInfo, DatasetStatus } from "../api/types";
import { DatasetStatusFilter } from "./DatasetStatusFilter";
import { DatasetTableViewOptions } from "./DatasetTableViewOptions";
import { DatasetTagFilter } from "./DatasetTagFilter";
import { Toggle } from "@/shared/ui/toggle";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { X } from "lucide-react";

interface DatasetTableToolbarProps {
  table: Table<DatasetInfo>;
  hasActiveFilters: boolean;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  searchQuery: string;
  allTags: string[];
  isUpdatingTags: boolean;
  setFavoriteOnly: (next: boolean) => void;
  setSearchQuery: (next: string) => void;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  clearFilters: () => void;
  resetColumnVisibilityToDefault: () => void;
  showAllColumns: () => void;
  onColumnVisibilityChange: (columnId: string, visible: boolean) => void;
  onDeleteTag: (tag: string) => Promise<void>;
  onRenameTag: (oldName: string, newName: string) => Promise<void>;
  onMergeTag: (source: string, target: string) => Promise<void>;
}

export function DatasetTableToolbar({
  table,
  hasActiveFilters,
  selectedTags,
  selectedStatuses,
  favoriteOnly,
  searchQuery,
  allTags,
  isUpdatingTags,
  setFavoriteOnly,
  setSearchQuery,
  handleTagToggle,
  handleStatusToggle,
  clearFilters,
  resetColumnVisibilityToDefault,
  showAllColumns,
  onColumnVisibilityChange,
  onDeleteTag,
  onRenameTag,
  onMergeTag,
}: DatasetTableToolbarProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-1.5 border-b px-2.5 py-1.5">
      <div className="flex flex-1 flex-wrap items-center gap-1.5">
        <Input
          placeholder="Filter datasets..."
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
          className="w-full max-w-62.5 min-w-37.5"
        />

        <DatasetTagFilter
          selectedTags={selectedTags}
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          onToggleTag={handleTagToggle}
          onDeleteTag={onDeleteTag}
          onRenameTag={onRenameTag}
          onMergeTag={onMergeTag}
        />

        <DatasetStatusFilter
          selectedStatuses={selectedStatuses}
          onToggleStatus={handleStatusToggle}
        />

        <Toggle
          variant="outline"
          pressed={favoriteOnly}
          className="border-dashed"
          onPressedChange={setFavoriteOnly}
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
