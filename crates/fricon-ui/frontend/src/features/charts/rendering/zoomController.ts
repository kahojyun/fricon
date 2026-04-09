/**
 * Zoom/pan controller on an SVG overlay.
 * Supports:
 * - wheel zoom centered on cursor
 * - left-drag pan
 * - right-drag anisotropic scale (x by horizontal drag, y by vertical drag)
 * - double-click to reset view
 * - shift+left-drag box zoom (brush zoom)
 */

import type { ChartMargin } from "./webgl";

export interface ZoomState {
  scaleX: number;
  scaleY: number;
  translateX: number;
  translateY: number;
}

export const IDENTITY_ZOOM: ZoomState = {
  scaleX: 1,
  scaleY: 1,
  translateX: 0,
  translateY: 0,
};

export interface BrushRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface ZoomController {
  /** Detach zoom behavior and clean up listeners. */
  destroy: () => void;
  /** Reset zoom to the current default view. */
  reset: (nextDefaultState?: ZoomState) => void;
  /** Update translate extents after a resize. */
  updateExtents: (
    chartWidth: number,
    chartHeight: number,
    margin: Pick<ChartMargin, "left" | "top">,
  ) => void;
  /** Update the default view used by double-click reset. */
  setDefaultState: (state: ZoomState) => void;
  /** Synchronize controller state with an externally managed zoom state. */
  syncState: (state: ZoomState) => void;
}

type TransformReason = "gesture" | "reset";

type ChartOrigin = Pick<ChartMargin, "left" | "top">;
interface ChartPoint {
  x: number;
  y: number;
}

interface PanGesture {
  kind: "pan";
  pointerId: number;
  startPointer: ChartPoint;
  startState: ZoomState;
}

interface ScaleGesture {
  kind: "scale";
  pointerId: number;
  startPointer: ChartPoint;
  anchor: ChartPoint;
  startState: ZoomState;
}

interface BrushGesture {
  kind: "brush";
  pointerId: number;
  startPointer: ChartPoint;
  startState: ZoomState;
}

type ActiveGesture = PanGesture | ScaleGesture | BrushGesture;

const WHEEL_ZOOM_SENSITIVITY = 0.002;
const DRAG_ZOOM_SENSITIVITY = 0.01;
const MIN_BRUSH_SIZE = 10;

export function scaleZoomStateAroundPoint(
  state: ZoomState,
  anchor: ChartPoint,
  scaleFactorX: number,
  scaleFactorY: number,
): ZoomState {
  return {
    scaleX: state.scaleX * scaleFactorX,
    scaleY: state.scaleY * scaleFactorY,
    translateX: scaleFactorX * (state.translateX - anchor.x) + anchor.x,
    translateY: scaleFactorY * (state.translateY - anchor.y) + anchor.y,
  };
}

export function attachZoom(
  svgElement: SVGSVGElement,
  chartWidth: number,
  chartHeight: number,
  margin: ChartOrigin,
  onTransformChange: (state: ZoomState, reason: TransformReason) => void,
  onBrushChange?: (rect: BrushRect | null) => void,
  onResetStart?: (targetState: ZoomState) => void,
): ZoomController {
  let currentWidth = chartWidth;
  let currentHeight = chartHeight;
  let currentMargin = margin;
  let currentState = IDENTITY_ZOOM;
  let defaultState = IDENTITY_ZOOM;
  let activeGesture: ActiveGesture | null = null;
  let resetAnimationFrame = 0;

  function cancelResetAnimation() {
    if (resetAnimationFrame !== 0) {
      cancelAnimationFrame(resetAnimationFrame);
      resetAnimationFrame = 0;
    }
  }

  function emit(state: ZoomState, reason: TransformReason) {
    currentState = state;
    onTransformChange(state, reason);
  }

  function toChartPoint(
    clientX: number,
    clientY: number,
    options?: { requireInsideChart?: boolean },
  ): ChartPoint | null {
    const rect = svgElement.getBoundingClientRect();
    const x = clientX - rect.left - currentMargin.left;
    const y = clientY - rect.top - currentMargin.top;

    if (
      options?.requireInsideChart !== false &&
      (x < 0 || x > currentWidth || y < 0 || y > currentHeight)
    ) {
      return null;
    }

    return { x, y };
  }

  function clampScale(value: number, fallback: number): number {
    if (!isFinite(value) || value <= 0) {
      return fallback;
    }
    return value;
  }

  function scaleFromStart(
    startState: ZoomState,
    anchor: ChartPoint,
    nextScaleX: number,
    nextScaleY: number,
  ): ZoomState {
    return scaleZoomStateAroundPoint(
      startState,
      anchor,
      nextScaleX / startState.scaleX,
      nextScaleY / startState.scaleY,
    );
  }

  function handlePointerDown(event: PointerEvent) {
    const chartPoint = toChartPoint(event.clientX, event.clientY, {
      requireInsideChart: true,
    });
    if (!chartPoint) return;

    cancelResetAnimation();

    if (event.button === 0 && event.shiftKey) {
      activeGesture = {
        kind: "brush",
        pointerId: event.pointerId,
        startPointer: chartPoint,
        startState: currentState,
      };
    } else if (event.button === 0) {
      activeGesture = {
        kind: "pan",
        pointerId: event.pointerId,
        startPointer: chartPoint,
        startState: currentState,
      };
    } else if (event.button === 2) {
      activeGesture = {
        kind: "scale",
        pointerId: event.pointerId,
        startPointer: chartPoint,
        anchor: chartPoint,
        startState: currentState,
      };
    } else {
      return;
    }

    event.preventDefault();
    svgElement.setPointerCapture(event.pointerId);
  }

  function handlePointerMove(event: PointerEvent) {
    if (event.pointerId !== activeGesture?.pointerId) return;

    const chartPoint = toChartPoint(event.clientX, event.clientY, {
      requireInsideChart: false,
    });
    if (!chartPoint) return;

    event.preventDefault();

    if (activeGesture.kind === "pan") {
      emit(
        {
          ...activeGesture.startState,
          translateX:
            activeGesture.startState.translateX +
            (chartPoint.x - activeGesture.startPointer.x),
          translateY:
            activeGesture.startState.translateY +
            (chartPoint.y - activeGesture.startPointer.y),
        },
        "gesture",
      );
      return;
    }

    if (activeGesture.kind === "brush") {
      const x = Math.min(activeGesture.startPointer.x, chartPoint.x);
      const y = Math.min(activeGesture.startPointer.y, chartPoint.y);
      const width = Math.abs(chartPoint.x - activeGesture.startPointer.x);
      const height = Math.abs(chartPoint.y - activeGesture.startPointer.y);
      onBrushChange?.({ x, y, width, height });
      return;
    }

    const dx = chartPoint.x - activeGesture.startPointer.x;
    const dy = chartPoint.y - activeGesture.startPointer.y;
    const scaleX = clampScale(
      activeGesture.startState.scaleX * Math.exp(dx * DRAG_ZOOM_SENSITIVITY),
      activeGesture.startState.scaleX,
    );
    const scaleY = clampScale(
      activeGesture.startState.scaleY * Math.exp(-dy * DRAG_ZOOM_SENSITIVITY),
      activeGesture.startState.scaleY,
    );

    emit(
      scaleFromStart(
        activeGesture.startState,
        activeGesture.anchor,
        scaleX,
        scaleY,
      ),
      "gesture",
    );
  }

  function clearGesture(pointerId: number) {
    if (activeGesture?.pointerId !== pointerId) return;
    activeGesture = null;
    if (svgElement.hasPointerCapture(pointerId)) {
      svgElement.releasePointerCapture(pointerId);
    }
  }

  function handlePointerUp(event: PointerEvent) {
    if (activeGesture?.pointerId === event.pointerId) {
      if (activeGesture.kind === "brush") {
        finalizeBrush(event);
      }
    }
    clearGesture(event.pointerId);
  }

  function finalizeBrush(event: PointerEvent) {
    if (activeGesture?.kind !== "brush") return;

    const chartPoint = toChartPoint(event.clientX, event.clientY, {
      requireInsideChart: false,
    });
    if (!chartPoint) {
      onBrushChange?.(null);
      return;
    }

    const x1 = Math.min(activeGesture.startPointer.x, chartPoint.x);
    const y1 = Math.min(activeGesture.startPointer.y, chartPoint.y);
    const x2 = Math.max(activeGesture.startPointer.x, chartPoint.x);
    const y2 = Math.max(activeGesture.startPointer.y, chartPoint.y);
    const brushW = x2 - x1;
    const brushH = y2 - y1;

    onBrushChange?.(null);

    if (brushW < MIN_BRUSH_SIZE || brushH < MIN_BRUSH_SIZE) return;

    const { startState } = activeGesture;
    // Convert brush corners to unzoomed chart coordinates
    const ux1 = (x1 - startState.translateX) / startState.scaleX;
    const ux2 = (x2 - startState.translateX) / startState.scaleX;
    const uy1 = (y1 - startState.translateY) / startState.scaleY;
    const uy2 = (y2 - startState.translateY) / startState.scaleY;

    const newScaleX = clampScale(currentWidth / (ux2 - ux1), startState.scaleX);
    const newScaleY = clampScale(
      currentHeight / (uy2 - uy1),
      startState.scaleY,
    );
    const newTranslateX = -ux1 * newScaleX;
    const newTranslateY = -uy1 * newScaleY;

    emit(
      {
        scaleX: newScaleX,
        scaleY: newScaleY,
        translateX: newTranslateX,
        translateY: newTranslateY,
      },
      "gesture",
    );
  }

  function handlePointerCancel(event: PointerEvent) {
    if (activeGesture?.kind === "brush") {
      onBrushChange?.(null);
    }
    clearGesture(event.pointerId);
  }

  function handleDblClick(event: MouseEvent) {
    const chartPoint = toChartPoint(event.clientX, event.clientY, {
      requireInsideChart: true,
    });
    if (!chartPoint) return;
    event.preventDefault();
    performReset();
  }

  function handleWheel(event: WheelEvent) {
    const chartPoint = toChartPoint(event.clientX, event.clientY, {
      requireInsideChart: true,
    });
    if (!chartPoint) return;

    cancelResetAnimation();
    event.preventDefault();

    const factor = Math.exp(-event.deltaY * WHEEL_ZOOM_SENSITIVITY);
    const nextScaleX = clampScale(
      currentState.scaleX * factor,
      currentState.scaleX,
    );
    const nextScaleY = clampScale(
      currentState.scaleY * factor,
      currentState.scaleY,
    );

    emit(
      scaleFromStart(currentState, chartPoint, nextScaleX, nextScaleY),
      "gesture",
    );
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
  }

  svgElement.addEventListener("pointerdown", handlePointerDown);
  svgElement.addEventListener("pointermove", handlePointerMove);
  svgElement.addEventListener("pointerup", handlePointerUp);
  svgElement.addEventListener("pointercancel", handlePointerCancel);
  svgElement.addEventListener("wheel", handleWheel, { passive: false });
  svgElement.addEventListener("contextmenu", handleContextMenu);
  svgElement.addEventListener("dblclick", handleDblClick);

  function performReset(targetState = defaultState) {
    cancelResetAnimation();
    onResetStart?.(targetState);
    const startState = currentState;
    const startTime = performance.now();
    const durationMs = 200;

    function tick(now: number) {
      const progress = Math.min((now - startTime) / durationMs, 1);
      const eased = 1 - (1 - progress) * (1 - progress);
      emit(
        {
          scaleX:
            startState.scaleX +
            (targetState.scaleX - startState.scaleX) * eased,
          scaleY:
            startState.scaleY +
            (targetState.scaleY - startState.scaleY) * eased,
          translateX:
            startState.translateX +
            (targetState.translateX - startState.translateX) * eased,
          translateY:
            startState.translateY +
            (targetState.translateY - startState.translateY) * eased,
        },
        "reset",
      );

      if (progress < 1) {
        resetAnimationFrame = requestAnimationFrame(tick);
      } else {
        resetAnimationFrame = 0;
      }
    }

    resetAnimationFrame = requestAnimationFrame(tick);
  }

  return {
    updateExtents(newWidth: number, newHeight: number, newMargin: ChartOrigin) {
      currentWidth = newWidth;
      currentHeight = newHeight;
      currentMargin = newMargin;
    },
    setDefaultState(state: ZoomState) {
      defaultState = state;
    },
    syncState(state: ZoomState) {
      cancelResetAnimation();
      currentState = state;
    },
    destroy() {
      cancelResetAnimation();
      svgElement.removeEventListener("pointerdown", handlePointerDown);
      svgElement.removeEventListener("pointermove", handlePointerMove);
      svgElement.removeEventListener("pointerup", handlePointerUp);
      svgElement.removeEventListener("pointercancel", handlePointerCancel);
      svgElement.removeEventListener("wheel", handleWheel);
      svgElement.removeEventListener("contextmenu", handleContextMenu);
      svgElement.removeEventListener("dblclick", handleDblClick);
    },
    reset(nextDefaultState) {
      if (nextDefaultState) {
        defaultState = nextDefaultState;
      }
      performReset(nextDefaultState);
    },
  };
}
