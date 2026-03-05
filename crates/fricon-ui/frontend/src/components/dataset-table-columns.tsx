import type { ColumnDef } from "@tanstack/react-table";
import { Star, StarOff } from "lucide-react";
import type { DatasetInfo, DatasetStatus } from "@/lib/backend";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";

export interface DatasetColumnMeta {
  label: string;
  width: string;
  hideable: boolean;
  defaultVisible: boolean;
}

const statusVariantMap: Record<
  DatasetStatus,
  "default" | "secondary" | "destructive"
> = {
  Writing: "secondary",
  Completed: "default",
  Aborted: "destructive",
};

export const datasetStatusOptions: DatasetStatus[] = [
  "Writing",
  "Completed",
  "Aborted",
];

interface CreateDatasetColumnsOptions {
  toggleFavorite: (dataset: DatasetInfo) => Promise<void>;
}

export function createDatasetColumns({
  toggleFavorite,
}: CreateDatasetColumnsOptions): ColumnDef<DatasetInfo>[] {
  return [
    {
      id: "favorite",
      accessorKey: "favorite",
      enableSorting: false,
      meta: {
        label: "Favorite",
        width: "60px",
        hideable: true,
        defaultVisible: true,
      } satisfies DatasetColumnMeta,
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
      meta: {
        label: "ID",
        width: "70px",
        hideable: true,
        defaultVisible: true,
      } satisfies DatasetColumnMeta,
      cell: ({ getValue }) => (
        <div className="px-2 tabular-nums">{getValue<number>()}</div>
      ),
    },
    {
      id: "name",
      accessorKey: "name",
      meta: {
        label: "Name",
        width: "minmax(160px,40%)",
        hideable: false,
        defaultVisible: true,
      } satisfies DatasetColumnMeta,
      cell: ({ getValue }) => {
        const name = getValue<string>();
        return (
          <div className="w-full min-w-0 truncate px-2" title={name}>
            {name}
          </div>
        );
      },
    },
    {
      id: "status",
      accessorKey: "status",
      enableSorting: false,
      meta: {
        label: "Status",
        width: "120px",
        hideable: true,
        defaultVisible: true,
      } satisfies DatasetColumnMeta,
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
        hideable: true,
        defaultVisible: false,
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
              <span className="text-xs text-muted-foreground">No tags</span>
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
        hideable: true,
        defaultVisible: false,
      } satisfies DatasetColumnMeta,
      cell: ({ getValue }) => (
        <div className="px-2 text-xs text-muted-foreground">
          {getValue<Date>().toLocaleString()}
        </div>
      ),
    },
  ];
}
