import { useLayoutEffect, useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ChartOptions } from "@/shared/lib/chartTypes";
import { resolveXYYAxisLabel } from "../model/seriesLabeling";
import { useWebGLChart, type ChartInteractionState } from "./useWebGLChart";

const noop = () => undefined;

const glStub = {
  createShader: () => ({}),
  shaderSource: noop,
  compileShader: noop,
  getShaderParameter: () => true,
  getShaderInfoLog: () => "",
  createProgram: () => ({}),
  attachShader: noop,
  linkProgram: noop,
  getProgramParameter: () => true,
  getProgramInfoLog: () => "",
  detachShader: noop,
  deleteShader: noop,
  createBuffer: () => ({}),
  bindBuffer: noop,
  bufferData: noop,
  bufferSubData: noop,
  useProgram: noop,
  getUniformLocation: () => ({}),
  getAttribLocation: () => 0,
  uniformMatrix3fv: noop,
  uniform4f: noop,
  uniform1f: noop,
  uniform3fv: noop,
  enableVertexAttribArray: noop,
  vertexAttribPointer: noop,
  vertexAttribDivisor: noop,
  drawArrays: noop,
  drawArraysInstanced: noop,
  createVertexArray: () => ({}),
  bindVertexArray: noop,
  deleteVertexArray: noop,
  deleteBuffer: noop,
  deleteProgram: noop,
  viewport: noop,
  scissor: noop,
  enable: noop,
  disable: noop,
  clearColor: noop,
  clear: noop,
  blendFunc: noop,
  VERTEX_SHADER: 0x8b31,
  FRAGMENT_SHADER: 0x8b30,
  COMPILE_STATUS: 0x8b81,
  LINK_STATUS: 0x8b82,
  ARRAY_BUFFER: 0x8892,
  STATIC_DRAW: 0x88e4,
  FLOAT: 0x1406,
  LINE_STRIP: 0x0003,
  POINTS: 0x0000,
  TRIANGLES: 0x0004,
  COLOR_BUFFER_BIT: 0x4000,
  SCISSOR_TEST: 0x0c11,
  BLEND: 0x0be2,
  SRC_ALPHA: 0x0302,
  ONE_MINUS_SRC_ALPHA: 0x0303,
};

HTMLCanvasElement.prototype.getContext = vi.fn().mockReturnValue(glStub);

interface HookHarnessProps {
  data?: ChartOptions;
  interactionKey?: string | null;
  liveMode?: boolean;
}

function serializeInteractionState(state: ChartInteractionState | null) {
  if (!state) return null;
  if ("zoomState" in state) {
    return {
      type: state.type,
      xMin: state.xMin,
      xMax: state.xMax,
      yMin: state.yMin,
      yMax: state.yMax,
      zoomState: state.zoomState,
    };
  }
  return {
    type: state.type,
    xCategories: state.xCategories,
    yCategories: state.yCategories,
  };
}

function HookHarness({
  data,
  interactionKey,
  liveMode = false,
}: HookHarnessProps) {
  const { canvasRef, svgRef, getInteractionState } = useWebGLChart({
    data,
    interactionKey,
    liveMode,
    theme: "light",
  });
  const [snapshot, setSnapshot] = useState("null");

  useLayoutEffect(() => {
    const canvas = canvasRef.current;
    const svg = svgRef.current;
    if (!canvas || !svg) return;

    Object.defineProperty(canvas, "clientWidth", {
      configurable: true,
      value: 300,
    });
    Object.defineProperty(canvas, "clientHeight", {
      configurable: true,
      value: 200,
    });
    Object.defineProperty(svg, "clientWidth", {
      configurable: true,
      value: 300,
    });
    Object.defineProperty(svg, "clientHeight", {
      configurable: true,
      value: 200,
    });
    Object.defineProperty(svg, "getBoundingClientRect", {
      configurable: true,
      value: () => ({
        left: 0,
        top: 0,
        width: 300,
        height: 200,
        right: 300,
        bottom: 200,
        x: 0,
        y: 0,
        toJSON: () => undefined,
      }),
    });
  }, [canvasRef, svgRef]);

  return (
    <div>
      <canvas ref={canvasRef} />
      <svg ref={svgRef} data-testid="chart-svg" />
      <button
        type="button"
        onClick={() =>
          setSnapshot(
            JSON.stringify(serializeInteractionState(getInteractionState())),
          )
        }
      >
        Snapshot
      </button>
      <output data-testid="snapshot">{snapshot}</output>
    </div>
  );
}

function makeLineData(
  points: [number, number][],
): Extract<ChartOptions, { type: "xy" }> {
  return {
    type: "xy",
    projection: "trend",
    drawStyle: "line",
    xName: "x",
    yName: null,
    series: [xySeries("sig", "sig", points)],
  };
}

function makeHeatmapData(): Extract<ChartOptions, { type: "heatmap" }> {
  return {
    type: "heatmap",
    xName: "x",
    yName: "y",
    xCategories: [0, 1],
    yCategories: [0, 1],
    series: [
      xyzSeries("z", "z", [
        [0, 0, 1],
        [1, 1, 2],
      ]),
    ],
  };
}

function xySeries(id: string, label: string, points: [number, number][]) {
  return {
    id,
    label,
    pointCount: points.length,
    values: Float64Array.from(points.flat()),
  };
}

function xyzSeries(
  id: string,
  label: string,
  points: [number, number, number][],
) {
  return {
    id,
    label,
    pointCount: points.length,
    values: Float64Array.from(points.flat()),
  };
}

function readSnapshot() {
  return JSON.parse(screen.getByTestId("snapshot").textContent ?? "null") as {
    type: string;
    xMin?: number;
    xMax?: number;
    yMin?: number;
    yMax?: number;
    zoomState?: {
      scaleX: number;
      scaleY: number;
      translateX: number;
      translateY: number;
    };
  } | null;
}

describe("useWebGLChart", () => {
  let originalRequestAnimationFrame: typeof requestAnimationFrame;
  let originalCancelAnimationFrame: typeof cancelAnimationFrame;

  beforeEach(() => {
    originalRequestAnimationFrame = globalThis.requestAnimationFrame;
    originalCancelAnimationFrame = globalThis.cancelAnimationFrame;
    vi.spyOn(performance, "now").mockReturnValue(0);
    globalThis.requestAnimationFrame = ((callback: FrameRequestCallback) => {
      callback(250);
      return 1;
    }) as typeof requestAnimationFrame;
    globalThis.cancelAnimationFrame = (() =>
      undefined) as typeof cancelAnimationFrame;
  });

  afterEach(() => {
    vi.restoreAllMocks();
    globalThis.requestAnimationFrame = originalRequestAnimationFrame;
    globalThis.cancelAnimationFrame = originalCancelAnimationFrame;
  });

  it("preserves a manual live zoom until reset, then auto-follows again", () => {
    const { rerender } = render(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [10, 10],
        ])}
        interactionKey="live-line"
        liveMode
      />,
    );

    const svg = screen.getByTestId("chart-svg");
    const snapshotButton = screen.getByRole("button", { name: "Snapshot" });

    fireEvent.wheel(svg, { clientX: 170, clientY: 90, deltaY: -240 });
    fireEvent.click(snapshotButton);
    const zoomed = readSnapshot();
    expect(zoomed?.zoomState?.scaleX).toBeGreaterThan(1);
    expect(zoomed?.zoomState?.scaleY).toBeGreaterThan(1);
    const zoomedView = zoomed;

    rerender(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [20, 20],
        ])}
        interactionKey="live-line"
        liveMode
      />,
    );

    fireEvent.click(snapshotButton);
    const persisted = readSnapshot();
    expect(persisted).toEqual(zoomedView);

    fireEvent.dblClick(svg, { clientX: 170, clientY: 90 });
    fireEvent.click(snapshotButton);
    const reset = readSnapshot();
    expect(reset?.zoomState?.scaleX).toBeCloseTo(1);
    expect(reset?.zoomState?.scaleY).toBeCloseTo(1);
    expect(reset?.zoomState?.translateX).toBeCloseTo(0);
    expect(reset?.zoomState?.translateY).toBeCloseTo(0);

    rerender(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [40, 40],
        ])}
        interactionKey="live-line"
        liveMode
      />,
    );

    fireEvent.click(snapshotButton);
    const autoFollow = readSnapshot();
    expect(autoFollow?.zoomState?.scaleX).toBeCloseTo(1);
    expect(autoFollow?.zoomState?.scaleY).toBeCloseTo(1);
    expect(autoFollow?.zoomState?.translateX).toBeCloseTo(0);
    expect(autoFollow?.zoomState?.translateY).toBeCloseTo(0);
    expect(autoFollow?.xMax).toBeGreaterThan(reset?.xMax ?? 0);
    expect(autoFollow?.yMax).toBeGreaterThan(reset?.yMax ?? 0);
  });

  it("uses the shared trend series label for the y axis when available", () => {
    expect(
      resolveXYYAxisLabel({
        projection: "trend",
        yName: null,
        series: [
          xySeries("sig-real", "signal (real)", [
            [0, 1],
            [1, 2],
          ]),
          xySeries("sig-imag", "signal (imag)", [
            [0, 2],
            [1, 3],
          ]),
        ],
      }),
    ).toBe("signal");
  });

  it("resets non-live charts back to the default view on data changes", () => {
    const { rerender } = render(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [10, 10],
        ])}
        interactionKey="static-line"
      />,
    );

    const svg = screen.getByTestId("chart-svg");
    const snapshotButton = screen.getByRole("button", { name: "Snapshot" });

    fireEvent.wheel(svg, { clientX: 170, clientY: 90, deltaY: -240 });
    fireEvent.click(snapshotButton);
    const zoomed = readSnapshot();
    expect(zoomed?.zoomState?.scaleX).toBeGreaterThan(1);

    rerender(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [20, 20],
        ])}
        interactionKey="static-line"
      />,
    );

    fireEvent.click(snapshotButton);
    const reset = readSnapshot();
    expect(reset?.zoomState?.scaleX).toBeCloseTo(1);
    expect(reset?.zoomState?.scaleY).toBeCloseTo(1);
    expect(reset?.zoomState?.translateX).toBeCloseTo(0);
    expect(reset?.zoomState?.translateY).toBeCloseTo(0);
    expect(reset?.xMax).toBeGreaterThan(zoomed?.xMax ?? 0);
    expect(reset?.yMax).toBeGreaterThan(zoomed?.yMax ?? 0);
  });

  it("keeps auto-follow when live data updates during reset animation", () => {
    const { rerender } = render(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [10, 10],
        ])}
        interactionKey="live-line"
        liveMode
      />,
    );

    const svg = screen.getByTestId("chart-svg");
    const snapshotButton = screen.getByRole("button", { name: "Snapshot" });

    fireEvent.wheel(svg, { clientX: 170, clientY: 90, deltaY: -240 });

    const queuedFrames: FrameRequestCallback[] = [];
    globalThis.requestAnimationFrame = ((callback: FrameRequestCallback) => {
      queuedFrames.push(callback);
      return queuedFrames.length;
    }) as typeof requestAnimationFrame;

    fireEvent.dblClick(svg, { clientX: 170, clientY: 90 });

    rerender(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [20, 20],
        ])}
        interactionKey="live-line"
        liveMode
      />,
    );

    while (queuedFrames.length > 0) {
      const callback = queuedFrames.shift();
      callback?.(250);
    }

    fireEvent.click(snapshotButton);
    const reset = readSnapshot();
    expect(reset?.zoomState?.scaleX).toBeCloseTo(1);
    expect(reset?.zoomState?.scaleY).toBeCloseTo(1);
    expect(reset?.xMax).toBeGreaterThan(10);

    rerender(
      <HookHarness
        data={makeLineData([
          [0, 0],
          [40, 40],
        ])}
        interactionKey="live-line"
        liveMode
      />,
    );

    while (queuedFrames.length > 0) {
      const callback = queuedFrames.shift();
      callback?.(250);
    }

    fireEvent.click(snapshotButton);
    const followed = readSnapshot();
    expect(followed?.zoomState?.scaleX).toBeCloseTo(1);
    expect(followed?.xMax).toBeGreaterThan(reset?.xMax ?? 0);
  });

  it("does not expose zoom interactions for heatmaps", () => {
    render(
      <HookHarness
        data={makeHeatmapData()}
        interactionKey="live-heatmap"
        liveMode
      />,
    );

    const svg = screen.getByTestId("chart-svg");
    const snapshotButton = screen.getByRole("button", { name: "Snapshot" });

    fireEvent.wheel(svg, { clientX: 170, clientY: 90, deltaY: -240 });
    fireEvent.dblClick(svg, { clientX: 170, clientY: 90 });
    fireEvent.click(snapshotButton);

    expect(readSnapshot()).toEqual({
      type: "heatmap",
      xCategories: [0, 1],
      yCategories: [0, 1],
    });
  });
});
