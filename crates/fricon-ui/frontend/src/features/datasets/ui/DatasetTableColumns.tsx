import type { Column, ColumnDef } from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ArrowUpDown, Star, StarOff } from "lucide-react";
import type { DatasetInfo, DatasetStatus } from "../api/types";
import type { DatasetColumnMeta } from "../model/datasetColumnMeta";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { Checkbox } from "@/shared/ui/checkbox";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/tooltip";

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

interface CreateDatasetSelectionColumnOptions {
  toggleRowSelected: (rowId: string, isSelected: boolean) => void;
  toggleAllRowsSelected: (isSelected: boolean) => void;
}

function renderSortableHeader({
  column,
  label,
}: {
  column: Column<DatasetInfo>;
  label: string;
}) {
  const sorted = column.getIsSorted();

  return (
    <Button
      variant="ghost"
      className="-ml-2 data-[state=open]:bg-accent"
      onClick={() => column.toggleSorting(sorted === "asc")}
    >
      <span>{label}</span>
      {sorted === "desc" ? (
        <ArrowDown data-icon="inline-end" />
      ) : sorted === "asc" ? (
        <ArrowUp data-icon="inline-end" />
      ) : (
        <ArrowUpDown
          data-icon="inline-end"
          className="text-muted-foreground/50"
        />
      )}
    </Button>
  );
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
                  onClick={(event) => {
                    event.stopPropagation();
                    void toggleFavorite(dataset);
                  }}
                />
              }
            >
              {dataset.favorite ? (
                <Star className="fill-primary text-primary" />
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
      size: 80,
      meta: {
        label: "ID",
        hideable: true,
      } as DatasetColumnMeta,
      header: ({ column }) => renderSortableHeader({ column, label: "ID" }),
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
      header: ({ column }) => renderSortableHeader({ column, label: "Name" }),
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
      size: 180,
      meta: {
        label: "Created At",
        hideable: true,
        defaultVisible: false,
      } as DatasetColumnMeta,
      header: ({ column }) =>
        renderSortableHeader({ column, label: "Created At" }),
      cell: ({ getValue }) => (
        <div className="text-muted-foreground">
          {getValue<Date>().toLocaleString()}
        </div>
      ),
    },
  ];
}

export function createDatasetSelectionColumn({
  toggleRowSelected,
  toggleAllRowsSelected,
}: CreateDatasetSelectionColumnOptions): ColumnDef<DatasetInfo> {
  return {
    id: "select",
    size: 40,
    header: ({ table }) => (
      <Checkbox
        checked={table.getIsAllPageRowsSelected()}
        onCheckedChange={(value) => {
          toggleAllRowsSelected(!!value);
        }}
        aria-label="Select all"
        className="translate-y-0.5"
      />
    ),
    cell: ({ row }) => (
      <Checkbox
        checked={row.getIsSelected()}
        onCheckedChange={(value) => {
          toggleRowSelected(row.id, !!value);
        }}
        onClick={(event) => event.stopPropagation()}
        aria-label="Select row"
        className="translate-y-0.5"
      />
    ),
    enableSorting: false,
    enableHiding: false,
  };
}
