import type { DatasetStatus } from "../api/types";
import { deriveChartViewerState } from "../model/chartViewerLogic";
import type { ChartOptions } from "@/shared/lib/chartTypes";

export interface ChartFrameHeaderData {
  title: string;
  meta: string[];
}

interface BuildChartFrameHeaderArgs {
  datasetId: number;
  datasetStatus?: DatasetStatus;
  data?: ChartOptions;
  derived: ReturnType<typeof deriveChartViewerState>;
  isLiveMode: boolean;
  liveWindowCount: number;
}

export function buildChartFrameHeader({
  datasetId,
  datasetStatus,
  data,
  derived,
  isLiveMode,
  liveWindowCount,
}: BuildChartFrameHeaderArgs): ChartFrameHeaderData | null {
  if (!data) {
    return null;
  }

  const meta: string[] = [];
  const statusText = headerStatusText(datasetStatus, isLiveMode);
  if (statusText) {
    meta.push(statusText);
  }

  const groupBy =
    isLiveMode && derived.liveMonitorUsesForcedRoles
      ? derived.liveMonitorGroupByIndexColumnNames
      : !isLiveMode && derived.xyRoleControlsVisible
        ? derived.effectiveGroupByIndexColumnNames
        : [];
  if (groupBy.length > 0) {
    meta.push(`grouped by ${groupBy.join(", ")}`);
  }
  if (isLiveMode) {
    meta.push(`recent ${liveWindowCount} ${liveWindowUnit(derived)}`);
  }

  return {
    title: `Dataset #${datasetId}`,
    meta,
  };
}

function liveWindowUnit(derived: ReturnType<typeof deriveChartViewerState>) {
  return derived.xyUsesTraceSource ||
    derived.liveMonitorGroupByIndexColumnNames.length > 0
    ? "sweeps"
    : "points";
}

function headerStatusText(
  datasetStatus: DatasetStatus | undefined,
  isLiveMode: boolean,
) {
  if (isLiveMode) {
    return datasetStatus === "Writing" ? "Live Acquisition" : "Live View";
  }
  if (datasetStatus === "Aborted") {
    return "Aborted run";
  }
  return null;
}
