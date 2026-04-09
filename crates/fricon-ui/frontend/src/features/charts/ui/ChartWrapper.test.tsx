import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ChartOptions } from "@/shared/lib/chartTypes";
import { ChartWrapper } from "./ChartWrapper";

// Stub WebGL2 context so the hook can initialize without a real GPU
const noop = () => undefined;

const glStub = {
  createShader: () => ({}),
  shaderSource: () => undefined,
  compileShader: () => undefined,
  getShaderParameter: () => true,
  getShaderInfoLog: () => "",
  createProgram: () => ({}),
  attachShader: () => undefined,
  linkProgram: () => undefined,
  getProgramParameter: () => true,
  getProgramInfoLog: () => "",
  detachShader: () => undefined,
  deleteShader: () => undefined,
  createBuffer: () => ({}),
  bindBuffer: () => undefined,
  bufferData: () => undefined,
  bufferSubData: () => undefined,
  useProgram: noop,
  getUniformLocation: () => ({}),
  getAttribLocation: () => 0,
  uniformMatrix3fv: () => undefined,
  uniform4f: () => undefined,
  uniform1f: () => undefined,
  uniform3fv: () => undefined,
  enableVertexAttribArray: () => undefined,
  vertexAttribPointer: () => undefined,
  vertexAttribDivisor: () => undefined,
  drawArrays: () => undefined,
  drawArraysInstanced: () => undefined,
  createVertexArray: () => ({}),
  bindVertexArray: () => undefined,
  deleteVertexArray: () => undefined,
  deleteBuffer: () => undefined,
  deleteProgram: () => undefined,
  viewport: () => undefined,
  scissor: () => undefined,
  enable: () => undefined,
  disable: () => undefined,
  clearColor: () => undefined,
  clear: () => undefined,
  blendFunc: () => undefined,
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

vi.mock("next-themes", () => ({
  useTheme: () => ({ resolvedTheme: "light" }),
}));

describe("ChartWrapper", () => {
  it("shows empty state when no data is provided", () => {
    render(<ChartWrapper />);
    expect(screen.getByText("No chart data")).toBeInTheDocument();
  });

  it("renders a canvas and svg for line chart data", () => {
    const data: ChartOptions = {
      type: "line",
      xName: "x",
      series: [
        xySeries("s1", "s1", [
          [0, 1],
          [1, 2],
        ]),
      ],
    };

    const { container } = render(<ChartWrapper data={data} />);
    expect(container.querySelector("canvas")).toBeInTheDocument();
    expect(container.querySelector("svg")).toBeInTheDocument();
    expect(screen.queryByText("No chart data")).not.toBeInTheDocument();
  });

  it("keeps svg interactions enabled in live mode for zoomable charts", () => {
    const data: ChartOptions = {
      type: "line",
      xName: "x",
      series: [
        xySeries("s1", "s1", [
          [0, 1],
          [1, 2],
        ]),
      ],
    };

    const { container } = render(<ChartWrapper data={data} liveMode />);
    expect(container.querySelector("svg")).toHaveStyle({
      pointerEvents: "auto",
    });
  });

  it("renders a canvas for scatter chart data", () => {
    const data: ChartOptions = {
      type: "scatter",
      xName: "x",
      yName: "y",
      series: [
        xySeries("s1", "s1", [
          [0, 1],
          [1, 2],
        ]),
      ],
    };

    const { container } = render(<ChartWrapper data={data} />);
    expect(container.querySelector("canvas")).toBeInTheDocument();
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("renders a canvas for heatmap chart data", () => {
    const data: ChartOptions = {
      type: "heatmap",
      xName: "x",
      yName: "y",
      xCategories: [1, 2],
      yCategories: [10],
      series: [
        xyzSeries("z", "z", [
          [0, 0, 100],
          [1, 0, 200],
        ]),
      ],
    };

    const { container } = render(<ChartWrapper data={data} />);
    expect(container.querySelector("canvas")).toBeInTheDocument();
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("shows heatmap cell values on hover", () => {
    const data: ChartOptions = {
      type: "heatmap",
      xName: "x",
      yName: "y",
      xCategories: [1, 2],
      yCategories: [10],
      series: [
        xyzSeries("z", "z", [
          [0, 0, 100],
          [1, 0, 200],
        ]),
      ],
    };

    const { container } = render(<ChartWrapper data={data} />);
    const svg = container.querySelector("svg");
    expect(svg).toBeInTheDocument();

    Object.defineProperty(svg!, "getBoundingClientRect", {
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

    fireEvent.pointerMove(svg!, { clientX: 180, clientY: 80 });

    expect(screen.getByText("x: 2, y: 10")).toBeInTheDocument();
    expect(screen.getByText("z: 200")).toBeInTheDocument();
  });
});

function xySeries(id: string, label: string, points: [number, number][]) {
  return {
    id,
    label,
    pointCount: points.length,
    values: Float32Array.from(points.flat()),
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
    values: Float32Array.from(points.flat()),
  };
}
