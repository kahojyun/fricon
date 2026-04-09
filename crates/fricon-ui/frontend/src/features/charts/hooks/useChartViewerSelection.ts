import { useState } from "react";
import type { ColumnInfo, DatasetStatus } from "../api/types";
import type {
  ChartType,
  ComplexViewOption,
  ScatterMode,
} from "@/shared/lib/chartTypes";
import { deriveChartViewerState } from "../model/chartViewerLogic";

export interface ChartViewerControlState {
  selectedComplexView: ComplexViewOption[];
  selectedComplexViewSingle: ComplexViewOption;
  isLiveMode: boolean;
}

export interface ChartViewerControlActions {
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
  setSelectedComplexView: (next: ComplexViewOption[]) => void;
  setSelectedComplexViewSingle: (next: ComplexViewOption) => void;
  setLiveMode: (next: boolean) => void;
}

export function useChartViewerSelection(
  columns: ColumnInfo[],
  datasetStatus?: DatasetStatus,
) {
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

  const [isLiveMode, setIsLiveMode] = useState(datasetStatus === "Writing");

  // Auto-toggle live mode on dataset status transitions.
  // Uses the "adjusting state during rendering" pattern (stores previous
  // status in state instead of using an effect) so the React compiler is happy.
  const [prevDatasetStatus, setPrevDatasetStatus] = useState(datasetStatus);
  if (prevDatasetStatus !== datasetStatus) {
    setPrevDatasetStatus(datasetStatus);
    if (datasetStatus === "Writing") {
      setIsLiveMode(true);
    } else if (prevDatasetStatus === "Writing") {
      setIsLiveMode(false);
    }
  }

  const derived = deriveChartViewerState(columns, {
    chartType,
    seriesName,
    xColumnName,
    yColumnName,
    scatterMode,
    scatterSeriesName,
    scatterTraceXName,
    scatterTraceYName,
    scatterXName,
    scatterYName,
  });

  const controlState: ChartViewerControlState = {
    selectedComplexView,
    selectedComplexViewSingle,
    isLiveMode,
  };

  const actions: ChartViewerControlActions = {
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
    setSelectedComplexView,
    setSelectedComplexViewSingle,
    setLiveMode: setIsLiveMode,
  };

  return {
    derived,
    controlState,
    actions,
  };
}
