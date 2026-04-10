import { useState } from "react";
import type { ColumnInfo, DatasetStatus } from "../api/types";
import type {
  ChartView,
  ComplexViewOption,
  NumericLabelFormatOptions,
  NumericLabelFormatMode,
  XYDrawStyle,
  XYPlotMode,
} from "@/shared/lib/chartTypes";
import { deriveChartViewerState } from "../model/chartViewerLogic";
import { DEFAULT_NUMERIC_LABEL_FORMAT } from "../rendering/numericLabelFormat";

export interface ChartViewerControlState {
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
  isLiveMode: boolean;
  liveWindowCount: number;
  numericLabelFormat: NumericLabelFormatOptions;
}

interface LiveModeSelection {
  datasetStatus?: DatasetStatus;
  value: boolean;
}

export interface ChartViewerControlActions {
  setView: (next: ChartView) => void;
  setPlotMode: (next: XYPlotMode) => void;
  setDrawStyle: (next: XYDrawStyle) => void;
  setSweepQuantityName: (next: string | null) => void;
  setHeatmapQuantityName: (next: string | null) => void;
  setComplexPlaneQuantityName: (next: string | null) => void;
  setXYXName: (next: string | null) => void;
  setXYYName: (next: string | null) => void;
  setHeatmapXName: (next: string | null) => void;
  setHeatmapYName: (next: string | null) => void;
  toggleTraceGroupIndexColumnName: (name: string) => void;
  setSweepIndexColumnName: (next: string | null) => void;
  setSelectedComplexView: (next: ComplexViewOption[]) => void;
  setSelectedComplexViewSingle: (next: ComplexViewOption) => void;
  setLiveMode: (next: boolean) => void;
  setLiveWindowCount: (next: number) => void;
  setNumericLabelFormatMode: (next: NumericLabelFormatMode) => void;
  setNumericLabelSignificantDigits: (next: number) => void;
}

export function useChartViewerSelection(
  columns: ColumnInfo[],
  datasetStatus?: DatasetStatus,
) {
  const [view, setView] = useState<ChartView>("xy");
  const [plotMode, setPlotMode] = useState<XYPlotMode>("quantity_vs_sweep");
  const [drawStyle, setDrawStyle] = useState<XYDrawStyle>("line");

  const [selectedComplexView, setSelectedComplexView] = useState<
    ComplexViewOption[]
  >(["real", "imag"]);
  const [selectedComplexViewSingle, setSelectedComplexViewSingle] =
    useState<ComplexViewOption>("mag");

  const [sweepQuantityName, setSweepQuantityName] = useState<string | null>(
    null,
  );
  const [heatmapQuantityName, setHeatmapQuantityName] = useState<string | null>(
    null,
  );
  const [complexPlaneQuantityName, setComplexPlaneQuantityName] = useState<
    string | null
  >(null);
  const [xyXName, setXyXName] = useState<string | null>(null);
  const [xyYName, setXyYName] = useState<string | null>(null);
  const [heatmapXName, setHeatmapXName] = useState<string | null>(null);
  const [heatmapYName, setHeatmapYName] = useState<string | null>(null);
  const [traceGroupIndexColumnNames, setTraceGroupIndexColumnNames] = useState<
    string[]
  >([]);
  const [sweepIndexColumnName, setSweepIndexColumnName] = useState<
    string | null
  >(null);

  const [liveModeSelection, setLiveModeSelection] =
    useState<LiveModeSelection | null>(null);
  const [liveWindowCount, setLiveWindowCount] = useState(5);
  const [numericLabelFormat, setNumericLabelFormat] =
    useState<NumericLabelFormatOptions>(DEFAULT_NUMERIC_LABEL_FORMAT);
  const currentLiveModeSelection =
    liveModeSelection?.datasetStatus === datasetStatus
      ? liveModeSelection
      : null;
  const isLiveMode =
    currentLiveModeSelection?.value ?? datasetStatus === "Writing";

  const derived = deriveChartViewerState(columns, {
    view,
    plotMode,
    drawStyle,
    sweepQuantityName,
    heatmapQuantityName,
    complexPlaneQuantityName,
    xyXName,
    xyYName,
    heatmapXName,
    heatmapYName,
    traceGroupIndexColumnNames,
    sweepIndexColumnName,
  });

  const controlState: ChartViewerControlState = {
    selectedComplexView,
    selectedComplexViewSingle,
    isLiveMode,
    liveWindowCount,
    numericLabelFormat,
  };

  const actions: ChartViewerControlActions = {
    setView,
    setPlotMode,
    setDrawStyle,
    setSweepQuantityName,
    setHeatmapQuantityName,
    setComplexPlaneQuantityName,
    setXYXName: setXyXName,
    setXYYName: setXyYName,
    setHeatmapXName,
    setHeatmapYName,
    toggleTraceGroupIndexColumnName: (name) => {
      setTraceGroupIndexColumnNames((current) =>
        current.includes(name)
          ? current.filter((item) => item !== name)
          : [...current, name],
      );
    },
    setSweepIndexColumnName,
    setSelectedComplexView,
    setSelectedComplexViewSingle,
    setLiveMode: (next) =>
      setLiveModeSelection({
        datasetStatus,
        value: next,
      }),
    setLiveWindowCount,
    setNumericLabelFormatMode: (next) =>
      setNumericLabelFormat((current) => ({ ...current, mode: next })),
    setNumericLabelSignificantDigits: (next) =>
      setNumericLabelFormat((current) => ({
        ...current,
        significantDigits: next,
      })),
  };

  return {
    derived,
    controlState,
    actions,
  };
}
