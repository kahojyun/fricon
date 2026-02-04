import type { FilterTableData, FilterTableRow } from "@/lib/backend";
import { cn } from "@/lib/utils";

interface FilterTableProps {
  value?: FilterTableRow;
  onChange: (value: FilterTableRow | undefined) => void;
  filterTableData?: FilterTableData;
  datasetId: string;
}

export function FilterTable({
  value,
  onChange,
  filterTableData,
}: FilterTableProps) {
  if (!filterTableData || filterTableData.rows.length === 0) {
    return (
      <div className="text-muted-foreground flex h-full items-center justify-center text-sm">
        No data available
      </div>
    );
  }

  return (
    <div className="h-full overflow-auto">
      <table className="w-full text-xs">
        <thead className="bg-muted/40 text-muted-foreground sticky top-0">
          <tr>
            {filterTableData.fields.map((field) => (
              <th key={field} className="px-2 py-2 text-left font-semibold">
                {field}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {filterTableData.rows.map((row) => {
            const isSelected = value?.index === row.index;
            return (
              <tr
                key={row.index}
                className={cn(
                  "cursor-pointer border-b",
                  isSelected ? "bg-primary/10" : "hover:bg-muted/40",
                )}
                onClick={() => onChange(row)}
              >
                {filterTableData.fields.map((field, idx) => (
                  <td key={field} className="px-2 py-2">
                    {row.displayValues[idx]}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
