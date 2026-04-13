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
          plotMode: "quantity_vs_sweep",
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
          effectivePlotMode: "quantity_vs_sweep",
          effectiveDrawStyle: "line",
          sweepQuantity: { name: "signal", isComplex: true },
          heatmapQuantity: undefined,
          complexPlaneQuantity: undefined,
          xyXColumn: undefined,
          xyYColumn: undefined,
          xyUsesTraceSource: false,
          liveMonitorUsesForcedRoles: true,
          liveMonitorSweepIndexColumnName: "idx_step",
          liveMonitorTraceGroupIndexColumnNames: ["idx_cycle"],
          xyRoleControlsVisible: true,
          effectiveSweepIndexColumnName: "idx_step",
          effectiveTraceGroupIndexColumnNames: ["idx_cycle"],
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
          plotMode: "xy",
          drawStyle: "points",
          xName: "current",
          yName: "voltage",
          series: [],
        },
        derived: {
          effectiveView: "xy",
          effectivePlotMode: "xy",
          effectiveDrawStyle: "points",
          sweepQuantity: undefined,
          heatmapQuantity: undefined,
          complexPlaneQuantity: undefined,
          xyXColumn: { name: "current" },
          xyYColumn: { name: "voltage" },
          xyUsesTraceSource: false,
          liveMonitorUsesForcedRoles: false,
          liveMonitorSweepIndexColumnName: null,
          liveMonitorTraceGroupIndexColumnNames: [],
          xyRoleControlsVisible: false,
          effectiveSweepIndexColumnName: null,
          effectiveTraceGroupIndexColumnNames: [],
        } as never,
        isLiveMode: false,
        liveWindowCount: 5,
      }),
    ).toEqual({
      title: "Dataset #7",
      meta: [],
    });
  });

  it("omits live window metadata for live heatmaps", () => {
    expect(
      buildChartFrameHeader({
        datasetId: 11,
        datasetStatus: "Writing",
        data: {
          type: "heatmap",
          xName: "x",
          yName: "y",
          series: [],
        },
        derived: {
          effectiveView: "heatmap",
          effectivePlotMode: "quantity_vs_sweep",
          effectiveDrawStyle: null,
          sweepQuantity: undefined,
          heatmapQuantity: { name: "signal" },
          complexPlaneQuantity: undefined,
          xyXColumn: undefined,
          xyYColumn: undefined,
          xyUsesTraceSource: false,
          liveMonitorUsesForcedRoles: false,
          liveMonitorSweepIndexColumnName: null,
          liveMonitorTraceGroupIndexColumnNames: [],
          xyRoleControlsVisible: false,
          effectiveSweepIndexColumnName: null,
          effectiveTraceGroupIndexColumnNames: [],
        } as never,
        isLiveMode: true,
        liveWindowCount: 5,
      }),
    ).toEqual({
      title: "Dataset #11",
      meta: ["Live Acquisition"],
    });
  });
});
