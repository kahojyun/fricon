import type { ColumnInfo } from "@/lib/backend";
import type {
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/lib/chartTypes";
import {
  complexSeriesOptions,
  deriveChartViewerState,
  isComplexViewOption,
} from "@/features/chart-viewer/lib/chartViewerLogic";
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
import { cn } from "@/lib/utils";

interface ChartViewerControlsProps {
  derived: ReturnType<typeof deriveChartViewerState>;
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
  setChartType: (next: ChartType) => void;
  setSeriesName: (next: string | null) => void;
  setXColumnName: (next: string | null) => void;
  setYColumnName: (next: string | null) => void;
  setScatterMode: (next: ScatterMode) => void;
  setScatterSeriesName: (next: string | null) => void;
  setScatterTraceXName: (next: string | null) => void;
  setScatterTraceYName: (next: string | null) => void;
  setScatterXName: (next: string | null) => void;
  setScatterYName: (next: string | null) => void;
  setScatterBinName: (next: string | null) => void;
  setSelectedComplexView: (next: ComplexViewOption[]) => void;
  setSelectedComplexViewSingle: (next: ComplexViewOption) => void;
}

export function ChartViewerControls({
  derived,
  selectedComplexView,
  selectedComplexViewSingle,
  setChartType,
  setSeriesName,
  setXColumnName,
  setYColumnName,
  setScatterMode,
  setScatterSeriesName,
  setScatterTraceXName,
  setScatterTraceYName,
  setScatterXName,
  setScatterYName,
  setScatterBinName,
  setSelectedComplexView,
  setSelectedComplexViewSingle,
}: ChartViewerControlsProps) {
  const {
    availableChartTypes,
    complexControlsDisabled,
    effectiveChartType,
    effectiveScatterBinName,
    effectiveScatterMode,
    effectiveScatterSeriesName,
    effectiveScatterTraceXName,
    effectiveScatterTraceYName,
    effectiveScatterXName,
    effectiveScatterYName,
    effectiveSeriesName,
    effectiveXColumnName,
    effectiveYColumnName,
    isTraceSeries,
    scatterBinColumnOptions,
    scatterComplexOptions,
    scatterIsTraceBased,
    scatterModeOptions,
    scatterTraceXYOptions,
    scatterXYOptions,
    series,
    seriesOptions,
    xColumnOptions,
    yColumnOptions,
  } = derived;

  return (
    <>
      <div className="flex flex-wrap gap-1.5 p-1.5">
        <div className="min-w-40">
          <Label className="mb-1 block">Chart Type</Label>
          <Select
            value={effectiveChartType}
            onValueChange={(value) => {
              if (value) {
                setChartType(value);
              }
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
          <div className="min-w-45">
            <Label className="mb-1 block">Series</Label>
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
                {seriesOptions.map((option: ColumnInfo) => (
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
          <div className="min-w-40">
            <Label className="mb-1 block">X</Label>
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
                {xColumnOptions.map((option: ColumnInfo) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType === "heatmap" ? (
          <div className="min-w-40">
            <Label className="mb-1 block">Y</Label>
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
                {yColumnOptions.map((option: ColumnInfo) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {effectiveChartType === "scatter" ? (
          <div className="min-w-50">
            <Label className="mb-1 block">Point Cloud Source</Label>
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
          <div className="min-w-50">
            <Label className="mb-1 block">Complex Series</Label>
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
                {scatterComplexOptions.map((option: ColumnInfo) => (
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
            <div className="min-w-40">
              <Label className="mb-1 block">X Column</Label>
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
                  {scatterXYOptions.map((option: ColumnInfo) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="min-w-40">
              <Label className="mb-1 block">Y Column</Label>
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
                  {scatterXYOptions.map((option: ColumnInfo) => (
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
            <div className="min-w-40">
              <Label className="mb-1 block">Trace X</Label>
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
                  {scatterTraceXYOptions.map((option: ColumnInfo) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="min-w-40">
              <Label className="mb-1 block">Trace Y</Label>
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
                  {scatterTraceXYOptions.map((option: ColumnInfo) => (
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
        (effectiveScatterMode === "xy" || effectiveScatterMode === "complex") &&
        !scatterIsTraceBased ? (
          <div className="min-w-50">
            <Label className="mb-1 block">Index Column (excluded)</Label>
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
                {scatterBinColumnOptions.map((option: ColumnInfo) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}
      </div>

      {effectiveChartType !== "scatter" ? (
        <div className="flex flex-wrap items-center gap-2 px-1.5 pb-1.5 text-xs">
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
                      "text-xs",
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
                        "text-xs",
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
    </>
  );
}
