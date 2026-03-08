import type { Table } from "@tanstack/react-table";
import type { DatasetInfo, DatasetStatus } from "@/lib/backend";
import {
  datasetStatusOptions,
  type DatasetColumnMeta,
} from "@/components/dataset-table-columns";
import { DatasetFilterCheckIcon } from "@/components/dataset-filter-check-icon";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
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
            <Separator className="my-2" />
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
