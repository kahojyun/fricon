import { useEffect, useMemo, useRef, useState } from "react";
import {
  flexRender,
  getCoreRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useVirtualizer } from "@tanstack/react-virtual";
import {
  createDatasetColumns,
  type DatasetColumnMeta,
} from "@/components/dataset-table-columns";
import { DatasetTableFilters } from "@/components/dataset-table-filters";
import { useDatasetColumnVisibility } from "@/components/use-dataset-column-visibility";
import { useDatasetTableData } from "@/components/use-dataset-table-data";
import { ScrollArea } from "@/components/ui/scroll-area";
import { TooltipProvider } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface DatasetTableProps {
  selectedDatasetId?: number;
  onDatasetSelected: (id: number) => void;
}

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  const {
    datasets,
    searchQuery,
    setSearchQuery,
    selectedTags,
    selectedStatuses,
    tagFilterQuery,
    setTagFilterQuery,
    sorting,
    setSorting,
    filteredTagOptions,
    favoriteOnly,
    setFavoriteOnly,
    hasMore,
    hasActiveFilters,
    toggleFavorite,
    handleTagToggle,
    handleStatusToggle,
    clearFilters,
    loadNextPage,
  } = useDatasetTableData();

  const [headerHeight, setHeaderHeight] = useState(0);
  const [headerScrollbarWidth, setHeaderScrollbarWidth] = useState(0);
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
    const viewport = scrollViewportRef.current;
    const header = headerRef.current;
    if (!viewport || !header) return;

    const syncHeaderScroll = () => {
      header.scrollLeft = viewport.scrollLeft;
    };
    const updateScrollbarWidth = () => {
      setHeaderScrollbarWidth(viewport.offsetWidth - viewport.clientWidth);
    };

    syncHeaderScroll();
    updateScrollbarWidth();

    viewport.addEventListener("scroll", syncHeaderScroll, { passive: true });

    if (typeof ResizeObserver === "undefined") {
      return () => {
        viewport.removeEventListener("scroll", syncHeaderScroll);
      };
    }

    const observer = new ResizeObserver(() => {
      updateScrollbarWidth();
    });
    observer.observe(viewport);

    return () => {
      viewport.removeEventListener("scroll", syncHeaderScroll);
      observer.disconnect();
    };
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

  const columns = useMemo(
    () => createDatasetColumns({ toggleFavorite }),
    [toggleFavorite],
  );
  const {
    columnVisibility,
    resetColumnVisibilityToDefault,
    showAllColumns,
    handleColumnVisibilityChange,
  } = useDatasetColumnVisibility(columns);

  const table = useReactTable({
    data: datasets,
    columns,
    state: {
      sorting,
      columnVisibility,
    },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
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
        <DatasetTableFilters
          table={table}
          gridTemplateColumns={gridTemplateColumns}
          headerRef={headerRef}
          hasActiveFilters={hasActiveFilters}
          selectedTags={selectedTags}
          selectedStatuses={selectedStatuses}
          favoriteOnly={favoriteOnly}
          searchQuery={searchQuery}
          tagFilterQuery={tagFilterQuery}
          filteredTagOptions={filteredTagOptions}
          setFavoriteOnly={setFavoriteOnly}
          setSearchQuery={setSearchQuery}
          setTagFilterQuery={setTagFilterQuery}
          handleTagToggle={handleTagToggle}
          handleStatusToggle={handleStatusToggle}
          clearFilters={clearFilters}
          resetColumnVisibilityToDefault={resetColumnVisibilityToDefault}
          showAllColumns={showAllColumns}
          onColumnVisibilityChange={handleColumnVisibilityChange}
          headerScrollbarWidth={headerScrollbarWidth}
        />
        <div className="flex min-h-0 flex-1 flex-col border-t">
          <ScrollArea ref={scrollRootRef} className="min-h-0 flex-1">
            <div className="min-w-190">
              <div
                className="relative w-full"
                style={{
                  height: rowVirtualizer.getTotalSize() + headerHeight,
                  paddingTop: headerHeight,
                }}
              >
                {rows.length === 0 ? (
                  <div className="absolute inset-x-0 top-30 px-3 text-xs text-muted-foreground">
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
                        <div key={cell.id} className="min-w-0 overflow-hidden">
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
