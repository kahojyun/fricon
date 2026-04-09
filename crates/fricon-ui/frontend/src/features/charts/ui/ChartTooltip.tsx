/**
 * Chart tooltip — HTML overlay positioned by pointer events.
 * On mousemove, projects pointer coordinates to data space and
 * finds the nearest data point(s), displaying formatted values.
 */

import { useEffect, useState } from "react";
import type { ChartOptions } from "@/shared/lib/chartTypes";
import type { ChartInteractionState } from "../hooks/useWebGLChart";
import { getTooltipLines } from "./tooltipLines";

interface TooltipState {
  visible: boolean;
  x: number;
  y: number;
  lines: string[];
}

interface ChartTooltipProps {
  data?: ChartOptions;
  /** Ref to the SVG overlay element – pointer events are tracked here for chart interactions. */
  svgRef: React.RefObject<SVGSVGElement | null>;
  getInteractionState: () => ChartInteractionState | null;
}

export function ChartTooltip({
  data,
  svgRef,
  getInteractionState,
}: ChartTooltipProps) {
  const [tooltip, setTooltip] = useState<TooltipState>({
    visible: false,
    x: 0,
    y: 0,
    lines: [],
  });

  useEffect(() => {
    const svg = svgRef.current;
    if (!svg) return;
    const svgElement = svg;

    function handlePointerMove(e: PointerEvent) {
      if (!data) {
        setTooltip((prev) =>
          prev.visible ? { ...prev, visible: false } : prev,
        );
        return;
      }

      const interactionState = getInteractionState();
      if (!interactionState) {
        setTooltip((prev) =>
          prev.visible ? { ...prev, visible: false } : prev,
        );
        return;
      }

      const rect = svgElement.getBoundingClientRect();
      const mouseX = e.clientX - rect.left;
      const mouseY = e.clientY - rect.top;

      const { margin } = interactionState;
      const chartWidth = rect.width - margin.left - margin.right;
      const chartHeight = rect.height - margin.top - margin.bottom;

      const chartX = mouseX - margin.left;
      const chartY = mouseY - margin.top;
      if (
        chartX < 0 ||
        chartX > chartWidth ||
        chartY < 0 ||
        chartY > chartHeight
      ) {
        setTooltip((prev) =>
          prev.visible ? { ...prev, visible: false } : prev,
        );
        return;
      }

      const lines = getTooltipLines(
        data,
        interactionState,
        chartX,
        chartY,
        chartWidth,
        chartHeight,
      );
      if (lines.length === 0) {
        setTooltip((prev) =>
          prev.visible ? { ...prev, visible: false } : prev,
        );
        return;
      }

      setTooltip({ visible: true, x: mouseX, y: mouseY, lines });
    }

    function handlePointerLeave() {
      setTooltip((prev) => (prev.visible ? { ...prev, visible: false } : prev));
    }

    svgElement.addEventListener("pointermove", handlePointerMove);
    svgElement.addEventListener("pointerleave", handlePointerLeave);
    return () => {
      svgElement.removeEventListener("pointermove", handlePointerMove);
      svgElement.removeEventListener("pointerleave", handlePointerLeave);
    };
  }, [data, getInteractionState, svgRef]);

  return (
    <div className="pointer-events-none absolute inset-0">
      {tooltip.visible ? (
        <div
          className="pointer-events-none absolute z-50 rounded border border-border bg-popover px-2 py-1 text-xs text-popover-foreground shadow-md"
          style={{
            left: tooltip.x + 12,
            top: tooltip.y - 12,
            maxWidth: 280,
          }}
        >
          {tooltip.lines.map((line) => (
            <div key={line}>{line}</div>
          ))}
        </div>
      ) : null}
    </div>
  );
}
