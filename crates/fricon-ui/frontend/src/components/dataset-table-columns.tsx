import type { ColumnDef } from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ArrowUpDown, Star, StarOff } from "lucide-react";
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
  hideable: boolean;
  defaultVisible?: boolean;
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
      size: 60,
      meta: {
        label: "Favorite",
        hideable: true,
      } as DatasetColumnMeta,
      header: () => <span className="sr-only">Favorite</span>,
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
                  size="icon"
                  className="h-8 w-8"
                  onClick={(event) => {
                    event.stopPropagation();
                    void toggleFavorite(dataset);
                  }}
                />
              }
            >
              {dataset.favorite ? (
                <Star className="h-4 w-4 fill-yellow-500 text-yellow-500" />
              ) : (
                <StarOff className="h-4 w-4 text-muted-foreground" />
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
      size: 80,
      meta: {
        label: "ID",
        hideable: true,
      } as DatasetColumnMeta,
      header: ({ column }) => {
        return (
          <Button
            variant="ghost"
            size="sm"
            className="-ml-3 h-8 data-[state=open]:bg-accent"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            <span>ID</span>
            {column.getIsSorted() === "desc" ? (
              <ArrowDown className="ml-2 h-4 w-4" />
            ) : column.getIsSorted() === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowUpDown className="ml-2 h-4 w-4 text-muted-foreground/50" />
            )}
          </Button>
        );
      },
      cell: ({ getValue }) => (
        <div className="font-medium tabular-nums">{getValue<number>()}</div>
      ),
    },
    {
      id: "name",
      accessorKey: "name",
      size: 300,
      meta: {
        label: "Name",
        hideable: false,
      } as DatasetColumnMeta,
      header: ({ column }) => {
        return (
          <Button
            variant="ghost"
            size="sm"
            className="-ml-3 h-8 data-[state=open]:bg-accent"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            <span>Name</span>
            {column.getIsSorted() === "desc" ? (
              <ArrowDown className="ml-2 h-4 w-4" />
            ) : column.getIsSorted() === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowUpDown className="ml-2 h-4 w-4 text-muted-foreground/50" />
            )}
          </Button>
        );
      },
      cell: ({ getValue }) => {
        const name = getValue<string>();
        return (
          <div
            className="max-w-75 truncate font-medium lg:max-w-125"
            title={name}
          >
            {name}
          </div>
        );
      },
    },
    {
      id: "status",
      accessorKey: "status",
      enableSorting: false,
      size: 120,
      meta: {
        label: "Status",
        hideable: true,
      } as DatasetColumnMeta,
      header: "Status",
      cell: ({ getValue }) => (
        <Badge variant={statusVariantMap[getValue<DatasetStatus>()]}>
          {getValue<string>()}
        </Badge>
      ),
    },
    {
      id: "tags",
      accessorKey: "tags",
      enableSorting: false,
      size: 300,
      meta: {
        label: "Tags",
        hideable: true,
        defaultVisible: false,
      } as DatasetColumnMeta,
      header: "Tags",
      cell: ({ getValue }) => {
        const tags = getValue<string[]>();
        return (
          <div className="flex flex-wrap gap-1">
            {tags.length > 0 ? (
              tags.map((tag) => (
                <Badge
                  key={tag}
                  variant="secondary"
                  className="rounded-sm font-normal"
                >
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
      size: 180,
      meta: {
        label: "Created At",
        hideable: true,
        defaultVisible: false,
      } as DatasetColumnMeta,
      header: ({ column }) => {
        return (
          <Button
            variant="ghost"
            size="sm"
            className="-ml-3 h-8 data-[state=open]:bg-accent"
            onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
          >
            <span>Created At</span>
            {column.getIsSorted() === "desc" ? (
              <ArrowDown className="ml-2 h-4 w-4" />
            ) : column.getIsSorted() === "asc" ? (
              <ArrowUp className="ml-2 h-4 w-4" />
            ) : (
              <ArrowUpDown className="ml-2 h-4 w-4 text-muted-foreground/50" />
            )}
          </Button>
        );
      },
      cell: ({ getValue }) => (
        <div className="text-sm text-muted-foreground">
          {getValue<Date>().toLocaleString()}
        </div>
      ),
    },
  ];
}
