import { useQuery } from "@tanstack/react-query";
import { fetchChartData, type ChartDataOptions } from "@/lib/backend";

function buildChartDataKey(
  datasetId: number,
  options: ChartDataOptions | null,
) {
  if (!options) return ["chartData", datasetId, "disabled"];
  const complexViewsKey = options.complexViews
    ? [...options.complexViews].sort().join(",")
    : "";
  const indexFiltersKey = options.indexFilters
    ? options.indexFilters.join(",")
    : "";
  const excludeColumnsKey = options.excludeColumns
    ? [...options.excludeColumns].sort().join("|")
    : "";

  return [
    "chartData",
    datasetId,
    options.chartType,
    options.series ?? "",
    options.xColumn ?? "",
    options.yColumn ?? "",
    options.scatterMode ?? "",
    options.scatterSeries ?? "",
    options.scatterXColumn ?? "",
    options.scatterYColumn ?? "",
    options.scatterTraceXColumn ?? "",
    options.scatterTraceYColumn ?? "",
    options.scatterBinColumn ?? "",
    complexViewsKey,
    options.complexViewSingle ?? "",
    indexFiltersKey,
    excludeColumnsKey,
  ];
}

export function useChartDataQuery(
  datasetId: number,
  options: ChartDataOptions | null,
) {
  return useQuery({
    queryKey: buildChartDataKey(datasetId, options),
    queryFn: () => fetchChartData(datasetId, options!),
    enabled: Boolean(options),
  });
}
