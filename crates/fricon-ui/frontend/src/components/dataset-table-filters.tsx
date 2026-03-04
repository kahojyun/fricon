import type { RefObject } from "react";
import type { Table } from "@tanstack/react-table";
import type { DatasetInfo, DatasetStatus } from "@/lib/backend";
import {
  datasetStatusOptions,
  type DatasetColumnMeta,
} from "@/components/dataset-table-columns";
import { DatasetFilterCheckIcon } from "@/components/dataset-filter-check-icon";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { X } from "lucide-react";

interface DatasetTableFiltersProps {
  table: Table<DatasetInfo>;
  gridTemplateColumns: string;
  headerRef: RefObject<HTMLDivElement | null>;
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
}

export function DatasetTableFilters({
  table,
  gridTemplateColumns,
  headerRef,
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
}: DatasetTableFiltersProps) {
  return (
    <>
      <div className="flex items-center justify-between gap-2 border-y px-3 py-2">
        <div className="flex items-center gap-2">
          <Badge variant="secondary">
            {hasActiveFilters
              ? `${selectedTags.length + selectedStatuses.length + Number(favoriteOnly) + Number(searchQuery.trim().length > 0)} filters`
              : "No filters"}
          </Badge>
          {selectedTags.length > 0 ? (
            <div className="text-xs text-muted-foreground">
              Tags: {selectedTags.join(", ")}
            </div>
          ) : null}
          {selectedStatuses.length > 0 ? (
            <div className="text-xs text-muted-foreground">
              Status: {selectedStatuses.join(", ")}
            </div>
          ) : null}
        </div>
        <Button
          type="button"
          size="sm"
          variant="ghost"
          disabled={!hasActiveFilters}
          onClick={clearFilters}
        >
          <X />
          Clear filters
        </Button>
      </div>

      <div ref={headerRef} className="sticky top-0 z-20 border-b bg-muted">
        <div
          className="grid items-center gap-2 px-3 py-2 text-xs text-muted-foreground"
          style={{ gridTemplateColumns }}
        >
          {table.getFlatHeaders().map((header) => {
            const meta = header.column.columnDef.meta as
              | DatasetColumnMeta
              | undefined;
            const canSort = header.column.getCanSort();
            const sorted = header.column.getIsSorted();
            return (
              <button
                key={header.id}
                type="button"
                className={cn(
                  "flex items-center gap-1 text-left font-medium",
                  canSort ? "hover:text-foreground" : "cursor-default",
                )}
                onClick={
                  canSort ? header.column.getToggleSortingHandler() : undefined
                }
              >
                <span>{meta?.label ?? ""}</span>
                {sorted ? <span>{sorted === "desc" ? "↓" : "↑"}</span> : null}
              </button>
            );
          })}
        </div>

        <div
          className="grid items-start gap-2 border-t px-3 py-2"
          style={{ gridTemplateColumns }}
        >
          {table.getFlatHeaders().map((header) => {
            if (header.column.id === "favorite") {
              return (
                <div key={`${header.id}-filter`} className="pt-1">
                  <Switch
                    aria-label="Favorites only"
                    checked={favoriteOnly}
                    onCheckedChange={setFavoriteOnly}
                  />
                </div>
              );
            }

            if (header.column.id === "name") {
              return (
                <div key={`${header.id}-filter`}>
                  <Input
                    aria-label="Search datasets"
                    value={searchQuery}
                    onChange={(event) => setSearchQuery(event.target.value)}
                    placeholder="Search by name"
                  />
                </div>
              );
            }

            if (header.column.id === "tags") {
              return (
                <div key={`${header.id}-filter`}>
                  <Popover>
                    <PopoverTrigger
                      render={
                        <Button
                          type="button"
                          aria-label="Filter tags"
                          variant={
                            selectedTags.length > 0 ? "secondary" : "outline"
                          }
                          size="sm"
                        />
                      }
                    >
                      Tags
                      {selectedTags.length > 0
                        ? ` (${selectedTags.length})`
                        : ""}
                    </PopoverTrigger>
                    <PopoverContent align="start" className="w-64 gap-2">
                      <Input
                        aria-label="Filter tags"
                        placeholder="Search tags"
                        value={tagFilterQuery}
                        onChange={(event) =>
                          setTagFilterQuery(event.target.value)
                        }
                      />
                      <div className="max-h-48 space-y-1 overflow-auto pr-1">
                        {filteredTagOptions.length > 0 ? (
                          filteredTagOptions.map((tag) => {
                            const isActive = selectedTags.includes(tag);
                            return (
                              <Button
                                key={tag}
                                type="button"
                                variant={isActive ? "secondary" : "ghost"}
                                size="sm"
                                className="w-full justify-start"
                                onClick={() => handleTagToggle(tag)}
                              >
                                <DatasetFilterCheckIcon active={isActive} />
                                {tag}
                              </Button>
                            );
                          })
                        ) : (
                          <div className="px-1 py-2 text-xs text-muted-foreground">
                            No tags found
                          </div>
                        )}
                      </div>
                    </PopoverContent>
                  </Popover>
                </div>
              );
            }

            if (header.column.id === "status") {
              return (
                <div key={`${header.id}-filter`}>
                  <Popover>
                    <PopoverTrigger
                      render={
                        <Button
                          type="button"
                          aria-label="Filter status"
                          variant={
                            selectedStatuses.length > 0
                              ? "secondary"
                              : "outline"
                          }
                          size="sm"
                        />
                      }
                    >
                      Status
                      {selectedStatuses.length > 0
                        ? ` (${selectedStatuses.length})`
                        : ""}
                    </PopoverTrigger>
                    <PopoverContent align="start" className="w-56 space-y-1">
                      {datasetStatusOptions.map((status) => {
                        const isActive = selectedStatuses.includes(status);
                        return (
                          <Button
                            key={status}
                            type="button"
                            variant={isActive ? "secondary" : "ghost"}
                            size="sm"
                            className="w-full justify-start"
                            onClick={() => handleStatusToggle(status)}
                          >
                            <DatasetFilterCheckIcon active={isActive} />
                            {status}
                          </Button>
                        );
                      })}
                    </PopoverContent>
                  </Popover>
                </div>
              );
            }

            return <div key={`${header.id}-filter`} />;
          })}
        </div>
      </div>
    </>
  );
}
