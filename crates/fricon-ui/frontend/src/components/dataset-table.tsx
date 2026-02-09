import { useEffect, useMemo, useRef, useState } from "react";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import { type DatasetInfo, type DatasetStatus } from "@/lib/backend";
import { useDatasetTableData } from "@/components/use-dataset-table-data";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";
import { Check, Star, StarOff, X } from "lucide-react";

interface DatasetTableProps {
  selectedDatasetId?: number;
  onDatasetSelected: (id: number) => void;
}

interface DatasetColumnMeta {
  label: string;
  width: string;
}

const statusVariantMap: Record<
  DatasetStatus,
  "default" | "secondary" | "destructive"
> = {
  Writing: "secondary",
  Completed: "default",
  Aborted: "destructive",
};

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  const {
    datasets,
    searchQuery,
    setSearchQuery,
    selectedTags,
    tagFilterQuery,
    setTagFilterQuery,
    sorting,
    setSorting,
    columnFilters,
    setColumnFilters,
    filteredTagOptions,
    favoriteOnly,
    hasMore,
    hasActiveFilters,
    toggleFavorite,
    handleTagToggle,
    clearFilters,
    loadNextPage,
  } = useDatasetTableData();

  const [headerHeight, setHeaderHeight] = useState(0);
  const headerRef = useRef<HTMLDivElement | null>(null);
  const scrollRootRef = useRef<HTMLDivElement | null>(null);
  const scrollViewportRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (!scrollRootRef.current) return;
    scrollViewportRef.current = scrollRootRef.current.querySelector(
      '[data-slot="scroll-area-viewport"]',
    );
  }, []);

  useEffect(() => {
    const header = headerRef.current;
    if (!header) return;

    const measure = () => {
      setHeaderHeight(header.getBoundingClientRect().height);
    };

    measure();

    if (typeof ResizeObserver === "undefined") {
      return;
    }

    const observer = new ResizeObserver(() => {
      measure();
    });
    observer.observe(header);

    return () => {
      observer.disconnect();
    };
  }, []);

  const columns = useMemo<ColumnDef<DatasetInfo>[]>(
    () => [
      {
        id: "favorite",
        accessorKey: "favorite",
        enableSorting: false,
        filterFn: (row, columnId, value) => {
          if (value !== true) return true;
          return row.getValue<boolean>(columnId);
        },
        meta: { label: "Favorite", width: "60px" } satisfies DatasetColumnMeta,
        cell: ({ row }) => {
          const dataset = row.original;
          const tooltipLabel = dataset.favorite
            ? "Remove from favorites"
            : "Add to favorites";
          return (
            <Tooltip>
              <TooltipTrigger
                render={
                  <Button
                    type="button"
                    aria-label={tooltipLabel}
                    variant="ghost"
                    size="icon-sm"
                    onClick={(event) => {
                      event.stopPropagation();
                      void toggleFavorite(dataset);
                    }}
                  />
                }
              >
                {dataset.favorite ? (
                  <Star className="text-yellow-500" />
                ) : (
                  <StarOff className="text-muted-foreground" />
                )}
              </TooltipTrigger>
              <TooltipContent>{tooltipLabel}</TooltipContent>
            </Tooltip>
          );
        },
      },
      {
        id: "id",
        accessorKey: "id",
        meta: { label: "ID", width: "70px" } satisfies DatasetColumnMeta,
        cell: ({ getValue }) => (
          <div className="px-2 tabular-nums">{getValue<number>()}</div>
        ),
      },
      {
        id: "name",
        accessorKey: "name",
        meta: {
          label: "Name",
          width: "minmax(160px,1fr)",
        } satisfies DatasetColumnMeta,
        cell: ({ getValue }) => (
          <div className="truncate px-2">{getValue<string>()}</div>
        ),
      },
      {
        id: "status",
        accessorKey: "status",
        enableSorting: false,
        meta: { label: "Status", width: "120px" } satisfies DatasetColumnMeta,
        cell: ({ getValue }) => (
          <div className="px-2">
            <Badge variant={statusVariantMap[getValue<DatasetStatus>()]}>
              {getValue<string>()}
            </Badge>
          </div>
        ),
      },
      {
        id: "tags",
        accessorKey: "tags",
        enableSorting: false,
        meta: {
          label: "Tags",
          width: "minmax(140px,1fr)",
        } satisfies DatasetColumnMeta,
        cell: ({ getValue }) => {
          const tags = getValue<string[]>();
          return (
            <div className="flex flex-wrap gap-1 px-2">
              {tags.length > 0 ? (
                tags.map((tag) => (
                  <Badge key={tag} variant="secondary">
                    {tag}
                  </Badge>
                ))
              ) : (
                <span className="text-muted-foreground text-xs">No tags</span>
              )}
            </div>
          );
        },
      },
      {
        id: "createdAt",
        accessorKey: "createdAt",
        meta: {
          label: "Created At",
          width: "160px",
        } satisfies DatasetColumnMeta,
        cell: ({ getValue }) => (
          <div className="text-muted-foreground px-2 text-xs">
            {getValue<Date>().toLocaleString()}
          </div>
        ),
      },
    ],
    [toggleFavorite],
  );

  const table = useReactTable({
    data: datasets,
    columns,
    state: {
      columnFilters,
      sorting,
    },
    onColumnFiltersChange: setColumnFilters,
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  const rows = table.getRowModel().rows;
  const gridTemplateColumns = table
    .getVisibleLeafColumns()
    .map((column) => {
      const meta = column.columnDef.meta as DatasetColumnMeta | undefined;
      return meta?.width ?? "minmax(120px, 1fr)";
    })
    .join(" ");

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollViewportRef.current,
    estimateSize: () => 56,
    overscan: 8,
  });

  useEffect(() => {
    const last = rowVirtualizer.getVirtualItems().at(-1);
    if (!last) return;
    if (hasMore && last.index >= rows.length - 10) {
      void loadNextPage();
    }
  }, [hasMore, loadNextPage, rowVirtualizer, rows.length]);

  return (
    <TooltipProvider>
      <div className="flex h-full min-h-0 flex-col">
        <div className="flex items-center justify-between gap-2 border-y px-3 py-2">
          <div className="flex items-center gap-2">
            <Badge variant="secondary">
              {hasActiveFilters
                ? `${selectedTags.length + Number(favoriteOnly) + Number(searchQuery.trim().length > 0)} filters`
                : "No filters"}
            </Badge>
            {selectedTags.length > 0 ? (
              <div className="text-muted-foreground text-xs">
                Tags: {selectedTags.join(", ")}
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
        <div className="flex min-h-0 flex-1 flex-col border-t">
          <ScrollArea ref={scrollRootRef} className="min-h-0 flex-1">
            <div className="min-w-[760px]">
              <div
                ref={headerRef}
                className="bg-muted sticky top-0 z-20 border-b"
              >
                <div
                  className="text-muted-foreground grid items-center gap-2 px-3 py-2 text-xs"
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
                          canSort
                            ? header.column.getToggleSortingHandler()
                            : undefined
                        }
                      >
                        <span>{meta?.label ?? ""}</span>
                        {sorted ? (
                          <span>{sorted === "desc" ? "↓" : "↑"}</span>
                        ) : null}
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
                            onCheckedChange={(checked) => {
                              setColumnFilters((prev) => {
                                const next = prev.filter(
                                  (filter) => filter.id !== "favorite",
                                );
                                if (checked) {
                                  next.push({ id: "favorite", value: true });
                                }
                                return next;
                              });
                            }}
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
                            onChange={(event) =>
                              setSearchQuery(event.target.value)
                            }
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
                                    selectedTags.length > 0
                                      ? "secondary"
                                      : "outline"
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
                            <PopoverContent
                              align="start"
                              className="w-64 gap-2"
                            >
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
                                        variant={
                                          isActive ? "secondary" : "ghost"
                                        }
                                        size="sm"
                                        className="w-full justify-start"
                                        onClick={() => handleTagToggle(tag)}
                                      >
                                        <Check
                                          className={cn(
                                            "size-3",
                                            isActive
                                              ? "text-foreground opacity-100"
                                              : "text-transparent opacity-0",
                                          )}
                                        />
                                        {tag}
                                      </Button>
                                    );
                                  })
                                ) : (
                                  <div className="text-muted-foreground px-1 py-2 text-xs">
                                    No tags found
                                  </div>
                                )}
                              </div>
                            </PopoverContent>
                          </Popover>
                        </div>
                      );
                    }

                    return <div key={`${header.id}-filter`} />;
                  })}
                </div>
              </div>

              <div
                className="relative w-full"
                style={{
                  height: rowVirtualizer.getTotalSize() + headerHeight,
                  paddingTop: headerHeight,
                }}
              >
                {rows.length === 0 ? (
                  <div className="text-muted-foreground absolute inset-x-0 top-[120px] px-3 text-xs">
                    No datasets matched the current filters.
                  </div>
                ) : null}
                {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                  const row = rows[virtualRow.index];
                  if (!row) return null;
                  const dataset = row.original;
                  const isSelected = dataset.id === selectedDatasetId;
                  return (
                    <div
                      key={row.id}
                      ref={rowVirtualizer.measureElement}
                      data-index={virtualRow.index}
                      className={cn(
                        "grid items-center gap-2 border-b px-3 py-2 text-sm",
                        isSelected ? "bg-primary/10" : "hover:bg-muted/50",
                      )}
                      style={{
                        gridTemplateColumns,
                        position: "absolute",
                        top: 0,
                        left: 0,
                        width: "100%",
                        transform: `translateY(${virtualRow.start}px)`,
                      }}
                      onClick={() => onDatasetSelected(dataset.id)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={(event) => {
                        if (event.key === "Enter" || event.key === " ") {
                          onDatasetSelected(dataset.id);
                        }
                        if (event.metaKey || event.ctrlKey) {
                          event.stopPropagation();
                        }
                      }}
                    >
                      {row.getVisibleCells().map((cell) => (
                        <div key={cell.id} className="min-w-0">
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext(),
                          )}
                        </div>
                      ))}
                    </div>
                  );
                })}
              </div>
            </div>
          </ScrollArea>
        </div>
      </div>
    </TooltipProvider>
  );
}
