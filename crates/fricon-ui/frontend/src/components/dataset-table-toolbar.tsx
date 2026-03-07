import type { Table } from "@tanstack/react-table";
import type { DatasetInfo, DatasetStatus } from "@/lib/backend";
import {
  datasetStatusOptions,
  type DatasetColumnMeta,
} from "@/components/dataset-table-columns";
import { DatasetFilterCheckIcon } from "@/components/dataset-filter-check-icon";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { Checkbox } from "@/components/ui/checkbox";
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
  setFavoriteOnly: (next: boolean) => void;
  setSearchQuery: (next: string) => void;
  setTagFilterQuery: (next: string) => void;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  clearFilters: () => void;
  resetColumnVisibilityToDefault: () => void;
  showAllColumns: () => void;
  onColumnVisibilityChange: (columnId: string, visible: boolean) => void;
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
}: FacetedFilterPopoverProps<T>) {
  const hasSearch = onSearchChange !== undefined;

  return (
    <Popover>
      <PopoverTrigger
        render={
          <Button variant="outline" size="sm" className="h-8 border-dashed" />
        }
      >
        <PlusCircle className="mr-2 h-4 w-4 shrink-0" />
        {title}
        {selectedValues.length > 0 && (
          <>
            <div className="mx-2 h-4 w-px shrink-0 bg-border" />
            <Badge
              variant="secondary"
              className="rounded-sm px-1 font-normal lg:hidden"
            >
              {selectedValues.length}
            </Badge>
            <div className="hidden space-x-1 lg:flex">
              {selectedValues.length > 2 ? (
                <Badge
                  variant="secondary"
                  className="rounded-sm px-1 font-normal"
                >
                  {selectedValues.length} selected
                </Badge>
              ) : (
                selectedValues.map((value) => (
                  <Badge
                    key={value}
                    variant="secondary"
                    className="max-w-24 truncate rounded-sm px-1 font-normal"
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
            className="mb-2 h-8"
          />
        )}
        <div className="max-h-50 space-y-1 overflow-auto pr-1">
          {options.length > 0 ? (
            options.map((value) => {
              const isActive = selectedValues.includes(value);
              return (
                <Button
                  key={value}
                  type="button"
                  variant={isActive ? "secondary" : "ghost"}
                  size="sm"
                  className="w-full justify-start font-normal"
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
            <div className="my-2 h-px w-full bg-border" />
            <Button
              variant="ghost"
              size="sm"
              className="w-full justify-center text-sm"
              onClick={() => {
                selectedValues.forEach(onToggle);
              }}
            >
              Clear filters
            </Button>
          </>
        )}
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
  setFavoriteOnly,
  setSearchQuery,
  setTagFilterQuery,
  handleTagToggle,
  handleStatusToggle,
  clearFilters,
  resetColumnVisibilityToDefault,
  showAllColumns,
  onColumnVisibilityChange,
}: DatasetTableToolbarProps) {
  const allColumns = table.getAllLeafColumns();

  return (
    <div className="flex flex-wrap items-center justify-between gap-1.5 border-b px-2.5 py-1.5">
      <div className="flex flex-1 flex-wrap items-center gap-1.5">
        <Input
          placeholder="Filter datasets..."
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
          className="h-7 w-full max-w-62.5 min-w-37.5 text-xs"
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
          contentClassName="w-64 p-2"
        />

        <FacetedFilterPopover
          title="Status"
          selectedValues={selectedStatuses}
          options={datasetStatusOptions}
          onToggle={handleStatusToggle}
          emptyLabel="No status found."
          contentClassName="w-50 p-2"
        />

        <div className="flex h-7 shrink-0 items-center space-x-2 rounded-md border border-dashed px-2">
          <Switch
            id="favorite-toggle"
            checked={favoriteOnly}
            onCheckedChange={setFavoriteOnly}
          />
          <Label
            htmlFor="favorite-toggle"
            className="cursor-pointer text-sm font-normal whitespace-nowrap"
          >
            Favorites Only
          </Label>
        </div>

        {hasActiveFilters && (
          <Button
            variant="ghost"
            onClick={clearFilters}
            className="h-8 shrink-0 px-2 lg:px-3"
          >
            Reset
            <X className="ml-2 h-4 w-4 shrink-0" />
          </Button>
        )}
      </div>

      <Popover>
        <PopoverTrigger
          render={
            <Button
              variant="outline"
              size="sm"
              className="ml-auto flex h-8 shrink-0"
            />
          }
        >
          <Settings2 className="mr-2 h-4 w-4 shrink-0" />
          View
        </PopoverTrigger>
        <PopoverContent align="end" className="w-37.5 p-2">
          <div className="mb-2 px-2 text-xs font-medium">Toggle columns</div>
          <div className="space-y-1">
            {allColumns.map((column) => {
              const meta = column.columnDef.meta as
                | DatasetColumnMeta
                | undefined;
              const label = meta?.label ?? column.id;
              const hideable = meta?.hideable ?? true;
              return (
                <label
                  key={column.id}
                  className="flex w-full cursor-pointer items-center space-x-2 rounded-sm px-2 py-1 hover:bg-muted/50"
                >
                  <Checkbox
                    aria-label={`Toggle ${label} column`}
                    checked={column.getIsVisible()}
                    disabled={!hideable}
                    onCheckedChange={() => {
                      if (!hideable) return;
                      onColumnVisibilityChange(
                        column.id,
                        !column.getIsVisible(),
                      );
                    }}
                  />
                  <span className="flex-1 truncate text-sm font-normal">
                    {label}
                  </span>
                  {!hideable && (
                    <span className="ml-auto text-[10px] text-muted-foreground">
                      Req
                    </span>
                  )}
                </label>
              );
            })}
          </div>
          <div className="my-2 h-px w-full bg-border" />
          <div className="flex flex-col gap-1">
            <Button
              variant="ghost"
              size="sm"
              onClick={showAllColumns}
              className="h-8 w-full justify-start px-2"
            >
              Show All
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={resetColumnVisibilityToDefault}
              className="h-8 w-full justify-start px-2"
            >
              Reset Default
            </Button>
          </div>
        </PopoverContent>
      </Popover>
    </div>
  );
}
