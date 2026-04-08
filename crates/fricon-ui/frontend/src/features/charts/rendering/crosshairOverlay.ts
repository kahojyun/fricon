/**
 * SVG crosshair overlay — vertical and horizontal dashed lines that follow
 * the cursor, with X/Y value labels at the axis edges.
 */

import { select, type Selection } from "d3-selection";
import type { ChartMargin } from "./webgl";
import type { ZoomState } from "./zoomController";
import type { OverlayTheme } from "./d3Overlay";
import { invertZoomedLinearRange } from "./mathUtils";

export interface CrosshairChartState {
  margin: ChartMargin;
  zoomState: ZoomState;
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
  theme: OverlayTheme;
}

export interface CrosshairController {
  destroy: () => void;
}

const LABEL_PADDING_X = 6;
const LABEL_PADDING_Y = 3;
const LABEL_RADIUS = 4;

function fmt(n: number): string {
  return Number.isInteger(n) ? String(n) : n.toPrecision(6);
}

export function attachCrosshair(
  svgElement: SVGSVGElement,
  getChartState: () => CrosshairChartState | null,
): CrosshairController {
  const svg = select(svgElement);
  const g = svg.append("g").attr("class", "crosshair").style("display", "none");

  const vLine = g
    .append("line")
    .attr("stroke-dasharray", "4,3")
    .attr("stroke-width", 1)
    .style("pointer-events", "none");

  const hLine = g
    .append("line")
    .attr("stroke-dasharray", "4,3")
    .attr("stroke-width", 1)
    .style("pointer-events", "none");

  const xLabel = g.append("g").style("pointer-events", "none");

  const xLabelBg = xLabel
    .append("rect")
    .attr("rx", LABEL_RADIUS)
    .attr("ry", LABEL_RADIUS);

  const xLabelText = xLabel
    .append("text")
    .attr("text-anchor", "middle")
    .attr("dominant-baseline", "middle")
    .style("font-size", "10px")
    .style("pointer-events", "none");

  const yLabel = g.append("g").style("pointer-events", "none");

  const yLabelBg = yLabel
    .append("rect")
    .attr("rx", LABEL_RADIUS)
    .attr("ry", LABEL_RADIUS);

  const yLabelText = yLabel
    .append("text")
    .attr("text-anchor", "end")
    .attr("dominant-baseline", "middle")
    .style("font-size", "10px")
    .style("pointer-events", "none");

  function syncLabelBackground(
    background: Selection<SVGRectElement, unknown, null, undefined>,
    text: Selection<SVGTextElement, unknown, null, undefined>,
    theme: OverlayTheme,
  ) {
    const bbox = text.node()?.getBBox();
    if (!bbox) return;

    background
      .attr("x", bbox.x - LABEL_PADDING_X)
      .attr("y", bbox.y - LABEL_PADDING_Y)
      .attr("width", bbox.width + LABEL_PADDING_X * 2)
      .attr("height", bbox.height + LABEL_PADDING_Y * 2)
      .attr("fill", theme.surfaceColor)
      .attr("stroke", theme.surfaceBorderColor)
      .attr("stroke-width", 1);
  }

  function handlePointerMove(event: PointerEvent) {
    const state = getChartState();
    if (!state) {
      g.style("display", "none");
      return;
    }

    const { margin, zoomState, xMin, xMax, yMin, yMax, theme } = state;
    const rect = svgElement.getBoundingClientRect();
    const mouseX = event.clientX - rect.left;
    const mouseY = event.clientY - rect.top;

    const chartWidth = svgElement.clientWidth - margin.left - margin.right;
    const chartHeight = svgElement.clientHeight - margin.top - margin.bottom;

    const chartX = mouseX - margin.left;
    const chartY = mouseY - margin.top;

    if (
      chartX < 0 ||
      chartX > chartWidth ||
      chartY < 0 ||
      chartY > chartHeight
    ) {
      g.style("display", "none");
      return;
    }

    const dataX = invertZoomedLinearRange(
      chartX,
      zoomState.translateX,
      zoomState.scaleX,
      xMin,
      xMax,
      0,
      chartWidth,
    );
    const dataY = invertZoomedLinearRange(
      chartY,
      zoomState.translateY,
      zoomState.scaleY,
      yMin,
      yMax,
      chartHeight,
      0,
    );

    g.raise();
    g.style("display", null);

    vLine
      .attr("x1", mouseX)
      .attr("y1", margin.top)
      .attr("x2", mouseX)
      .attr("y2", margin.top + chartHeight)
      .attr("stroke", theme.textColor)
      .attr("stroke-opacity", 0.5);

    hLine
      .attr("x1", margin.left)
      .attr("y1", mouseY)
      .attr("x2", margin.left + chartWidth)
      .attr("y2", mouseY)
      .attr("stroke", theme.textColor)
      .attr("stroke-opacity", 0.5);

    xLabelText
      .attr("x", mouseX)
      .attr("y", margin.top + chartHeight + 13)
      .attr("fill", theme.textColor)
      .text(fmt(dataX));
    syncLabelBackground(xLabelBg, xLabelText, theme);

    yLabelText
      .attr("x", margin.left - 8)
      .attr("y", mouseY)
      .attr("fill", theme.textColor)
      .text(fmt(dataY));
    syncLabelBackground(yLabelBg, yLabelText, theme);
  }

  function handlePointerLeave() {
    g.style("display", "none");
  }

  svgElement.addEventListener("pointermove", handlePointerMove);
  svgElement.addEventListener("pointerleave", handlePointerLeave);

  return {
    destroy() {
      svgElement.removeEventListener("pointermove", handlePointerMove);
      svgElement.removeEventListener("pointerleave", handlePointerLeave);
      g.remove();
    },
  };
}
