import { useEffect, useMemo, useRef, useState } from "react";
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
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { Star, StarOff } from "lucide-react";

interface DatasetTableProps {
  selectedDatasetId?: number;
  onDatasetSelected: (id: number) => void;
}

const statusVariantMap: Record<DatasetStatus, "info" | "success" | "danger"> = {
  Writing: "info",
  Completed: "success",
  Aborted: "danger",
};

export function DatasetTable({
  selectedDatasetId,
  onDatasetSelected,
}: DatasetTableProps) {
  const [datasets, setDatasets] = useState<DatasetInfo[]>([]);
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const searchDebounce = useRef<number | null>(null);
  const statusRefreshTimer = useRef<number | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  const tagOptions = useMemo(() => {
    const tagSet = new Set<string>();
    datasets.forEach((dataset) => {
      dataset.tags.forEach((tag) => tagSet.add(tag));
    });
    return Array.from(tagSet).sort((a, b) => a.localeCompare(b));
  }, [datasets]);

  const filteredDatasets = useMemo(() => {
    return favoritesOnly
      ? datasets.filter((dataset) => dataset.favorite)
      : datasets;
  }, [datasets, favoritesOnly]);

  const loadDatasets = async ({ append = false } = {}) => {
    if (isLoading || (append && !hasMore)) return;
    setIsLoading(true);
    try {
      const offset = append ? datasets.length : 0;
      const next = await listDatasets(
        searchQuery,
        selectedTags,
        DATASET_PAGE_SIZE,
        offset,
      );
      setHasMore(next.length === DATASET_PAGE_SIZE);
      setDatasets((prev) => (append ? [...prev, ...next] : next));
    } finally {
      setIsLoading(false);
    }
  };

  const refreshDatasets = async () => {
    if (isLoading) return;
    setIsLoading(true);
    try {
      const limit = Math.max(datasets.length, DATASET_PAGE_SIZE);
      const next = await listDatasets(searchQuery, selectedTags, limit, 0);
      setDatasets(next);
      setHasMore(next.length >= limit);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    void loadDatasets();

    let unlistenCreated: (() => void) | undefined;
    let unlistenUpdated: (() => void) | undefined;
    let active = true;

    onDatasetCreated((event) => {
      if (!active) return;
      setDatasets((prev) => [event, ...prev]);
      if (searchQuery.trim() || selectedTags.length > 0) {
        void loadDatasets();
      }
    }).then((unlisten) => {
      unlistenCreated = unlisten;
    });

    onDatasetUpdated((event) => {
      if (!active) return;
      setDatasets((prev) => {
        const index = prev.findIndex((dataset) => dataset.id === event.id);
        if (index >= 0) {
          const next = [...prev];
          next[index] = event;
          return next;
        }
        if (!searchQuery.trim() && selectedTags.length === 0) {
          return [event, ...prev];
        }
        return prev;
      });
    }).then((unlisten) => {
      unlistenUpdated = unlisten;
    });

    return () => {
      active = false;
      unlistenCreated?.();
      unlistenUpdated?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (searchDebounce.current) {
      window.clearTimeout(searchDebounce.current);
    }
    searchDebounce.current = window.setTimeout(() => {
      setHasMore(true);
      void loadDatasets();
    }, 300);
    return () => {
      if (searchDebounce.current) {
        window.clearTimeout(searchDebounce.current);
      }
    };
  }, [searchQuery, selectedTags]);

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
  }, [datasets]);

  const rowVirtualizer = useVirtualizer({
    count: filteredDatasets.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 40,
    overscan: 8,
  });

  useEffect(() => {
    const last = rowVirtualizer.getVirtualItems().at(-1);
    if (!last) return;
    if (hasMore && last.index >= filteredDatasets.length - 10) {
      void loadDatasets({ append: true });
    }
  }, [filteredDatasets.length, hasMore, rowVirtualizer]);

  const toggleFavorite = async (dataset: DatasetInfo) => {
    const nextFavorite = !dataset.favorite;
    setDatasets((prev) =>
      prev.map((item) =>
        item.id === dataset.id ? { ...item, favorite: nextFavorite } : item,
      ),
    );
    try {
      await updateDatasetFavorite(dataset.id, nextFavorite);
    } catch (error) {
      setDatasets((prev) =>
        prev.map((item) =>
          item.id === dataset.id
            ? { ...item, favorite: dataset.favorite }
            : item,
        ),
      );
      throw error;
    }
  };

  const handleTagToggle = (tag: string) => {
    setSelectedTags((prev) =>
      prev.includes(tag) ? prev.filter((item) => item !== tag) : [...prev, tag],
    );
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex flex-wrap items-center gap-3 p-3">
        <div className="flex items-center gap-2">
          <Switch checked={favoritesOnly} onCheckedChange={setFavoritesOnly} />
          <span className="text-sm">Favorites only</span>
        </div>
        <div className="min-w-[200px] flex-1">
          <Input
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
            placeholder="Search by name"
          />
        </div>
      </div>

      {tagOptions.length > 0 ? (
        <div className="flex flex-wrap gap-2 px-3 pb-3">
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
      ) : null}

      <div className="border-t">
        <div className="bg-muted/40 text-muted-foreground grid grid-cols-[60px_70px_minmax(160px,1fr)_120px_minmax(140px,1fr)_160px] gap-2 border-b px-3 py-2 text-xs font-semibold">
          <div>Favorite</div>
          <div>ID</div>
          <div>Name</div>
          <div>Status</div>
          <div>Tags</div>
          <div>Created At</div>
        </div>
        <div ref={scrollRef} className="h-[calc(100vh-190px)] overflow-auto">
          <div
            className="relative w-full"
            style={{ height: rowVirtualizer.getTotalSize() }}
          >
            {rowVirtualizer.getVirtualItems().map((virtualRow) => {
              const dataset = filteredDatasets[virtualRow.index];
              if (!dataset) return null;
              const isSelected = dataset.id === selectedDatasetId;
              return (
                <div
                  key={dataset.id}
                  className={cn(
                    "grid grid-cols-[60px_70px_minmax(160px,1fr)_120px_minmax(140px,1fr)_160px] items-center gap-2 border-b px-3 py-2 text-sm",
                    isSelected ? "bg-primary/10" : "hover:bg-muted/50",
                  )}
                  style={{
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
                  <div className="tabular-nums">{dataset.id}</div>
                  <div className="truncate">{dataset.name}</div>
                  <div>
                    <Badge variant={statusVariantMap[dataset.status]}>
                      {dataset.status}
                    </Badge>
                  </div>
                  <div className="flex flex-wrap gap-1">
                    {dataset.tags.length > 0 ? (
                      dataset.tags.map((tag) => (
                        <Badge key={tag} variant="secondary">
                          {tag}
                        </Badge>
                      ))
                    ) : (
                      <span className="text-muted-foreground text-xs">
                        No tags
                      </span>
                    )}
                  </div>
                  <div className="text-muted-foreground text-xs">
                    {dataset.createdAt.toLocaleString()}
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
