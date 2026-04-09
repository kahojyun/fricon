import { describe, expect, it } from "vitest";
import { buildChartFrameHeader } from "./chartFrameHeaderModel";

describe("buildChartFrameHeader", () => {
  it("describes live trend charts with dataset provenance and roles", () => {
    expect(
      buildChartFrameHeader({
        datasetId: 42,
        datasetStatus: "Writing",
        data: {
          type: "xy",
          projection: "trend",
          drawStyle: "line",
          xName: "step",
          yName: null,
          series: [
            {
              id: "sig-real",
              label: "signal (real)",
              pointCount: 1,
              values: new Float64Array([0, 0]),
            },
            {
              id: "sig-imag",
              label: "signal (imag)",
              pointCount: 1,
              values: new Float64Array([1, 1]),
            },
          ],
        },
        derived: {
          effectiveView: "xy",
          effectiveProjection: "trend",
          effectiveDrawStyle: "line",
          trendSeries: { name: "signal", isComplex: true },
          heatmapSeries: undefined,
          complexXYSeries: undefined,
          xyXColumn: undefined,
          xyYColumn: undefined,
          xyUsesTraceSource: false,
          liveMonitorUsesForcedRoles: true,
          liveMonitorOrderByIndexColumnName: "idx_step",
          liveMonitorGroupByIndexColumnNames: ["idx_cycle"],
          xyRoleControlsVisible: true,
          effectiveOrderByIndexColumnName: "idx_step",
          effectiveGroupByIndexColumnNames: ["idx_cycle"],
        } as never,
        isLiveMode: true,
        liveWindowCount: 5,
      }),
    ).toEqual({
      title: "Dataset #42",
      meta: ["Live Acquisition", "grouped by idx_cycle", "recent 5 sweeps"],
    });
  });

  it("keeps static completed charts minimal", () => {
    expect(
      buildChartFrameHeader({
        datasetId: 7,
        datasetStatus: "Completed",
        data: {
          type: "xy",
          projection: "xy",
          drawStyle: "points",
          xName: "current",
          yName: "voltage",
          series: [],
        },
        derived: {
          effectiveView: "xy",
          effectiveProjection: "xy",
          effectiveDrawStyle: "points",
          trendSeries: undefined,
          heatmapSeries: undefined,
          complexXYSeries: undefined,
          xyXColumn: { name: "current" },
          xyYColumn: { name: "voltage" },
          xyUsesTraceSource: false,
          liveMonitorUsesForcedRoles: false,
          liveMonitorOrderByIndexColumnName: null,
          liveMonitorGroupByIndexColumnNames: [],
          xyRoleControlsVisible: false,
          effectiveOrderByIndexColumnName: null,
          effectiveGroupByIndexColumnNames: [],
        } as never,
        isLiveMode: false,
        liveWindowCount: 5,
      }),
    ).toEqual({
      title: "Dataset #7",
      meta: [],
    });
  });
});
