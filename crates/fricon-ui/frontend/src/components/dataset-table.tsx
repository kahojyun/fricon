import { useCallback, useEffect, useMemo, useRef, useState } from "react";
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
  const [datasets, setDatasets] = useState<DatasetInfo[]>([]);
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);
  const datasetsRef = useRef<DatasetInfo[]>([]);
  const searchRef = useRef("");
  const selectedTagsRef = useRef<string[]>([]);
  const isLoadingRef = useRef(false);
  const hasMoreRef = useRef(true);
  const searchDebounce = useRef<number | null>(null);
  const statusRefreshTimer = useRef<number | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  const setDatasetsState = useCallback((next: DatasetInfo[]) => {
    datasetsRef.current = next;
    setDatasets(next);
  }, []);

  const setHasMoreState = useCallback((next: boolean) => {
    hasMoreRef.current = next;
    setHasMore(next);
  }, []);

  useEffect(() => {
    searchRef.current = searchQuery;
  }, [searchQuery]);

  useEffect(() => {
    selectedTagsRef.current = selectedTags;
  }, [selectedTags]);

  useEffect(() => {
    isLoadingRef.current = isLoading;
  }, [isLoading]);

  useEffect(() => {
    hasMoreRef.current = hasMore;
  }, [hasMore]);

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

  const loadDatasets = useCallback(
    async ({ append = false } = {}) => {
      if (isLoadingRef.current || (append && !hasMoreRef.current)) return;
      setIsLoading(true);
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
        setIsLoading(false);
      }
    },
    [setDatasetsState, setHasMoreState],
  );

  const refreshDatasets = useCallback(async () => {
    if (isLoadingRef.current) return;
    setIsLoading(true);
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
      setIsLoading(false);
    }
  }, [setDatasetsState, setHasMoreState]);

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
      setHasMore(true);
      void loadDatasets();
    }, 300);
    return () => {
      if (searchDebounce.current) {
        window.clearTimeout(searchDebounce.current);
      }
    };
  }, [loadDatasets, searchQuery, selectedTags]);

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
  }, [filteredDatasets.length, hasMore, loadDatasets, rowVirtualizer]);

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
    <div className="flex h-full min-h-0 flex-col">
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

      <div className="flex min-h-0 flex-1 flex-col border-t">
        <div className="bg-muted/40 text-muted-foreground grid grid-cols-[60px_70px_minmax(160px,1fr)_120px_minmax(140px,1fr)_160px] gap-2 border-b px-3 py-2 text-xs font-semibold">
          <div>Favorite</div>
          <div>ID</div>
          <div>Name</div>
          <div>Status</div>
          <div>Tags</div>
          <div>Created At</div>
        </div>
        <ScrollArea className="min-h-0 flex-1">
          <div
            ref={scrollRef}
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
        </ScrollArea>
      </div>
    </div>
  );
}
