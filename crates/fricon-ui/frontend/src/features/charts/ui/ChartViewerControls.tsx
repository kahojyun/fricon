import { useState } from "react";
import type { ColumnInfo, DatasetStatus } from "../api/types";
import {
  complexSeriesOptions,
  deriveChartViewerState,
  isComplexViewOption,
} from "../model/chartViewerLogic";
import type {
  ChartViewerControlActions,
  ChartViewerControlState,
} from "../hooks/useChartViewerSelection";
import { Checkbox } from "@/shared/ui/checkbox";
import { Label } from "@/shared/ui/label";
import { RadioGroup, RadioGroupItem } from "@/shared/ui/radio-group";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/ui/select";
import { Button } from "@/shared/ui/button";
import { Toggle } from "@/shared/ui/toggle";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/shared/ui/tooltip";
import { Activity, ChevronDownIcon, ChevronRightIcon } from "lucide-react";
import { cn } from "@/shared/lib/utils";

interface ChartViewerControlsProps {
  derived: ReturnType<typeof deriveChartViewerState>;
  controlState: ChartViewerControlState;
  actions: ChartViewerControlActions;
  datasetStatus?: DatasetStatus;
}

const viewLabels = {
  xy: "XY",
  heatmap: "Heatmap",
} as const;

const projectionLabels = {
  trend: "Trend",
  xy: "X-Y",
  complex_xy: "Complex Plane",
} as const;

const drawStyleLabels = {
  line: "Line",
  points: "Points",
  line_points: "Line + Points",
} as const;

const complexViewLabels = {
  real: "Real",
  imag: "Imaginary",
  mag: "Magnitude",
  arg: "Phase",
} as const;

const liveWindowOptions = [1, 3, 5, 10, 20] as const;

export function ChartViewerControls({
  derived,
  controlState,
  actions,
  datasetStatus,
}: ChartViewerControlsProps) {
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const {
    selectedComplexView,
    selectedComplexViewSingle,
    isLiveMode,
    liveWindowCount,
  } = controlState;
  const {
    setView,
    setProjection,
    setDrawStyle,
    setTrendSeriesName,
    setHeatmapSeriesName,
    setComplexXYSeriesName,
    setXYXName,
    setXYYName,
    setHeatmapXName,
    setHeatmapYName,
    toggleGroupByIndexColumnName,
    setOrderByIndexColumnName,
    setSelectedComplexView,
    setSelectedComplexViewSingle,
    setLiveWindowCount,
  } = actions;
  const showPrimaryProjection = derived.effectiveView === "xy";
  const showPrimaryLiveWindow = isLiveMode && derived.effectiveView === "xy";
  const showPrimaryOrderBy = derived.xyRoleControlsVisible && !isLiveMode;
  const showAdvancedStyle = derived.effectiveView === "xy";
  const showAdvancedHeatmapAxes =
    derived.effectiveView === "heatmap" && !isLiveMode;
  const showAdvancedGrouping = derived.xyRoleControlsVisible && !isLiveMode;
  const showAdvancedComplexControls =
    derived.effectiveView === "heatmap" ||
    (derived.effectiveView === "xy" && derived.effectiveProjection === "trend");
  const showAdvancedControls =
    showAdvancedStyle ||
    showAdvancedHeatmapAxes ||
    showAdvancedGrouping ||
    showAdvancedComplexControls;

  return (
    <>
      <div className="flex flex-wrap items-end gap-1.5 p-1.5">
        {datasetStatus === "Writing" ? (
          <div className="flex flex-col">
            <Label className="mb-1 block">Live</Label>
            <Tooltip>
              <TooltipTrigger
                render={
                  <Toggle
                    pressed={isLiveMode}
                    onPressedChange={actions.setLiveMode}
                    variant="outline"
                    aria-label="Toggle live monitor"
                  />
                }
              >
                <Activity />
              </TooltipTrigger>
              <TooltipContent>Live Monitor</TooltipContent>
            </Tooltip>
          </div>
        ) : null}

        <div className="min-w-40">
          <Label className="mb-1 block">View</Label>
          <Select
            value={derived.effectiveView}
            onValueChange={(value) => {
              if (value === "xy" || value === "heatmap") {
                setView(value);
              }
            }}
          >
            <SelectTrigger className="w-full">
              <SelectValue placeholder="Select view">
                {viewLabels[derived.effectiveView]}
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {derived.availableViews.map((view) => (
                <SelectItem key={view} value={view}>
                  {viewLabels[view]}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {showPrimaryProjection ? (
          <>
            <div className="min-w-40">
              <Label className="mb-1 block">Projection</Label>
              <Select
                value={derived.effectiveProjection}
                onValueChange={(value) => {
                  if (
                    value === "trend" ||
                    value === "xy" ||
                    value === "complex_xy"
                  ) {
                    setProjection(value);
                  }
                }}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select projection">
                    {projectionLabels[derived.effectiveProjection]}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {derived.availableProjections.map((projection) => (
                    <SelectItem key={projection.value} value={projection.value}>
                      {projection.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </>
        ) : null}

        {derived.effectiveView === "xy" &&
        derived.effectiveProjection === "trend" ? (
          <div className="min-w-50">
            <Label className="mb-1 block">Series</Label>
            <Select
              value={derived.effectiveTrendSeriesName ?? ""}
              onValueChange={(value) =>
                setTrendSeriesName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select series">
                  {derived.effectiveTrendSeriesName ?? undefined}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {derived.trendSeriesOptions.map((option: ColumnInfo) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {derived.effectiveView === "xy" &&
        derived.effectiveProjection === "xy" ? (
          <>
            <div className="min-w-40">
              <Label className="mb-1 block">X Column</Label>
              <Select
                value={derived.effectiveXYXName ?? ""}
                onValueChange={(value) =>
                  setXYXName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select X">
                    {derived.effectiveXYXName ?? undefined}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {derived.xyXOptions.map((option: ColumnInfo) => (
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
                value={derived.effectiveXYYName ?? ""}
                onValueChange={(value) =>
                  setXYYName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select Y">
                    {derived.effectiveXYYName ?? undefined}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {derived.xyYOptions.map((option: ColumnInfo) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </>
        ) : null}

        {derived.effectiveView === "xy" &&
        derived.effectiveProjection === "complex_xy" ? (
          <div className="min-w-50">
            <Label className="mb-1 block">Complex Series</Label>
            <Select
              value={derived.effectiveComplexXYSeriesName ?? ""}
              onValueChange={(value) =>
                setComplexXYSeriesName(value === "" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select series">
                  {derived.effectiveComplexXYSeriesName ?? undefined}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {derived.complexXYSeriesOptions.map((option: ColumnInfo) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {derived.effectiveView === "heatmap" ? (
          <>
            <div className="min-w-50">
              <Label className="mb-1 block">Series</Label>
              <Select
                value={derived.effectiveHeatmapSeriesName ?? ""}
                onValueChange={(value) =>
                  setHeatmapSeriesName(value === "" ? null : value)
                }
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select series">
                    {derived.effectiveHeatmapSeriesName ?? undefined}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {derived.heatmapSeriesOptions.map((option: ColumnInfo) => (
                    <SelectItem key={option.name} value={option.name}>
                      {option.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </>
        ) : null}

        {showPrimaryOrderBy ? (
          <div className="min-w-40">
            <Label className="mb-1 block">Order By</Label>
            <Select
              value={derived.effectiveOrderByIndexColumnName ?? "__none__"}
              onValueChange={(value) =>
                setOrderByIndexColumnName(value === "__none__" ? null : value)
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="No explicit order">
                  {derived.effectiveOrderByIndexColumnName ?? "None"}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__none__">None</SelectItem>
                {derived.orderByOptions.map((option: ColumnInfo) => (
                  <SelectItem key={option.name} value={option.name}>
                    {option.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {showPrimaryLiveWindow ? (
          <div className="min-w-32">
            <Label className="mb-1 block">Live Window</Label>
            <Select
              value={String(liveWindowCount)}
              onValueChange={(value) => setLiveWindowCount(Number(value))}
            >
              <SelectTrigger className="w-full">
                <SelectValue placeholder="Select window">
                  {String(liveWindowCount)}
                </SelectValue>
              </SelectTrigger>
              <SelectContent>
                {liveWindowOptions.map((option) => (
                  <SelectItem key={option} value={String(option)}>
                    {option}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        ) : null}

        {showAdvancedControls ? (
          <div className="flex flex-col">
            <Label className="mb-1 block opacity-0">Advanced</Label>
            <Button
              type="button"
              variant="outline"
              size="default"
              aria-expanded={advancedOpen}
              onClick={() => setAdvancedOpen((current) => !current)}
            >
              {advancedOpen ? <ChevronDownIcon /> : <ChevronRightIcon />}
              Advanced
            </Button>
          </div>
        ) : null}
      </div>

      {showAdvancedControls && advancedOpen ? (
        <div className="flex flex-wrap items-end gap-1.5 border-t border-border/60 px-1.5 pt-1.5 pb-1.5">
          {showAdvancedStyle ? (
            <div className="min-w-40">
              <Label className="mb-1 block">Style</Label>
              <Select
                value={derived.effectiveDrawStyle ?? "line"}
                onValueChange={(value) => {
                  if (
                    value === "line" ||
                    value === "points" ||
                    value === "line_points"
                  ) {
                    setDrawStyle(value);
                  }
                }}
              >
                <SelectTrigger className="w-full">
                  <SelectValue placeholder="Select style">
                    {derived.effectiveDrawStyle
                      ? drawStyleLabels[derived.effectiveDrawStyle]
                      : undefined}
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  {derived.drawStyleOptions.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          ) : null}

          {showAdvancedHeatmapAxes ? (
            <>
              <div className="min-w-40">
                <Label className="mb-1 block">X Index</Label>
                <Select
                  value={derived.effectiveHeatmapXName ?? ""}
                  onValueChange={(value) =>
                    setHeatmapXName(value === "" ? null : value)
                  }
                  disabled={Boolean(derived.heatmapSeries?.isTrace)}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select X index">
                      {derived.effectiveHeatmapXName ?? undefined}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {derived.heatmapXOptions.map((option: ColumnInfo) => (
                      <SelectItem key={option.name} value={option.name}>
                        {option.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="min-w-40">
                <Label className="mb-1 block">Y Index</Label>
                <Select
                  value={derived.effectiveHeatmapYName ?? ""}
                  onValueChange={(value) =>
                    setHeatmapYName(value === "" ? null : value)
                  }
                >
                  <SelectTrigger className="w-full">
                    <SelectValue placeholder="Select Y index">
                      {derived.effectiveHeatmapYName ?? undefined}
                    </SelectValue>
                  </SelectTrigger>
                  <SelectContent>
                    {derived.heatmapYOptions.map((option: ColumnInfo) => (
                      <SelectItem key={option.name} value={option.name}>
                        {option.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </>
          ) : null}

          {showAdvancedGrouping ? (
            <div className="flex min-w-60 flex-col">
              <Label className="mb-1 block">Split Into Series</Label>
              <div className="flex flex-wrap gap-2 rounded border border-border/60 px-2 py-1.5">
                {derived.groupByOptions.length === 0 ? (
                  <span className="text-xs text-muted-foreground">
                    No remaining index columns
                  </span>
                ) : (
                  derived.groupByOptions.map((option: ColumnInfo) => {
                    const checked =
                      derived.effectiveGroupByIndexColumnNames.includes(
                        option.name,
                      );
                    return (
                      <label
                        key={option.name}
                        className="flex items-center gap-2 text-xs"
                      >
                        <Checkbox
                          checked={checked}
                          onCheckedChange={() =>
                            toggleGroupByIndexColumnName(option.name)
                          }
                        />
                        <span>{option.name}</span>
                      </label>
                    );
                  })
                )}
              </div>
            </div>
          ) : null}

          {showAdvancedComplexControls ? (
            <div className="flex min-w-full flex-wrap items-center gap-2 text-xs">
              <span className="font-medium">Complex:</span>
              {derived.effectiveView === "heatmap" ? (
                <RadioGroup
                  className="flex flex-wrap gap-2"
                  value={selectedComplexViewSingle}
                  onValueChange={(value: string) => {
                    if (derived.complexControlsDisabled) return;
                    if (isComplexViewOption(value)) {
                      setSelectedComplexViewSingle(value);
                    }
                  }}
                >
                  {complexSeriesOptions.map((option) => (
                    <label key={option} className="flex items-center gap-2">
                      <RadioGroupItem
                        value={option}
                        disabled={derived.complexControlsDisabled}
                      />
                      <span
                        className={cn(
                          "text-xs",
                          derived.complexControlsDisabled && "opacity-50",
                        )}
                      >
                        {complexViewLabels[option]}
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
                          disabled={derived.complexControlsDisabled}
                          onCheckedChange={(checked) => {
                            if (derived.complexControlsDisabled) return;
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
                            derived.complexControlsDisabled && "opacity-50",
                          )}
                        >
                          {complexViewLabels[option]}
                        </span>
                      </label>
                    );
                  })}
                </div>
              )}
            </div>
          ) : null}
        </div>
      ) : null}

      {isLiveMode && derived.effectiveView === "xy" ? (
        <div className="px-1.5 pb-1.5 text-xs text-muted-foreground">
          {derived.xyUsesTraceSource ? (
            <span>Live mode shows the last {liveWindowCount} sweeps.</span>
          ) : derived.liveMonitorGroupByIndexColumnNames.length > 0 ? (
            <span>
              Live mode shows the last {liveWindowCount} sweeps, grouped by{" "}
              {derived.liveMonitorGroupByIndexColumnNames.join(", ")} and
              ordered by {derived.liveMonitorOrderByIndexColumnName}.
            </span>
          ) : derived.liveMonitorUsesForcedRoles ? (
            <span>
              Live mode shows the last {liveWindowCount} updates, ordered by{" "}
              {derived.liveMonitorOrderByIndexColumnName}.
            </span>
          ) : (
            <span>Live mode shows the last {liveWindowCount} updates.</span>
          )}
        </div>
      ) : null}
    </>
  );
}
