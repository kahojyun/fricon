import { describe, expect, it } from "vitest";
import { renderColorScale, LIGHT_THEME } from "./d3Overlay";
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
});
