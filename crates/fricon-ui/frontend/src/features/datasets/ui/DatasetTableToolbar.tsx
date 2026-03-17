import type { ReactNode } from "react";
import type { Table } from "@tanstack/react-table";
import type { DatasetInfo, DatasetStatus } from "../api/types";
import type { DatasetColumnMeta } from "../model/datasetColumnMeta";
import { datasetStatusOptions } from "./DatasetTableColumns";
import { DatasetFilterCheckIcon } from "./DatasetFilterCheckIcon";
import { ManageTagsDialog } from "./ManageTagsDialog";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";
import { Input } from "@/shared/ui/input";
import { Popover, PopoverContent, PopoverTrigger } from "@/shared/ui/popover";
import { Badge } from "@/shared/ui/badge";
import { Separator } from "@/shared/ui/separator";
import { Settings2, X, PlusCircle } from "lucide-react";

interface DatasetTableToolbarProps {
  table: Table<DatasetInfo>;
  hasActiveFilters: boolean;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  searchQuery: string;
  tagFilterQuery: string;
  filteredTagOptions: string[];
  allTags: string[];
  isUpdatingTags: boolean;
  setFavoriteOnly: (next: boolean) => void;
  setSearchQuery: (next: string) => void;
  setTagFilterQuery: (next: string) => void;
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

interface FacetedFilterPopoverProps<T extends string> {
  title: string;
  selectedValues: T[];
  options: T[];
  onToggle: (value: T) => void;
  searchValue?: string;
  onSearchChange?: (next: string) => void;
  searchPlaceholder?: string;
  emptyLabel: string;
  contentClassName: string;
  footer?: ReactNode;
}

function FacetedFilterPopover<T extends string>({
  title,
  selectedValues,
  options,
  onToggle,
  searchValue,
  onSearchChange,
  searchPlaceholder,
  emptyLabel,
  contentClassName,
  footer,
}: FacetedFilterPopoverProps<T>) {
  const hasSearch = onSearchChange !== undefined;

  return (
    <Popover>
      <PopoverTrigger
        render={<Button variant="outline" className="border-dashed" />}
      >
        <PlusCircle data-icon="inline-start" />
        {title}
        {selectedValues.length > 0 && (
          <>
            <Separator orientation="vertical" className="mx-2 h-4" />
            <Badge variant="secondary" className="lg:hidden">
              {selectedValues.length}
            </Badge>
            <div className="hidden flex-wrap gap-1 lg:flex">
              {selectedValues.length > 2 ? (
                <Badge variant="secondary">
                  {selectedValues.length} selected
                </Badge>
              ) : (
                selectedValues.map((value) => (
                  <Badge
                    key={value}
                    variant="secondary"
                    className="max-w-24 truncate"
                  >
                    {value}
                  </Badge>
                ))
              )}
            </div>
          </>
        )}
      </PopoverTrigger>
      <PopoverContent className={contentClassName} align="start">
        {hasSearch && (
          <Input
            placeholder={searchPlaceholder}
            value={searchValue}
            onChange={(event) => onSearchChange(event.target.value)}
            className="mb-2"
          />
        )}
        <div className="flex max-h-50 flex-col gap-1 overflow-auto pr-1">
          {options.length > 0 ? (
            options.map((value) => {
              const isActive = selectedValues.includes(value);
              return (
                <Button
                  key={value}
                  type="button"
                  variant={isActive ? "secondary" : "ghost"}
                  className="w-full justify-start"
                  onClick={() => onToggle(value)}
                >
                  <DatasetFilterCheckIcon active={isActive} />
                  {value}
                </Button>
              );
            })
          ) : (
            <div className="py-6 text-center text-sm text-muted-foreground">
              {emptyLabel}
            </div>
          )}
        </div>
        {selectedValues.length > 0 && (
          <>
            <Separator />
            <Button
              variant="ghost"
              className="w-full justify-center"
              onClick={() => {
                selectedValues.forEach(onToggle);
              }}
            >
              Clear filters
            </Button>
          </>
        )}
        {footer}
      </PopoverContent>
    </Popover>
  );
}

export function DatasetTableToolbar({
  table,
  hasActiveFilters,
  selectedTags,
  selectedStatuses,
  favoriteOnly,
  searchQuery,
  tagFilterQuery,
  filteredTagOptions,
  allTags,
  isUpdatingTags,
  setFavoriteOnly,
  setSearchQuery,
  setTagFilterQuery,
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
  const allColumns = table.getAllLeafColumns();
  const handleManagedTagDelete = async (tag: string) => {
    await onDeleteTag(tag);
    if (selectedTags.includes(tag)) {
      handleTagToggle(tag);
    }
  };

  const handleManagedTagRename = async (oldName: string, newName: string) => {
    await onRenameTag(oldName, newName);
    if (!selectedTags.includes(oldName)) {
      return;
    }
    handleTagToggle(oldName);
    if (!selectedTags.includes(newName)) {
      handleTagToggle(newName);
    }
  };

  const handleManagedTagMerge = async (source: string, target: string) => {
    await onMergeTag(source, target);
    if (!selectedTags.includes(source)) {
      return;
    }
    handleTagToggle(source);
    if (!selectedTags.includes(target)) {
      handleTagToggle(target);
    }
  };

  return (
    <div className="flex flex-wrap items-center justify-between gap-1.5 border-b px-2.5 py-1.5">
      <div className="flex flex-1 flex-wrap items-center gap-1.5">
        <Input
          placeholder="Filter datasets..."
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
          className="w-full max-w-62.5 min-w-37.5"
        />

        <FacetedFilterPopover
          title="Tags"
          selectedValues={selectedTags}
          options={filteredTagOptions}
          onToggle={handleTagToggle}
          searchValue={tagFilterQuery}
          onSearchChange={setTagFilterQuery}
          searchPlaceholder="Search tags"
          emptyLabel="No tags found."
          contentClassName="w-64 p-2 gap-1"
          footer={
            <>
              <Separator />
              <ManageTagsDialog
                allTags={allTags}
                isUpdatingTags={isUpdatingTags}
                onDeleteTag={handleManagedTagDelete}
                onRenameTag={handleManagedTagRename}
                onMergeTag={handleManagedTagMerge}
              />
            </>
          }
        />

        <FacetedFilterPopover
          title="Status"
          selectedValues={selectedStatuses}
          options={datasetStatusOptions}
          onToggle={handleStatusToggle}
          emptyLabel="No status found."
          contentClassName="w-50 p-2"
        />

        <Button
          type="button"
          variant={favoriteOnly ? "secondary" : "outline"}
          aria-pressed={favoriteOnly}
          className="shrink-0 border-dashed"
          onClick={() => setFavoriteOnly(!favoriteOnly)}
        >
          Favorites Only
        </Button>

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

      <DropdownMenu>
        <DropdownMenuTrigger
          render={<Button variant="outline" className="ml-auto shrink-0" />}
        >
          <Settings2 data-icon="inline-start" />
          View
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="w-42">
          <DropdownMenuGroup>
            <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
            <DropdownMenuSeparator />
            {allColumns.map((column) => {
              const meta = column.columnDef.meta as
                | DatasetColumnMeta
                | undefined;
              const label = meta?.label ?? column.id;
              const hideable = meta?.hideable ?? true;
              return (
                <DropdownMenuCheckboxItem
                  key={column.id}
                  checked={column.getIsVisible()}
                  disabled={!hideable}
                  onCheckedChange={(checked) => {
                    if (!hideable) return;
                    onColumnVisibilityChange(column.id, checked);
                  }}
                >
                  {label}
                </DropdownMenuCheckboxItem>
              );
            })}
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={showAllColumns}>Show All</DropdownMenuItem>
          <DropdownMenuItem onClick={resetColumnVisibilityToDefault}>
            Reset Default
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}
