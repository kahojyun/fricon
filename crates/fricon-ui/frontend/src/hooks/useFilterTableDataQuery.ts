import { useQuery } from "@tanstack/react-query";
import { getFilterTableData } from "@/lib/backend";

function buildExcludeColumnsKey(excludeColumns: string[]) {
  return excludeColumns.slice().sort().join("|");
}

export function useFilterTableDataQuery(
  datasetId: number,
  excludeColumns: string[],
  enabled = true,
) {
  const excludeColumnsKey = buildExcludeColumnsKey(excludeColumns);
  return useQuery({
    queryKey: ["filterTableData", datasetId, excludeColumnsKey],
    queryFn: () => getFilterTableData(datasetId, { excludeColumns }),
    enabled,
  });
}
