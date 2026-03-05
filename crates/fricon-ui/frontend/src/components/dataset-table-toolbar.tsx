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
    <div className="flex flex-wrap items-center justify-between gap-2 border-b px-3 py-2">
      <div className="flex flex-1 flex-wrap items-center gap-2">
        <Input
          placeholder="Filter datasets..."
          value={searchQuery}
          onChange={(event) => setSearchQuery(event.target.value)}
          className="h-8 w-full max-w-62.5 min-w-37.5"
        />

        <Popover>
          <PopoverTrigger
            render={
              <Button
                variant="outline"
                size="sm"
                className="h-8 border-dashed"
              />
            }
          >
            <PlusCircle className="mr-2 h-4 w-4 shrink-0" />
            Tags
            {selectedTags.length > 0 && (
              <>
                <div className="mx-2 h-4 w-px shrink-0 bg-border" />
                <Badge
                  variant="secondary"
                  className="rounded-sm px-1 font-normal lg:hidden"
                >
                  {selectedTags.length}
                </Badge>
                <div className="hidden space-x-1 lg:flex">
                  {selectedTags.length > 2 ? (
                    <Badge
                      variant="secondary"
                      className="rounded-sm px-1 font-normal"
                    >
                      {selectedTags.length} selected
                    </Badge>
                  ) : (
                    selectedTags.map((tag) => (
                      <Badge
                        key={tag}
                        variant="secondary"
                        className="max-w-24 truncate rounded-sm px-1 font-normal"
                      >
                        {tag}
                      </Badge>
                    ))
                  )}
                </div>
              </>
            )}
          </PopoverTrigger>
          <PopoverContent className="w-64 p-2" align="start">
            <Input
              placeholder="Search tags"
              value={tagFilterQuery}
              onChange={(event) => setTagFilterQuery(event.target.value)}
              className="mb-2 h-8"
            />
            <div className="max-h-50 space-y-1 overflow-auto pr-1">
              {filteredTagOptions.length > 0 ? (
                filteredTagOptions.map((tag) => {
                  const isActive = selectedTags.includes(tag);
                  return (
                    <Button
                      key={tag}
                      type="button"
                      variant={isActive ? "secondary" : "ghost"}
                      size="sm"
                      className="w-full justify-start font-normal"
                      onClick={() => handleTagToggle(tag)}
                    >
                      <DatasetFilterCheckIcon active={isActive} />
                      {tag}
                    </Button>
                  );
                })
              ) : (
                <div className="py-6 text-center text-sm text-muted-foreground">
                  No tags found.
                </div>
              )}
            </div>
            {selectedTags.length > 0 && (
              <>
                <div className="my-2 h-px w-full bg-border" />
                <Button
                  variant="ghost"
                  size="sm"
                  className="w-full justify-center text-sm"
                  onClick={() => {
                    selectedTags.forEach(handleTagToggle);
                  }}
                >
                  Clear filters
                </Button>
              </>
            )}
          </PopoverContent>
        </Popover>

        <Popover>
          <PopoverTrigger
            render={
              <Button
                variant="outline"
                size="sm"
                className="h-8 border-dashed"
              />
            }
          >
            <PlusCircle className="mr-2 h-4 w-4 shrink-0" />
            Status
            {selectedStatuses.length > 0 && (
              <>
                <div className="mx-2 h-4 w-px shrink-0 bg-border" />
                <Badge
                  variant="secondary"
                  className="rounded-sm px-1 font-normal lg:hidden"
                >
                  {selectedStatuses.length}
                </Badge>
                <div className="hidden space-x-1 lg:flex">
                  {selectedStatuses.length > 2 ? (
                    <Badge
                      variant="secondary"
                      className="rounded-sm px-1 font-normal"
                    >
                      {selectedStatuses.length} selected
                    </Badge>
                  ) : (
                    selectedStatuses.map((status) => (
                      <Badge
                        key={status}
                        variant="secondary"
                        className="max-w-24 truncate rounded-sm px-1 font-normal"
                      >
                        {status}
                      </Badge>
                    ))
                  )}
                </div>
              </>
            )}
          </PopoverTrigger>
          <PopoverContent className="w-50 p-2" align="start">
            <div className="space-y-1">
              {datasetStatusOptions.map((status) => {
                const isActive = selectedStatuses.includes(status);
                return (
                  <Button
                    key={status}
                    type="button"
                    variant={isActive ? "secondary" : "ghost"}
                    size="sm"
                    className="w-full justify-start font-normal"
                    onClick={() => handleStatusToggle(status)}
                  >
                    <DatasetFilterCheckIcon active={isActive} />
                    {status}
                  </Button>
                );
              })}
            </div>
            {selectedStatuses.length > 0 && (
              <>
                <div className="my-2 h-px w-full bg-border" />
                <Button
                  variant="ghost"
                  size="sm"
                  className="w-full justify-center text-sm"
                  onClick={() => {
                    selectedStatuses.forEach(handleStatusToggle);
                  }}
                >
                  Clear filters
                </Button>
              </>
            )}
          </PopoverContent>
        </Popover>

        <div className="flex h-8 shrink-0 items-center space-x-2 rounded-md border border-dashed px-2.5">
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
