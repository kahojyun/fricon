import type { DatasetStatus } from "../api/types";
import { datasetStatusOptions } from "./DatasetTableColumns";
import { ToggleGroup, ToggleGroupItem } from "@/shared/ui/toggle-group";

interface DatasetStatusFilterProps {
  selectedStatuses: DatasetStatus[];
  onToggleStatus: (status: DatasetStatus) => void;
}

export function DatasetStatusFilter({
  selectedStatuses,
  onToggleStatus,
}: DatasetStatusFilterProps) {
  return (
    <div className="flex shrink-0 items-center gap-1.5 rounded-md border border-dashed px-2 py-1">
      <span className="text-xs text-muted-foreground">Status</span>
      <ToggleGroup
        multiple
        value={selectedStatuses}
        onValueChange={(nextValues) => {
          const next = new Set(nextValues);
          const current = new Set(selectedStatuses);

          datasetStatusOptions.forEach((status) => {
            if (next.has(status) !== current.has(status)) {
              onToggleStatus(status);
            }
          });
        }}
        variant="outline"
        size="sm"
        spacing={1}
      >
        {datasetStatusOptions.map((status) => (
          <ToggleGroupItem key={status} value={status}>
            {status}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
