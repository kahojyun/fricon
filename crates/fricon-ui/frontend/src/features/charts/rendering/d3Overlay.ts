/**
 * d3-based SVG overlay for chart axes, grid lines, and axis labels.
 * Renders into an SVG element that sits on top of the WebGL canvas.
 */

import { axisBottom, axisLeft } from "d3-axis";
import { select } from "d3-selection";
import type { ScaleBand, ScaleLinear } from "d3-scale";
import type { ChartMargin } from "./webgl";

export type AxisScale =
  | ScaleLinear<number, number>
  | ScaleBand<string | number>;

export interface OverlayTheme {
  textColor: string;
  gridColor: string;
  surfaceColor: string;
  surfaceBorderColor: string;
}

export const LIGHT_THEME: OverlayTheme = {
  textColor: "#333",
  gridColor: "#e0e0e0",
  surfaceColor: "rgba(255, 255, 255, 0.96)",
  surfaceBorderColor: "rgba(51, 51, 51, 0.18)",
};

export const DARK_THEME: OverlayTheme = {
  textColor: "#ccc",
  gridColor: "#444",
  surfaceColor: "rgba(32, 32, 36, 0.96)",
  surfaceBorderColor: "rgba(255, 255, 255, 0.18)",
};

export function getOverlayTheme(
  resolvedTheme: string | undefined,
): OverlayTheme {
  return resolvedTheme === "dark" ? DARK_THEME : LIGHT_THEME;
}

export function renderAxes(
  svgEl: SVGSVGElement,
  xScale: ScaleLinear<number, number>,
  yScale: ScaleLinear<number, number>,
  xName: string,
  yName: string,
  margin: ChartMargin,
  theme: OverlayTheme,
): void {
  const svg = select(svgEl);
  const width = svgEl.clientWidth;
  const height = svgEl.clientHeight;
  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;

  svg.selectAll(".axes").remove();

  const g = svg
    .append("g")
    .attr("class", "axes")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  // X axis
  const xAxis = axisBottom(xScale.range([0, chartWidth])).ticks(
    Math.max(2, Math.floor(chartWidth / 80)),
  );
  g.append("g")
    .attr("transform", `translate(0,${chartHeight})`)
    .call(xAxis)
    .call((g) => {
      g.selectAll("line, path").attr("stroke", theme.gridColor);
      g.selectAll("text")
        .attr("fill", theme.textColor)
        .style("font-size", "11px");
    });

  // X grid lines
  g.append("g")
    .attr("class", "grid")
    .call(
      axisBottom(xScale.range([0, chartWidth]))
        .ticks(Math.max(2, Math.floor(chartWidth / 80)))
        .tickSize(-chartHeight)
        .tickFormat(() => ""),
    )
    .attr("transform", `translate(0,${chartHeight})`)
    .call((g) => {
      g.selectAll("line")
        .attr("stroke", theme.gridColor)
        .attr("stroke-opacity", 0.3);
      g.select(".domain").remove();
    });

  // Y axis
  const yAxis = axisLeft(yScale.range([chartHeight, 0])).ticks(
    Math.max(2, Math.floor(chartHeight / 50)),
  );
  g.append("g")
    .call(yAxis)
    .call((g) => {
      g.selectAll("line, path").attr("stroke", theme.gridColor);
      g.selectAll("text")
        .attr("fill", theme.textColor)
        .style("font-size", "11px");
    });

  // Y grid lines
  g.append("g")
    .attr("class", "grid")
    .call(
      axisLeft(yScale.range([chartHeight, 0]))
        .ticks(Math.max(2, Math.floor(chartHeight / 50)))
        .tickSize(-chartWidth)
        .tickFormat(() => ""),
    )
    .call((g) => {
      g.selectAll("line")
        .attr("stroke", theme.gridColor)
        .attr("stroke-opacity", 0.3);
      g.select(".domain").remove();
    });

  // X axis label
  if (xName) {
    g.append("text")
      .attr("x", chartWidth / 2)
      .attr("y", chartHeight + margin.bottom - 6)
      .attr("text-anchor", "middle")
      .attr("fill", theme.textColor)
      .style("font-size", "12px")
      .text(xName);
  }

  // Y axis label
  if (yName) {
    g.append("text")
      .attr("x", -chartHeight / 2)
      .attr("y", -margin.left + 14)
      .attr("transform", "rotate(-90)")
      .attr("text-anchor", "middle")
      .attr("fill", theme.textColor)
      .style("font-size", "12px")
      .text(yName);
  }
}

export function renderCategoryAxes(
  svgEl: SVGSVGElement,
  xScale: ScaleBand<string | number>,
  yScale: ScaleBand<string | number>,
  xName: string,
  yName: string,
  margin: ChartMargin,
  theme: OverlayTheme,
): void {
  const svg = select(svgEl);
  const width = svgEl.clientWidth;
  const height = svgEl.clientHeight;
  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;

  svg.selectAll(".axes").remove();

  const g = svg
    .append("g")
    .attr("class", "axes")
    .attr("transform", `translate(${margin.left},${margin.top})`);

  // X axis
  const xTicks = xScale.domain();
  const maxXTicks = Math.max(2, Math.floor(chartWidth / 40));
  const xTickValues =
    xTicks.length > maxXTicks
      ? xTicks.filter((_, i) => i % Math.ceil(xTicks.length / maxXTicks) === 0)
      : xTicks;

  const xAxis = axisBottom(xScale.range([0, chartWidth])).tickValues(
    xTickValues,
  );
  g.append("g")
    .attr("transform", `translate(0,${chartHeight})`)
    .call(xAxis)
    .call((g) => {
      g.selectAll("line, path").attr("stroke", theme.gridColor);
      g.selectAll("text")
        .attr("fill", theme.textColor)
        .style("font-size", "10px");
    });

  // Y axis
  const yTicks = yScale.domain();
  const maxYTicks = Math.max(2, Math.floor(chartHeight / 20));
  const yTickValues =
    yTicks.length > maxYTicks
      ? yTicks.filter((_, i) => i % Math.ceil(yTicks.length / maxYTicks) === 0)
      : yTicks;

  const yAxis = axisLeft(yScale.range([chartHeight, 0])).tickValues(
    yTickValues,
  );
  g.append("g")
    .call(yAxis)
    .call((g) => {
      g.selectAll("line, path").attr("stroke", theme.gridColor);
      g.selectAll("text")
        .attr("fill", theme.textColor)
        .style("font-size", "10px");
    });

  // Axis labels
  if (xName) {
    g.append("text")
      .attr("x", chartWidth / 2)
      .attr("y", chartHeight + margin.bottom - 6)
      .attr("text-anchor", "middle")
      .attr("fill", theme.textColor)
      .style("font-size", "12px")
      .text(xName);
  }
  if (yName) {
    g.append("text")
      .attr("x", -chartHeight / 2)
      .attr("y", -margin.left + 14)
      .attr("transform", "rotate(-90)")
      .attr("text-anchor", "middle")
      .attr("fill", theme.textColor)
      .style("font-size", "12px")
      .text(yName);
  }
}
