import { scaleLinear } from "d3-scale";
import { describe, expect, it } from "vitest";
import { DEFAULT_NUMERIC_LABEL_FORMAT } from "./numericLabelFormat";
import { renderAxes, renderColorScale, LIGHT_THEME } from "./d3Overlay";
import type { ChartMargin } from "./webgl";

const margin: ChartMargin = {
  top: 20,
  right: 80,
  bottom: 40,
  left: 60,
};

describe("renderColorScale", () => {
  it("defines gradient stops in ascending offset order with max color at the top", () => {
    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    document.body.appendChild(svgEl);
    Object.defineProperty(svgEl, "clientWidth", {
      configurable: true,
      value: 320,
    });
    Object.defineProperty(svgEl, "clientHeight", {
      configurable: true,
      value: 220,
    });

    renderColorScale(
      svgEl,
      ["#2c7bb6", "#abd9e9", "#ffffbf", "#fdae61", "#d7191c"],
      1,
      9,
      margin,
      LIGHT_THEME,
      DEFAULT_NUMERIC_LABEL_FORMAT,
    );

    const stops = Array.from(svgEl.getElementsByTagName("stop"));
    expect(stops).toHaveLength(5);
    expect(stops.map((stop) => stop.getAttribute("offset"))).toEqual([
      "0%",
      "25%",
      "50%",
      "75%",
      "100%",
    ]);
    expect(stops.map((stop) => stop.getAttribute("stop-color"))).toEqual([
      "#d7191c",
      "#fdae61",
      "#ffffbf",
      "#abd9e9",
      "#2c7bb6",
    ]);

    const rect = svgEl.querySelector(".color-scale rect");
    expect(rect?.getAttribute("fill")).toMatch(
      /^url\(#heatmap-color-scale-grad-/,
    );
  });

  it("removes stale heatmap legend when cartesian axes are rendered", () => {
    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    document.body.appendChild(svgEl);
    Object.defineProperty(svgEl, "clientWidth", {
      configurable: true,
      value: 320,
    });
    Object.defineProperty(svgEl, "clientHeight", {
      configurable: true,
      value: 220,
    });

    renderColorScale(
      svgEl,
      ["#2c7bb6", "#abd9e9", "#ffffbf", "#fdae61", "#d7191c"],
      1,
      9,
      margin,
      LIGHT_THEME,
      DEFAULT_NUMERIC_LABEL_FORMAT,
    );
    expect(svgEl.querySelector(".color-scale")).not.toBeNull();

    renderAxes(
      svgEl,
      scaleLinear().domain([0, 10]).range([0, 100]),
      scaleLinear().domain([0, 10]).range([100, 0]),
      "x",
      "y",
      margin,
      LIGHT_THEME,
      DEFAULT_NUMERIC_LABEL_FORMAT,
    );

    expect(svgEl.querySelector(".color-scale")).toBeNull();
    expect(svgEl.querySelector(".axes")).not.toBeNull();
  });

  it("formats cartesian axis ticks with SI prefixes", () => {
    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    document.body.appendChild(svgEl);
    Object.defineProperty(svgEl, "clientWidth", {
      configurable: true,
      value: 320,
    });
    Object.defineProperty(svgEl, "clientHeight", {
      configurable: true,
      value: 220,
    });

    renderAxes(
      svgEl,
      scaleLinear().domain([0, 2000]).range([0, 100]),
      scaleLinear().domain([0, 1]).range([100, 0]),
      "x",
      "y",
      margin,
      LIGHT_THEME,
      { mode: "si", significantDigits: 4 },
    );

    const labels = Array.from(svgEl.querySelectorAll(".axes text")).map(
      (node) => node.textContent,
    );

    expect(labels).toContain("1k");
    expect(labels).toContain("2k");
  });

  it("renders requested numeric tick values for heatmap center labels", () => {
    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    document.body.appendChild(svgEl);
    Object.defineProperty(svgEl, "clientWidth", {
      configurable: true,
      value: 320,
    });
    Object.defineProperty(svgEl, "clientHeight", {
      configurable: true,
      value: 220,
    });

    renderAxes(
      svgEl,
      scaleLinear().domain([5, 50]).range([0, 100]),
      scaleLinear().domain([110, 320]).range([100, 0]),
      "x",
      "y",
      margin,
      LIGHT_THEME,
      DEFAULT_NUMERIC_LABEL_FORMAT,
      {
        showGrid: false,
        xTickValues: [7, 19, 46],
        yTickValues: [120, 185, 310],
      },
    );

    const axisLabels = Array.from(
      svgEl.querySelectorAll(".axes .tick text"),
    ).map((node) => node.textContent);

    expect(axisLabels).toEqual(
      expect.arrayContaining(["7", "19", "46", "120", "185", "310"]),
    );
  });

  it("formats heatmap color scale labels with scientific notation", () => {
    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    document.body.appendChild(svgEl);
    Object.defineProperty(svgEl, "clientWidth", {
      configurable: true,
      value: 320,
    });
    Object.defineProperty(svgEl, "clientHeight", {
      configurable: true,
      value: 220,
    });

    renderColorScale(
      svgEl,
      ["#2c7bb6", "#abd9e9", "#ffffbf", "#fdae61", "#d7191c"],
      0.00001234,
      123400,
      margin,
      LIGHT_THEME,
      { mode: "scientific", significantDigits: 4 },
    );

    const labels = Array.from(svgEl.querySelectorAll(".color-scale text")).map(
      (node) => node.textContent,
    );

    expect(labels).toContain("1.234e+5");
    expect(labels).toContain("1.234e-5");
  });
});
