import type { Table } from "@tanstack/react-table";
import type { DatasetInfo } from "../api/types";
import type { DatasetColumnMeta } from "../model/datasetColumnMeta";
import { Settings2 } from "lucide-react";
import { Button } from "@/shared/ui/button";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/shared/ui/dropdown-menu";

interface DatasetTableViewOptionsProps {
  table: Table<DatasetInfo>;
  resetColumnVisibilityToDefault: () => void;
  showAllColumns: () => void;
  onColumnVisibilityChange: (columnId: string, visible: boolean) => void;
}

export function DatasetTableViewOptions({
  table,
  resetColumnVisibilityToDefault,
  showAllColumns,
  onColumnVisibilityChange,
}: DatasetTableViewOptionsProps) {
  const allColumns = table.getAllLeafColumns();

  return (
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
            const meta = column.columnDef.meta as DatasetColumnMeta | undefined;
            const label = meta?.label ?? column.id;
            const hideable = meta?.hideable ?? true;

            return (
              <DropdownMenuCheckboxItem
                key={column.id}
                checked={column.getIsVisible()}
                disabled={!hideable}
                onCheckedChange={(checked) => {
                  if (!hideable) {
                    return;
                  }
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
  );
}
