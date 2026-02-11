import { useEffect, useState } from "react";
import { type ChartDataOptions, type ColumnInfo } from "@/lib/backend";
import type {
  ChartOptions,
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/lib/chartTypes";
import { ChartWrapper } from "@/components/chart-wrapper";
import { FilterTable } from "@/components/filter-table";
import { useCascadeSelection } from "@/hooks/useCascadeSelection";
import { useChartDataQuery } from "@/hooks/useChartDataQuery";
import { useDatasetDetailQuery } from "@/hooks/useDatasetDetailQuery";
import { useDatasetWriteStatusQuery } from "@/hooks/useDatasetWriteStatusQuery";
import { useFilterTableDataQuery } from "@/hooks/useFilterTableDataQuery";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "@/components/ui/resizable";
import { cn } from "@/lib/utils";

interface ChartViewerProps {
  datasetId: number;
}

const complexSeriesOptions: ComplexViewOption[] = [
  "real",
  "imag",
  "mag",
  "arg",
];

function isComplexViewOption(value: string): value is ComplexViewOption {
  return (complexSeriesOptions as readonly string[]).includes(value);
}

export function ChartViewer({ datasetId }: ChartViewerProps) {
  const [chartType, setChartType] = useState<ChartType>("line");
  const [selectedComplexView, setSelectedComplexView] = useState<
    ComplexViewOption[]
  >(["real", "imag"]);
  const [selectedComplexViewSingle, setSelectedComplexViewSingle] =
    useState<ComplexViewOption>("mag");

  const [seriesName, setSeriesName] = useState<string | null>(null);
  const [xColumnName, setXColumnName] = useState<string | null>(null);
  const [yColumnName, setYColumnName] = useState<string | null>(null);

  const [scatterMode, setScatterMode] = useState<ScatterMode>("complex");
  const [scatterSeriesName, setScatterSeriesName] = useState<string | null>(
    null,
  );
  const [scatterTraceXName, setScatterTraceXName] = useState<string | null>(
    null,
  );
  const [scatterTraceYName, setScatterTraceYName] = useState<string | null>(
    null,
  );
  const [scatterXName, setScatterXName] = useState<string | null>(null);
  const [scatterYName, setScatterYName] = useState<string | null>(null);
  const [scatterBinName, setScatterBinName] = useState<string | null>(null);

  const datasetDetailQuery = useDatasetDetailQuery(datasetId);
  const datasetDetail = datasetDetailQuery.data ?? null;

  const columns = datasetDetail?.columns ?? [];

  const pickSelection = (
    options: ColumnInfo[],
    current: string | null,
    defaultIndex = 0,
  ): string | null => {
    if (options.length === 0) return null;
    const found = options.find((option) => option.name === current);
    if (found) return found.name;
    return options[defaultIndex]?.name ?? options[0]?.name ?? null;
  };

  const seriesOptions = columns.filter((column) => !column.isIndex);
  const effectiveSeriesName = pickSelection(seriesOptions, seriesName);
  const series = columns.find((column) => column.name === effectiveSeriesName);
  const isComplexSeries = Boolean(series?.isComplex);
  const complexControlsDisabled = !isComplexSeries;
  const isTraceSeries = Boolean(series?.isTrace);

  const xColumnOptions = series?.isTrace
    ? []
    : columns.filter((column) => column.isIndex);
  const yColumnOptions = columns.filter((column) => column.isIndex);
  const effectiveXColumnName = pickSelection(xColumnOptions, xColumnName);
  const effectiveYColumnName = pickSelection(yColumnOptions, yColumnName, 1);
  const xColumn = columns.find(
    (column) => column.name === effectiveXColumnName,
  );
  const yColumn = columns.find(
    (column) => column.name === effectiveYColumnName,
  );

  const scatterComplexOptions = columns.filter(
    (column) => !column.isIndex && column.isComplex,
  );
  const scatterTraceXYOptions = columns.filter(
    (column) => !column.isIndex && !column.isComplex && column.isTrace,
  );
  const scatterXYOptions = columns.filter(
    (column) => !column.isIndex && !column.isComplex && !column.isTrace,
  );

  const hasIndexColumn = columns.some((column) => column.isIndex);
  const canUseScatterComplex = scatterComplexOptions.length > 0;
  const canUseScatterTraceXY = scatterTraceXYOptions.length >= 2;
  const canUseScatterXY = scatterXYOptions.length >= 2 && hasIndexColumn;

  const effectiveScatterMode: ScatterMode = (() => {
    if (scatterMode === "complex" && canUseScatterComplex) return "complex";
    if (scatterMode === "trace_xy" && canUseScatterTraceXY) return "trace_xy";
    if (scatterMode === "xy" && canUseScatterXY) return "xy";
    if (canUseScatterComplex) return "complex";
    if (canUseScatterTraceXY) return "trace_xy";
    return "xy";
  })();

  const effectiveScatterSeriesName = pickSelection(
    scatterComplexOptions,
    scatterSeriesName,
  );
  const effectiveScatterTraceXName = pickSelection(
    scatterTraceXYOptions,
    scatterTraceXName,
  );
  const effectiveScatterTraceYName = pickSelection(
    scatterTraceXYOptions,
    scatterTraceYName,
    1,
  );
  const effectiveScatterXName = pickSelection(scatterXYOptions, scatterXName);
  const effectiveScatterYName = pickSelection(
    scatterXYOptions,
    scatterYName,
    1,
  );

  const scatterSeries = columns.find(
    (column) => column.name === effectiveScatterSeriesName,
  );
  const scatterTraceXColumn = columns.find(
    (column) => column.name === effectiveScatterTraceXName,
  );
  const scatterTraceYColumn = columns.find(
    (column) => column.name === effectiveScatterTraceYName,
  );
  const scatterXColumn = columns.find(
    (column) => column.name === effectiveScatterXName,
  );
  const scatterYColumn = columns.find(
    (column) => column.name === effectiveScatterYName,
  );

  const scatterIsTraceBased = (() => {
    if (effectiveScatterMode === "trace_xy") return true;
    return effectiveScatterMode === "complex" && scatterSeries?.isTrace;
  })();

  const scatterBinColumnOptions = (() => {
    const excludedNames = new Set(
      [
        scatterSeries?.name,
        scatterXColumn?.name,
        scatterYColumn?.name,
        scatterTraceXColumn?.name,
        scatterTraceYColumn?.name,
      ].filter((name): name is string => Boolean(name)),
    );
    return columns.filter(
      (column) => column.isIndex && !excludedNames.has(column.name),
    );
  })();

  const effectiveScatterBinName = (() => {
    if (effectiveScatterMode !== "xy" || scatterIsTraceBased) return null;
    return pickSelection(scatterBinColumnOptions, scatterBinName);
  })();
  const scatterBinColumn = columns.find(
    (column) => column.name === effectiveScatterBinName,
  );

  const scatterModeOptions = (() => {
    const options: { label: string; value: ScatterMode }[] = [];
    if (canUseScatterComplex) {
      options.push({ label: "Complex (real/imag)", value: "complex" });
    }
    if (canUseScatterTraceXY) {
      options.push({ label: "Trace X/Y", value: "trace_xy" });
    }
    if (canUseScatterXY) {
      options.push({ label: "X/Y columns", value: "xy" });
    }
    return options;
  })();

  const availableChartTypes = (() => {
    const cols = columns;
    if (cols.length === 0) return [];
    const hasSeries = cols.some((column) => !column.isIndex);
    const hasIndex = cols.some((column) => column.isIndex);
    const hasComplex = cols.some(
      (column) => !column.isIndex && column.isComplex,
    );
    const realColumns = cols.filter(
      (column) => !column.isIndex && !column.isComplex && !column.isTrace,
    );
    const realTraceColumns = cols.filter(
      (column) => !column.isIndex && !column.isComplex && column.isTrace,
    );
    const canScatter =
      hasComplex || realColumns.length >= 2 || realTraceColumns.length >= 2;
    const types: ChartType[] = [];
    if (hasSeries) types.push("line");
    if (hasSeries && hasIndex) types.push("heatmap");
    if (canScatter) types.push("scatter");
    return types;
  })();

  const effectiveChartType = (() => {
    if (availableChartTypes.length === 0) return chartType;
    return availableChartTypes.includes(chartType)
      ? chartType
      : (availableChartTypes[0] ?? chartType);
  })();

  const excludeColumns = (() => {
    const excludes: string[] = [];
    if (effectiveChartType === "line") {
      if (xColumn) excludes.push(xColumn.name);
    } else if (effectiveChartType === "heatmap") {
      if (series?.isTrace) {
        if (yColumn) excludes.push(yColumn.name);
      } else {
        if (xColumn) excludes.push(xColumn.name);
        if (yColumn) excludes.push(yColumn.name);
      }
    } else if (effectiveChartType === "scatter") {
      if (effectiveScatterMode === "xy" && scatterBinColumn?.isIndex) {
        excludes.push(scatterBinColumn.name);
      }
    }
    return excludes;
  })();

  const filterTableQuery = useFilterTableDataQuery(
    datasetId,
    excludeColumns,
    Boolean(datasetDetail),
  );
  const filterTableData = filterTableQuery.data ?? null;
  const { refetch: refetchFilterTable } = filterTableQuery;

  useEffect(() => {
    if (!datasetDetail) return;
    void refetchFilterTable();
  }, [datasetDetail, refetchFilterTable]);

  const cascade = useCascadeSelection(filterTableData);
  const filterRow = cascade.resolvedRow;
  const hasFilters = (filterTableData?.fields.length ?? 0) > 0;
  const indexFilters = hasFilters ? filterRow?.valueIndices : undefined;

  useDatasetWriteStatusQuery(datasetId, datasetDetail?.status === "Writing");

  const chartRequest: ChartDataOptions | null = (() => {
    if (!datasetDetail || !filterTableData) return null;
    if (hasFilters && !filterRow) return null;

    if (effectiveChartType === "scatter") {
      if (effectiveScatterMode === "complex" && !scatterSeries) return null;
      if (
        effectiveScatterMode === "trace_xy" &&
        (!scatterTraceXColumn || !scatterTraceYColumn)
      ) {
        return null;
      }
      if (
        effectiveScatterMode === "xy" &&
        (!scatterXColumn || !scatterYColumn)
      ) {
        return null;
      }
    } else {
      if (!series) return null;
      if (effectiveChartType === "line") {
        if (!series.isTrace && !xColumn) return null;
      } else if (effectiveChartType === "heatmap") {
        if (!yColumn) return null;
        if (!series.isTrace && !xColumn) return null;
      }
    }

    if (effectiveChartType === "line" && series) {
      return {
        chartType: "line",
        series: series.name,
        xColumn: xColumn?.name,
        complexViews: selectedComplexView,
        indexFilters,
        excludeColumns,
      };
    }

    if (effectiveChartType === "heatmap" && series && yColumn) {
      return {
        chartType: "heatmap",
        series: series.name,
        xColumn: xColumn?.name,
        yColumn: yColumn.name,
        complexViewSingle: selectedComplexViewSingle,
        indexFilters,
        excludeColumns,
      };
    }

    if (effectiveScatterMode === "complex" && scatterSeries) {
      return {
        chartType: "scatter",
        scatter: {
          mode: "complex",
          series: scatterSeries.name,
        },
        indexFilters,
        excludeColumns,
      };
    }

    if (
      effectiveScatterMode === "trace_xy" &&
      scatterTraceXColumn &&
      scatterTraceYColumn
    ) {
      return {
        chartType: "scatter",
        scatter: {
          mode: "trace_xy",
          traceXColumn: scatterTraceXColumn.name,
          traceYColumn: scatterTraceYColumn.name,
        },
        indexFilters,
        excludeColumns,
      };
    }

    if (scatterXColumn && scatterYColumn) {
      return {
        chartType: "scatter",
        scatter: {
          mode: "xy",
          xColumn: scatterXColumn.name,
          yColumn: scatterYColumn.name,
          binColumn: scatterBinColumn?.name,
        },
        indexFilters,
        excludeColumns,
      };
    }

    return null;
  })();

  const chartQuery = useChartDataQuery(datasetId, chartRequest);
  const data: ChartOptions | undefined = chartQuery.data;
  const scatterError =
    effectiveChartType === "scatter" && chartQuery.error
      ? chartQuery.error instanceof Error
        ? chartQuery.error.message
        : "Scatter data error. Please check trace lengths."
      : null;

  return (
    <div className="flex size-full min-h-0 flex-col overflow-hidden">
      <div className="flex flex-wrap gap-2 p-2">
        <div className="min-w-[160px]">
          <Label className="mb-1 block text-xs">Chart Type</Label>
          <Select
            value={effectiveChartType}
            onValueChange={(value) => {
              if (value) setChartType(value);
            }}
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select chart type" />
            </SelectTrigger>
            <SelectContent>
              {availableChartTypes.map((type) => (
                <SelectItem key={type} value={type}>
                  {type}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {effectiveChartType !== "scatter" ? (
          <div className="min-w-[180px]">
            <Label className="mb-1 block text-xs">Series</Label>
            <Select
              value={effectiveSeriesName ?? ""}
              onValueChange={(value) =>
                setSeriesName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select series" />
              </SelectTrigger>
              <SelectContent>
                {seriesOptions.map((option) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType !== "scatter" &&
        (effectiveChartType === "line" ||
          (effectiveChartType === "heatmap" && !series?.isTrace)) ? (
          <div className="min-w-[160px]">
            <Label className="mb-1 block text-xs">X</Label>
            <Select
              disabled={effectiveChartType === "line" && isTraceSeries}
              value={effectiveXColumnName ?? ""}
              onValueChange={(value) =>
                setXColumnName(value === "" ? null : value)
              }
            >
              <SelectTrigger
                className="w-full"
                disabled={effectiveChartType === "line" && isTraceSeries}
              >
                <SelectValue placeholder="Select X" />
              </SelectTrigger>
              <SelectContent>
                {xColumnOptions.map((option) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType === "heatmap" ? (
          <div className="min-w-[160px]">
            <Label className="mb-1 block text-xs">Y</Label>
            <Select
              value={effectiveYColumnName ?? ""}
              onValueChange={(value) =>
                setYColumnName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select Y" />
              </SelectTrigger>
              <SelectContent>
                {yColumnOptions.map((option) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType === "scatter" ? (
          <div className="min-w-[200px]">
            <Label className="mb-1 block text-xs">Point Cloud Source</Label>
            <Select
              value={effectiveScatterMode}
              onValueChange={(value) => {
                if (
                  value === "complex" ||
                  value === "trace_xy" ||
                  value === "xy"
                ) {
                  setScatterMode(value);
                }
              }}
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select mode" />
              </SelectTrigger>
              <SelectContent>
                {scatterModeOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType === "scatter" &&
        effectiveScatterMode === "complex" ? (
          <div className="min-w-[200px]">
            <Label className="mb-1 block text-xs">Complex Series</Label>
            <Select
              value={effectiveScatterSeriesName ?? ""}
              onValueChange={(value) =>
                setScatterSeriesName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select series" />
              </SelectTrigger>
              <SelectContent>
                {scatterComplexOptions.map((option) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType === "scatter" && effectiveScatterMode === "xy" ? (
          <>
            <div className="min-w-[160px]">
              <Label className="mb-1 block text-xs">X Column</Label>
              <Select
                value={effectiveScatterXName ?? ""}
                onValueChange={(value) =>
                  setScatterXName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select X" />
                </SelectTrigger>
                <SelectContent>
                  {scatterXYOptions.map((option) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="min-w-[160px]">
              <Label className="mb-1 block text-xs">Y Column</Label>
              <Select
                value={effectiveScatterYName ?? ""}
                onValueChange={(value) =>
                  setScatterYName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select Y" />
                </SelectTrigger>
                <SelectContent>
                  {scatterXYOptions.map((option) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </>
        ) : null}

        {effectiveChartType === "scatter" &&
        effectiveScatterMode === "trace_xy" ? (
          <>
            <div className="min-w-[160px]">
              <Label className="mb-1 block text-xs">Trace X</Label>
              <Select
                value={effectiveScatterTraceXName ?? ""}
                onValueChange={(value) =>
                  setScatterTraceXName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select trace X" />
                </SelectTrigger>
                <SelectContent>
                  {scatterTraceXYOptions.map((option) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="min-w-[160px]">
              <Label className="mb-1 block text-xs">Trace Y</Label>
              <Select
                value={effectiveScatterTraceYName ?? ""}
                onValueChange={(value) =>
                  setScatterTraceYName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select trace Y" />
                </SelectTrigger>
                <SelectContent>
                  {scatterTraceXYOptions.map((option) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </>
        ) : null}

        {effectiveChartType === "scatter" && effectiveScatterMode === "xy" ? (
          <div className="min-w-[200px]">
            <Label className="mb-1 block text-xs">
              Index Column (excluded)
            </Label>
            <Select
              value={effectiveScatterBinName ?? ""}
              onValueChange={(value) =>
                setScatterBinName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select index column" />
              </SelectTrigger>
              <SelectContent>
                {scatterBinColumnOptions.map((option) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}
      </div>

      {effectiveChartType === "scatter" && scatterError ? (
        <div className="text-destructive px-2 text-sm">{scatterError}</div>
      ) : null}

      {effectiveChartType !== "scatter" ? (
        <div className="flex flex-wrap items-center gap-3 px-2 pb-2 text-xs">
          <span className="font-medium">Complex:</span>
          {effectiveChartType === "heatmap" ? (
            <RadioGroup
              className="flex flex-wrap gap-2"
              value={selectedComplexViewSingle}
              onValueChange={(value: string) => {
                if (complexControlsDisabled) return;
                if (isComplexViewOption(value)) {
                  setSelectedComplexViewSingle(value);
                }
              }}
            >
              {complexSeriesOptions.map((option) => (
                <label key={option} className="flex items-center gap-2">
                  <RadioGroupItem
                    value={option}
                    disabled={complexControlsDisabled}
                  />
                  <span
                    className={cn(
                      "text-sm",
                      complexControlsDisabled && "opacity-50",
                    )}
                  >
                    {option}
                  </span>
                </label>
              ))}
            </RadioGroup>
          ) : (
            <div className="flex flex-wrap gap-2">
              {complexSeriesOptions.map((option) => {
                const isChecked = selectedComplexView.includes(option);
                return (
                  <label key={option} className="flex items-center gap-2">
                    <Checkbox
                      checked={isChecked}
                      disabled={complexControlsDisabled}
                      onCheckedChange={(checked) => {
                        if (complexControlsDisabled) return;
                        const next = checked
                          ? [...selectedComplexView, option]
                          : selectedComplexView.filter(
                              (item) => item !== option,
                            );
                        setSelectedComplexView(next);
                      }}
                    />
                    <span
                      className={cn(
                        "text-sm",
                        complexControlsDisabled && "opacity-50",
                      )}
                    >
                      {option}
                    </span>
                  </label>
                );
              })}
            </div>
          )}
        </div>
      ) : null}

      <div className="min-h-0 flex-1 overflow-hidden p-2">
        <ResizablePanelGroup orientation="vertical" className="h-full min-h-0">
          <ResizablePanel defaultSize={60} minSize={35} className="min-h-0">
            <div className="h-full min-h-0">
              <ChartWrapper data={data} />
            </div>
          </ResizablePanel>
          <ResizableHandle withHandle />
          <ResizablePanel defaultSize={40} minSize={25} className="min-h-0">
            <div className="h-full min-h-0">
              <FilterTable
                data={filterTableData ?? undefined}
                mode={cascade.state.mode}
                onModeChange={cascade.setMode}
                selectedRowIndex={filterRow?.index ?? null}
                onSelectRow={cascade.selectRow}
                selectedValueIndices={cascade.selectedValueIndices}
                onSelectFieldValue={(fieldIndex, valueIndex) => {
                  if (!filterTableData) return;
                  cascade.selectFieldValue(
                    fieldIndex,
                    valueIndex,
                    filterRow?.index ?? null,
                  );
                }}
              />
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  );
}
