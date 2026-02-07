import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  type ColumnDef,
  type ColumnFiltersState,
  type SortingState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  DATASET_PAGE_SIZE,
  type DatasetInfo,
  type DatasetStatus,
  listDatasets,
  onDatasetCreated,
  onDatasetUpdated,
  updateDatasetFavorite,
} from "@/lib/backend";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { Star, StarOff } from "lucide-react";

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

const headerHeight = 96;

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  const [datasets, setDatasets] = useState<DatasetInfo[]>([]);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [hasMore, setHasMore] = useState(true);
  const [sorting, setSorting] = useState<SortingState>([
    { id: "createdAt", desc: true },
  ]);
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([]);

  const datasetsRef = useRef<DatasetInfo[]>([]);
  const searchRef = useRef("");
  const selectedTagsRef = useRef<string[]>([]);
  const isLoadingRef = useRef(false);
  const hasMoreRef = useRef(true);
  const searchDebounce = useRef<number | null>(null);
  const statusRefreshTimer = useRef<number | null>(null);
  const scrollRootRef = useRef<HTMLDivElement | null>(null);
  const scrollViewportRef = useRef<HTMLDivElement | null>(null);

  const setDatasetsState = useCallback((next: DatasetInfo[]) => {
    datasetsRef.current = next;
    setDatasets(next);
  }, []);

  const setHasMoreState = useCallback((next: boolean) => {
    hasMoreRef.current = next;
    setHasMore(next);
  }, []);

  const setIsLoadingState = useCallback((next: boolean) => {
    isLoadingRef.current = next;
  }, []);

  useEffect(() => {
    searchRef.current = searchQuery;
  }, [searchQuery]);

  useEffect(() => {
    selectedTagsRef.current = selectedTags;
  }, [selectedTags]);

  useEffect(() => {
    if (!scrollRootRef.current) return;
    scrollViewportRef.current = scrollRootRef.current.querySelector(
      '[data-slot="scroll-area-viewport"]',
    );
  }, []);

  const loadDatasets = useCallback(
    async ({ append = false } = {}) => {
      if (isLoadingRef.current || (append && !hasMoreRef.current)) return;
      setIsLoadingState(true);
      try {
        const offset = append ? datasetsRef.current.length : 0;
        const next = await listDatasets(
          searchRef.current,
          selectedTagsRef.current,
          DATASET_PAGE_SIZE,
          offset,
        );
        setHasMoreState(next.length === DATASET_PAGE_SIZE);
        if (append) {
          setDatasetsState([...datasetsRef.current, ...next]);
        } else {
          setDatasetsState(next);
        }
      } finally {
        setIsLoadingState(false);
      }
    },
    [setDatasetsState, setHasMoreState, setIsLoadingState],
  );

  const refreshDatasets = useCallback(async () => {
    if (isLoadingRef.current) return;
    setIsLoadingState(true);
    try {
      const limit = Math.max(datasetsRef.current.length, DATASET_PAGE_SIZE);
      const next = await listDatasets(
        searchRef.current,
        selectedTagsRef.current,
        limit,
        0,
      );
      setDatasetsState(next);
      setHasMoreState(next.length >= limit);
    } finally {
      setIsLoadingState(false);
    }
  }, [setDatasetsState, setHasMoreState, setIsLoadingState]);

  useEffect(() => {
    void loadDatasets();

    let unlistenCreated: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;
    let active = true;

    void onDatasetCreated((event) => {
      if (!active) return;
      setDatasetsState([event, ...datasetsRef.current]);
      if (searchRef.current.trim() || selectedTagsRef.current.length > 0) {
        void loadDatasets();
      }
    }).then((unlisten) => {
      unlistenCreated = unlisten;
    });

    void onDatasetUpdated((event) => {
      if (!active) return;
      const next = [...datasetsRef.current];
      const index = next.findIndex((dataset) => dataset.id === event.id);
      if (index >= 0) {
        next[index] = event;
        setDatasetsState(next);
        return;
      }
      if (!searchRef.current.trim() && selectedTagsRef.current.length === 0) {
        setDatasetsState([event, ...next]);
      }
    }).then((unlisten) => {
      unlistenUpdated = unlisten;
    });

    return () => {
      active = false;
      unlistenCreated?.();
      unlistenUpdated?.();
    };
  }, [loadDatasets, setDatasetsState]);

  useEffect(() => {
    if (searchDebounce.current) {
      window.clearTimeout(searchDebounce.current);
    }
    searchDebounce.current = window.setTimeout(() => {
      setHasMoreState(true);
      void loadDatasets();
    }, 300);
    return () => {
      if (searchDebounce.current) {
        window.clearTimeout(searchDebounce.current);
      }
    };
  }, [loadDatasets, searchQuery, selectedTags, setHasMoreState]);

  useEffect(() => {
    const hasWriting = datasets.some((dataset) => dataset.status === "Writing");
    if (hasWriting && statusRefreshTimer.current == null) {
      statusRefreshTimer.current = window.setInterval(() => {
        void refreshDatasets();
      }, 2000);
    }
    if (!hasWriting && statusRefreshTimer.current != null) {
      window.clearInterval(statusRefreshTimer.current);
      statusRefreshTimer.current = null;
    }
    return () => {
      if (statusRefreshTimer.current != null) {
        window.clearInterval(statusRefreshTimer.current);
        statusRefreshTimer.current = null;
      }
    };
  }, [datasets, refreshDatasets]);

  const toggleFavorite = useCallback(
    async (dataset: DatasetInfo) => {
      const nextFavorite = !dataset.favorite;
      setDatasetsState(
        datasetsRef.current.map((item) =>
          item.id === dataset.id ? { ...item, favorite: nextFavorite } : item,
        ),
      );
      try {
        await updateDatasetFavorite(dataset.id, nextFavorite);
      } catch {
        setDatasetsState(
          datasetsRef.current.map((item) =>
            item.id === dataset.id
              ? { ...item, favorite: dataset.favorite }
              : item,
          ),
        );
      }
    },
    [setDatasetsState],
  );

  const tagOptions = useMemo(() => {
    const tagSet = new Set<string>();
    datasets.forEach((dataset) => {
      dataset.tags.forEach((tag) => tagSet.add(tag));
    });
    return Array.from(tagSet).sort((a, b) => a.localeCompare(b));
  }, [datasets]);

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
          return (
            <Button
              type="button"
              variant="ghost"
              size="icon-sm"
              onClick={(event) => {
                event.stopPropagation();
                void toggleFavorite(dataset);
              }}
            >
              {dataset.favorite ? (
                <Star className="text-yellow-500" />
              ) : (
                <StarOff className="text-muted-foreground" />
              )}
            </Button>
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

  const favoriteOnly =
    columnFilters.find((filter) => filter.id === "favorite")?.value === true;

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
      void loadDatasets({ append: true });
    }
  }, [hasMore, loadDatasets, rowVirtualizer, rows.length]);

  const handleTagToggle = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((item) => item !== tag) : [...prev, tag],
    );
  };

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex min-h-0 flex-1 flex-col border-t">
        <ScrollArea ref={scrollRootRef} className="min-h-0 flex-1">
          <div className="min-w-[760px]">
            <div className="bg-muted sticky top-0 z-20 border-b">
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
                      <div
                        key={`${header.id}-filter`}
                        className="flex flex-wrap gap-1"
                      >
                        {tagOptions.map((tag) => {
                          const isActive = selectedTags.includes(tag);
                          return (
                            <Button
                              key={tag}
                              type="button"
                              variant={isActive ? "secondary" : "outline"}
                              size="xs"
                              onClick={() => handleTagToggle(tag)}
                            >
                              {tag}
                            </Button>
                          );
                        })}
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
  );
}
