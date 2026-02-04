import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  fetchChartData,
  getDatasetDetail,
  getDatasetWriteStatus,
  getFilterTableData,
  type ColumnInfo,
  type DatasetDetail,
  type FilterTableData,
  type FilterTableRow,
} from "@/lib/backend";
import type {
  ChartOptions,
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/lib/chartTypes";
import { ChartWrapper } from "@/components/chart-wrapper";
import { FilterTable } from "@/components/filter-table";
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
  const [datasetDetail, setDatasetDetail] = useState<DatasetDetail | null>(
    null,
  );
  const [filterTableData, setFilterTableData] =
    useState<FilterTableData | null>(null);
  const excludeColumnsRef = useRef<string[]>([]);
  const [datasetUpdateTick, setDatasetUpdateTick] = useState(0);

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

  const [filterRow, setFilterRow] = useState<FilterTableRow | undefined>();
  const [data, setData] = useState<ChartOptions | undefined>();
  const [scatterError, setScatterError] = useState<string | null>(null);

  const columns = useMemo(() => datasetDetail?.columns ?? [], [datasetDetail]);

  const pickSelection = useCallback(
    (
      options: ColumnInfo[],
      current: string | null,
      defaultIndex = 0,
    ): string | null => {
      if (options.length === 0) return null;
      const found = options.find((option) => option.name === current);
      if (found) return found.name;
      return options[defaultIndex]?.name ?? options[0]?.name ?? null;
    },
    [],
  );

  const seriesOptions = useMemo(
    () => columns.filter((column) => !column.isIndex),
    [columns],
  );
  const effectiveSeriesName = useMemo(
    () => pickSelection(seriesOptions, seriesName),
    [pickSelection, seriesName, seriesOptions],
  );
  const series = useMemo(
    () => columns.find((column) => column.name === effectiveSeriesName),
    [columns, effectiveSeriesName],
  );

  const xColumnOptions = useMemo(() => {
    if (series?.isTrace) return [];
    return columns.filter((column) => column.isIndex);
  }, [columns, series?.isTrace]);
  const yColumnOptions = useMemo(
    () => columns.filter((column) => column.isIndex),
    [columns],
  );
  const effectiveXColumnName = useMemo(
    () => pickSelection(xColumnOptions, xColumnName),
    [pickSelection, xColumnName, xColumnOptions],
  );
  const effectiveYColumnName = useMemo(
    () => pickSelection(yColumnOptions, yColumnName, 1),
    [pickSelection, yColumnName, yColumnOptions],
  );
  const xColumn = useMemo(
    () => columns.find((column) => column.name === effectiveXColumnName),
    [columns, effectiveXColumnName],
  );
  const yColumn = useMemo(
    () => columns.find((column) => column.name === effectiveYColumnName),
    [columns, effectiveYColumnName],
  );

  const scatterComplexOptions = useMemo(
    () => columns.filter((column) => !column.isIndex && column.isComplex),
    [columns],
  );
  const scatterTraceXYOptions = useMemo(
    () =>
      columns.filter(
        (column) => !column.isIndex && !column.isComplex && column.isTrace,
      ),
    [columns],
  );
  const scatterXYOptions = useMemo(
    () =>
      columns.filter(
        (column) => !column.isIndex && !column.isComplex && !column.isTrace,
      ),
    [columns],
  );

  const hasIndexColumn = useMemo(
    () => columns.some((column) => column.isIndex),
    [columns],
  );
  const canUseScatterComplex = scatterComplexOptions.length > 0;
  const canUseScatterTraceXY = scatterTraceXYOptions.length >= 2;
  const canUseScatterXY = scatterXYOptions.length >= 2 && hasIndexColumn;

  const effectiveScatterMode = useMemo(() => {
    if (scatterMode === "complex" && canUseScatterComplex) return "complex";
    if (scatterMode === "trace_xy" && canUseScatterTraceXY) return "trace_xy";
    if (scatterMode === "xy" && canUseScatterXY) return "xy";
    if (canUseScatterComplex) return "complex";
    if (canUseScatterTraceXY) return "trace_xy";
    return "xy";
  }, [
    canUseScatterComplex,
    canUseScatterTraceXY,
    canUseScatterXY,
    scatterMode,
  ]);

  const effectiveScatterSeriesName = useMemo(
    () => pickSelection(scatterComplexOptions, scatterSeriesName),
    [pickSelection, scatterComplexOptions, scatterSeriesName],
  );
  const effectiveScatterTraceXName = useMemo(
    () => pickSelection(scatterTraceXYOptions, scatterTraceXName),
    [pickSelection, scatterTraceXName, scatterTraceXYOptions],
  );
  const effectiveScatterTraceYName = useMemo(
    () => pickSelection(scatterTraceXYOptions, scatterTraceYName, 1),
    [pickSelection, scatterTraceYName, scatterTraceXYOptions],
  );
  const effectiveScatterXName = useMemo(
    () => pickSelection(scatterXYOptions, scatterXName),
    [pickSelection, scatterXName, scatterXYOptions],
  );
  const effectiveScatterYName = useMemo(
    () => pickSelection(scatterXYOptions, scatterYName, 1),
    [pickSelection, scatterYName, scatterXYOptions],
  );

  const scatterSeries = useMemo(
    () => columns.find((column) => column.name === effectiveScatterSeriesName),
    [columns, effectiveScatterSeriesName],
  );
  const scatterTraceXColumn = useMemo(
    () => columns.find((column) => column.name === effectiveScatterTraceXName),
    [columns, effectiveScatterTraceXName],
  );
  const scatterTraceYColumn = useMemo(
    () => columns.find((column) => column.name === effectiveScatterTraceYName),
    [columns, effectiveScatterTraceYName],
  );
  const scatterXColumn = useMemo(
    () => columns.find((column) => column.name === effectiveScatterXName),
    [columns, effectiveScatterXName],
  );
  const scatterYColumn = useMemo(
    () => columns.find((column) => column.name === effectiveScatterYName),
    [columns, effectiveScatterYName],
  );

  const scatterIsTraceBased = useMemo(() => {
    if (effectiveScatterMode === "trace_xy") return true;
    return effectiveScatterMode === "complex" && scatterSeries?.isTrace;
  }, [effectiveScatterMode, scatterSeries?.isTrace]);

  const scatterBinColumnOptions = useMemo(() => {
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
  }, [
    columns,
    scatterSeries?.name,
    scatterXColumn?.name,
    scatterYColumn?.name,
    scatterTraceXColumn?.name,
    scatterTraceYColumn?.name,
  ]);

  const effectiveScatterBinName = useMemo(() => {
    if (effectiveScatterMode !== "xy" || scatterIsTraceBased) return null;
    return pickSelection(scatterBinColumnOptions, scatterBinName);
  }, [
    effectiveScatterMode,
    pickSelection,
    scatterBinColumnOptions,
    scatterBinName,
    scatterIsTraceBased,
  ]);
  const scatterBinColumn = useMemo(
    () => columns.find((column) => column.name === effectiveScatterBinName),
    [columns, effectiveScatterBinName],
  );

  const scatterModeOptions = useMemo(() => {
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
  }, [canUseScatterComplex, canUseScatterTraceXY, canUseScatterXY]);

  const availableChartTypes = useMemo(() => {
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
  }, [columns]);

  const effectiveChartType = useMemo(() => {
    if (availableChartTypes.length === 0) return chartType;
    return availableChartTypes.includes(chartType)
      ? chartType
      : (availableChartTypes[0] ?? chartType);
  }, [availableChartTypes, chartType]);

  const excludeColumns = useMemo(() => {
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
  }, [
    effectiveChartType,
    effectiveScatterMode,
    scatterBinColumn,
    series,
    xColumn,
    yColumn,
  ]);

  useEffect(() => {
    excludeColumnsRef.current = excludeColumns;
  }, [excludeColumns]);

  useEffect(() => {
    let aborted = false;

    const load = async () => {
      const detail = await getDatasetDetail(datasetId);
      if (aborted) return;

      const filter = await getFilterTableData(datasetId, {
        excludeColumns: excludeColumnsRef.current,
      });
      if (aborted) return;

      setDatasetDetail(detail);
      setFilterTableData(filter);
      setFilterRow(filter.rows[0]);

      const poll = async () => {
        while (!aborted) {
          const { isComplete } = await getDatasetWriteStatus(datasetId);
          if (aborted) return;

          if (isComplete) {
            const updatedDetail = await getDatasetDetail(datasetId);
            if (aborted) return;
            const updatedFilter = await getFilterTableData(datasetId, {
              excludeColumns: excludeColumnsRef.current,
            });
            if (aborted) return;
            setDatasetDetail(updatedDetail);
            setFilterTableData(updatedFilter);
            setDatasetUpdateTick((tick) => tick + 1);
            break;
          }

          const updatedFilter = await getFilterTableData(datasetId, {
            excludeColumns: excludeColumnsRef.current,
          });
          if (aborted) return;
          setFilterTableData(updatedFilter);
          setDatasetUpdateTick((tick) => tick + 1);

          await new Promise((resolve) => setTimeout(resolve, 1000));
        }
      };

      void poll();
    };

    void load();

    return () => {
      aborted = true;
    };
  }, [datasetId]);

  useEffect(() => {
    if (!datasetDetail) return;
    let active = true;
    void getFilterTableData(datasetId, { excludeColumns }).then((filter) => {
      if (!active) return;
      setFilterTableData(filter);
      setFilterRow(filter.rows[0]);
    });
    return () => {
      active = false;
    };
  }, [datasetDetail, datasetId, excludeColumns]);

  const getNewData = useCallback(async (): Promise<
    ChartOptions | undefined
  > => {
    if (!datasetDetail || !filterTableData) return undefined;

    const hasFilters = filterTableData.fields.length > 0;
    if (hasFilters && !filterRow) return undefined;

    setScatterError(null);

    if (effectiveChartType === "scatter") {
      if (effectiveScatterMode === "complex" && !scatterSeries)
        return undefined;
      if (
        effectiveScatterMode === "trace_xy" &&
        (!scatterTraceXColumn || !scatterTraceYColumn)
      ) {
        return undefined;
      }
      if (
        effectiveScatterMode === "xy" &&
        (!scatterXColumn || !scatterYColumn)
      ) {
        return undefined;
      }
    } else {
      if (!series) return undefined;
      if (effectiveChartType === "line") {
        if (!series.isTrace && !xColumn) return undefined;
      } else if (effectiveChartType === "heatmap") {
        if (!yColumn) return undefined;
        if (!series.isTrace && !xColumn) return undefined;
      }
    }

    try {
      return await fetchChartData(datasetId, {
        chartType: effectiveChartType,
        series: series?.name,
        xColumn: xColumn?.name,
        yColumn: yColumn?.name,
        scatterMode: effectiveScatterMode,
        scatterSeries: scatterSeries?.name,
        scatterXColumn: scatterXColumn?.name,
        scatterYColumn: scatterYColumn?.name,
        scatterTraceXColumn: scatterTraceXColumn?.name,
        scatterTraceYColumn: scatterTraceYColumn?.name,
        scatterBinColumn: scatterBinColumn?.name,
        complexViews: selectedComplexView,
        complexViewSingle: selectedComplexViewSingle,
        indexFilters: hasFilters ? filterRow?.valueIndices : undefined,
        excludeColumns,
      });
    } catch (error) {
      if (effectiveChartType === "scatter") {
        setScatterError(
          error instanceof Error
            ? error.message
            : "Scatter data error. Please check trace lengths.",
        );
      }
      return undefined;
    }
  }, [
    effectiveChartType,
    datasetDetail,
    datasetId,
    excludeColumns,
    filterRow,
    filterTableData,
    scatterBinColumn,
    effectiveScatterMode,
    scatterSeries,
    scatterTraceXColumn,
    scatterTraceYColumn,
    scatterXColumn,
    scatterYColumn,
    selectedComplexView,
    selectedComplexViewSingle,
    series,
    xColumn,
    yColumn,
  ]);

  useEffect(() => {
    let isActive = true;
    const handle = window.setTimeout(() => {
      void getNewData().then((next) => {
        if (isActive) {
          setData(next);
        }
      });
    }, 50);
    return () => {
      isActive = false;
      window.clearTimeout(handle);
    };
  }, [getNewData, datasetUpdateTick]);

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
              value={effectiveXColumnName ?? ""}
              onValueChange={(value) =>
                setXColumnName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
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
                if (isComplexViewOption(value)) {
                  setSelectedComplexViewSingle(value);
                }
              }}
            >
              {complexSeriesOptions.map((option) => (
                <label key={option} className="flex items-center gap-2">
                  <RadioGroupItem value={option} />
                  <span className="text-sm">{option}</span>
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
                      onCheckedChange={(checked) => {
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
                        !series?.isComplex && "opacity-50",
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
                value={filterRow}
                onChange={setFilterRow}
                filterTableData={filterTableData ?? undefined}
                datasetId={String(datasetId)}
              />
            </div>
          </ResizablePanel>
        </ResizablePanelGroup>
      </div>
    </div>
  );
}
