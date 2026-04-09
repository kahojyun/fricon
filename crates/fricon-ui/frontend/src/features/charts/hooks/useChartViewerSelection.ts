import { useState } from "react";
import type { ColumnInfo, DatasetStatus } from "../api/types";
import type {
  ChartView,
  ComplexViewOption,
  XYDrawStyle,
  XYProjection,
} from "@/shared/lib/chartTypes";
import { deriveChartViewerState } from "../model/chartViewerLogic";

export interface ChartViewerControlState {
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
  isLiveMode: boolean;
}

interface LiveModeSelection {
  datasetStatus?: DatasetStatus;
  value: boolean;
}

export interface ChartViewerControlActions {
  setView: (next: ChartView) => void;
  setProjection: (next: XYProjection) => void;
  setDrawStyle: (next: XYDrawStyle) => void;
  setTrendSeriesName: (next: string | null) => void;
  setHeatmapSeriesName: (next: string | null) => void;
  setComplexXYSeriesName: (next: string | null) => void;
  setXYXName: (next: string | null) => void;
  setXYYName: (next: string | null) => void;
  setHeatmapXName: (next: string | null) => void;
  setHeatmapYName: (next: string | null) => void;
  toggleGroupByIndexColumnName: (name: string) => void;
  setOrderByIndexColumnName: (next: string | null) => void;
  setSelectedComplexView: (next: ComplexViewOption[]) => void;
  setSelectedComplexViewSingle: (next: ComplexViewOption) => void;
  setLiveMode: (next: boolean) => void;
}

function defaultDrawStyle(projection: XYProjection): XYDrawStyle {
  if (projection === "trend") return "line";
  return "points";
}

export function useChartViewerSelection(
  columns: ColumnInfo[],
  datasetStatus?: DatasetStatus,
) {
  const [view, setView] = useState<ChartView>("xy");
  const [projection, setProjection] = useState<XYProjection>("trend");
  const [drawStyle, setDrawStyle] = useState<XYDrawStyle>(() =>
    defaultDrawStyle("trend"),
  );

  const [selectedComplexView, setSelectedComplexView] = useState<
    ComplexViewOption[]
  >(["real", "imag"]);
  const [selectedComplexViewSingle, setSelectedComplexViewSingle] =
    useState<ComplexViewOption>("mag");

  const [trendSeriesName, setTrendSeriesName] = useState<string | null>(null);
  const [heatmapSeriesName, setHeatmapSeriesName] = useState<string | null>(
    null,
  );
  const [complexXYSeriesName, setComplexXYSeriesName] = useState<string | null>(
    null,
  );
  const [xyXName, setXyXName] = useState<string | null>(null);
  const [xyYName, setXyYName] = useState<string | null>(null);
  const [heatmapXName, setHeatmapXName] = useState<string | null>(null);
  const [heatmapYName, setHeatmapYName] = useState<string | null>(null);
  const [groupByIndexColumnNames, setGroupByIndexColumnNames] = useState<
    string[]
  >([]);
  const [orderByIndexColumnName, setOrderByIndexColumnName] = useState<
    string | null
  >(null);

  const [liveModeSelection, setLiveModeSelection] =
    useState<LiveModeSelection | null>(null);
  const currentLiveModeSelection =
    liveModeSelection?.datasetStatus === datasetStatus
      ? liveModeSelection
      : null;
  const isLiveMode =
    currentLiveModeSelection?.value ?? datasetStatus === "Writing";

  const derived = deriveChartViewerState(columns, {
    view,
    projection,
    drawStyle,
    trendSeriesName,
    heatmapSeriesName,
    complexXYSeriesName,
    xyXName,
    xyYName,
    heatmapXName,
    heatmapYName,
    groupByIndexColumnNames,
    orderByIndexColumnName,
  });

  const controlState: ChartViewerControlState = {
    selectedComplexView,
    selectedComplexViewSingle,
    isLiveMode,
  };

  const actions: ChartViewerControlActions = {
    setView,
    setProjection: (next) => {
      setProjection(next);
      setDrawStyle((current) =>
        current === defaultDrawStyle(projection)
          ? defaultDrawStyle(next)
          : current,
      );
    },
    setDrawStyle,
    setTrendSeriesName,
    setHeatmapSeriesName,
    setComplexXYSeriesName,
    setXYXName: setXyXName,
    setXYYName: setXyYName,
    setHeatmapXName,
    setHeatmapYName,
    toggleGroupByIndexColumnName: (name) => {
      setGroupByIndexColumnNames((current) =>
        current.includes(name)
          ? current.filter((item) => item !== name)
          : [...current, name],
      );
    },
    setOrderByIndexColumnName,
    setSelectedComplexView,
    setSelectedComplexViewSingle,
    setLiveMode: (next) =>
      setLiveModeSelection({
        datasetStatus,
        value: next,
      }),
  };

  return {
    derived,
    controlState,
    actions,
  };
}
