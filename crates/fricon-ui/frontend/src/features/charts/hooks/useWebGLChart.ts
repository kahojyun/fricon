/**
 * React hook that manages the full WebGL2 + d3 chart lifecycle.
 *
 * - Creates and tears down the WebGL2 context
 * - Delegates to the correct renderer based on ChartOptions.type
 * - Manages d3 SVG overlay for axes
 * - Handles zoom/pan via d3-zoom
 * - Handles resize via ResizeObserver
 */

import { useEffect, useRef, useCallback } from "react";
import { scaleLinear } from "d3-scale";
import { select } from "d3-selection";
import {
  xyDrawStyleIncludesLine,
  xyDrawStyleIncludesPoints,
  type ChartOptions,
  type NumericLabelFormatOptions,
} from "@/shared/lib/chartTypes";
import { resolveXYYAxisLabel } from "../model/seriesLabeling";
import {
  resizeCanvas,
  dataToClipMatrix,
  mul3x3,
  zoomToClipMatrix,
  chartAreaToViewport,
  DEFAULT_MARGIN,
  HEATMAP_MARGIN,
  type ChartMargin,
} from "../rendering/webgl";
import {
  renderAxes,
  renderColorScale,
  getOverlayTheme,
} from "../rendering/d3Overlay";
import { DEFAULT_NUMERIC_LABEL_FORMAT } from "../rendering/numericLabelFormat";
import {
  attachZoom,
  IDENTITY_ZOOM,
  type BrushRect,
  type ZoomController,
  type ZoomState,
} from "../rendering/zoomController";
import {
  attachCrosshair,
  type CrosshairController,
} from "../rendering/crosshairOverlay";
import {
  createLineRenderState,
  syncLineRenderState,
  drawLines,
  destroyLineRenderState,
  lineDataBounds,
  type LineRenderState,
} from "../rendering/lineRenderer";
import {
  createScatterRenderState,
  syncScatterRenderState,
  drawScatter,
  destroyScatterRenderState,
  type ScatterRenderState,
} from "../rendering/scatterRenderer";
import {
  createHeatmapRenderState,
  syncHeatmapRenderState,
  drawHeatmap,
  destroyHeatmapRenderState,
  COLOR_RAMP,
  type HeatmapRenderState,
} from "../rendering/heatmapRenderer";
import {
  deriveHeatmapLayout,
  getHeatmapXTickValues,
  type HeatmapGeometry,
} from "../rendering/heatmapGeometry";

type RenderState =
  | {
      type: "xy";
      lineState: LineRenderState | null;
      scatterState: ScatterRenderState | null;
    }
  | { type: "heatmap"; state: HeatmapRenderState };

interface RenderStateCache {
  line: LineRenderState | null;
  scatter: ScatterRenderState | null;
  heatmap: HeatmapRenderState | null;
}

interface WebGLChartRefs {
  gl: WebGL2RenderingContext | null;
  renderStateCache: RenderStateCache;
  renderState: RenderState | null;
  zoomController: ZoomController | null;
  crosshairController: CrosshairController | null;
  zoomState: ZoomState;
  defaultZoomState: ZoomState;
  followsDefaultView: boolean;
  viewBounds: NumericBounds | null;
  data: ChartOptions | undefined;
  interactionKey: string | null;
  lastResolvedInteractionKey: string | null;
  liveMode: boolean;
  theme: string | undefined;
  numericLabelFormat: NumericLabelFormatOptions;
  animFrameId: number;
  needsRender: boolean;
  contextLost: boolean;
}

export interface UseWebGLChartOptions {
  data?: ChartOptions;
  interactionKey?: string | null;
  liveMode?: boolean;
  theme?: string;
  numericLabelFormat?: NumericLabelFormatOptions;
}

export interface UseWebGLChartReturn {
  canvasRef: React.RefObject<HTMLCanvasElement | null>;
  svgRef: React.RefObject<SVGSVGElement | null>;
  /** For tooltip: get current chart interaction state. */
  getInteractionState: () => ChartInteractionState | null;
}

export type ChartInteractionState =
  | {
      type: "xy";
      xMin: number;
      xMax: number;
      yMin: number;
      yMax: number;
      margin: ChartMargin;
      zoomState: ZoomState;
    }
  | {
      type: "heatmap";
      xMin: number;
      xMax: number;
      yMin: number;
      yMax: number;
      margin: ChartMargin;
      zoomState: ZoomState;
      geometry: HeatmapGeometry;
    };

export function useWebGLChart({
  data,
  interactionKey,
  liveMode = false,
  theme,
  numericLabelFormat = DEFAULT_NUMERIC_LABEL_FORMAT,
}: UseWebGLChartOptions): UseWebGLChartReturn {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const svgRef = useRef<SVGSVGElement | null>(null);
  const chartType = data?.type;
  const chartRef = useRef<WebGLChartRefs>({
    gl: null,
    renderStateCache: createRenderStateCache(),
    renderState: null,
    zoomController: null,
    crosshairController: null,
    zoomState: IDENTITY_ZOOM,
    defaultZoomState: IDENTITY_ZOOM,
    followsDefaultView: true,
    viewBounds: null,
    data: undefined,
    interactionKey: null,
    lastResolvedInteractionKey: null,
    liveMode: false,
    theme: undefined,
    numericLabelFormat,
    animFrameId: 0,
    needsRender: true,
    contextLost: false,
  });

  // Keep mutable refs in sync with props (via effects for React Compiler compat)
  useEffect(() => {
    chartRef.current.data = data;
  }, [data]);

  useEffect(() => {
    chartRef.current.liveMode = liveMode;
  }, [liveMode]);

  useEffect(() => {
    chartRef.current.interactionKey = interactionKey ?? null;
  }, [interactionKey]);

  useEffect(() => {
    chartRef.current.theme = theme;
  }, [theme]);

  useEffect(() => {
    chartRef.current.numericLabelFormat = numericLabelFormat;
  }, [numericLabelFormat]);

  // Main render function
  const render = useCallback(() => {
    const r = chartRef.current;
    const gl = r.gl;
    const canvas = canvasRef.current;
    const svgEl = svgRef.current;
    if (!gl || !canvas || !svgEl || r.contextLost) return;

    resizeCanvas(canvas);

    const currentData = r.data;
    const currentLive = r.liveMode;
    const margin =
      currentData?.type === "heatmap" ? HEATMAP_MARGIN : DEFAULT_MARGIN;

    // Clear entire canvas with theme background first
    const isDark = r.theme === "dark";
    if (isDark) {
      gl.clearColor(0.09, 0.09, 0.11, 1);
    } else {
      gl.clearColor(1, 1, 1, 1);
    }
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clear(gl.COLOR_BUFFER_BIT);

    // Set viewport and scissor to chart area
    const viewport = chartAreaToViewport(canvas, margin);
    gl.viewport(viewport.x, viewport.y, viewport.width, viewport.height);
    gl.enable(gl.SCISSOR_TEST);
    gl.scissor(viewport.x, viewport.y, viewport.width, viewport.height);

    if (!currentData || !r.renderState) {
      // Clear stale SVG axes when there is no data
      select(svgEl).selectAll("*").remove();
      gl.disable(gl.SCISSOR_TEST);
      return;
    }

    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

    const rs = r.renderState;

    if (rs.type === "xy" && currentData.type === "xy") {
      const bounds = r.viewBounds ?? lineDataBounds(currentData.series);
      const { finalMatrix, zoomedXScale, zoomedYScale, overlayTheme } =
        buildZoomedAxes(canvas, margin, bounds, r.zoomState, r.theme);

      if (rs.lineState) {
        drawLines(gl, rs.lineState, finalMatrix, currentData, currentLive);
      }
      if (rs.scatterState) {
        drawScatter(gl, rs.scatterState, finalMatrix, currentData, currentLive);
      }

      renderAxes(
        svgEl,
        zoomedXScale,
        zoomedYScale,
        currentData.xName,
        resolveXYYAxisLabel(currentData),
        margin,
        overlayTheme,
        r.numericLabelFormat,
      );
    } else if (rs.type === "heatmap" && currentData.type === "heatmap") {
      const bounds = r.viewBounds ?? rs.state.bounds;
      const { finalMatrix, zoomedXScale, zoomedYScale, overlayTheme } =
        buildZoomedAxes(canvas, margin, bounds, r.zoomState, r.theme);

      drawHeatmap(gl, rs.state, finalMatrix);

      renderAxes(
        svgEl,
        zoomedXScale,
        zoomedYScale,
        currentData.xName,
        currentData.yName,
        margin,
        overlayTheme,
        r.numericLabelFormat,
        {
          showGrid: false,
          xTickValues: getHeatmapXTickValues(
            rs.state.centers,
            rs.state.xTopology,
          ),
          yTickValues:
            rs.state.centers.yValues.length <= 10
              ? rs.state.centers.yValues
              : undefined,
        },
      );
      renderColorScale(
        svgEl,
        COLOR_RAMP,
        rs.state.valueMin,
        rs.state.valueMax,
        margin,
        overlayTheme,
        r.numericLabelFormat,
      );
    }

    gl.disable(gl.SCISSOR_TEST);
    gl.disable(gl.BLEND);
  }, []);

  const scheduleRender = useCallback(() => {
    const currentRefs = chartRef.current;
    currentRefs.needsRender = true;

    function flushRender() {
      currentRefs.animFrameId = 0;
      if (!currentRefs.needsRender) return;

      currentRefs.needsRender = false;
      render();

      if (currentRefs.needsRender) {
        currentRefs.animFrameId = requestAnimationFrame(flushRender);
      }
    }

    if (currentRefs.animFrameId !== 0) return;

    currentRefs.animFrameId = requestAnimationFrame(flushRender);
  }, [render]);

  // Initialize WebGL2 context
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    function initContext() {
      const gl = canvas!.getContext("webgl2", {
        antialias: true,
        alpha: false,
        premultipliedAlpha: false,
      });
      if (!gl) {
        console.error("WebGL2 not supported");
        return;
      }
      const currentRefs = chartRef.current;
      currentRefs.gl = gl;
      currentRefs.contextLost = false;
      scheduleRender();
    }

    initContext();

    // Handle context loss: prevent default to allow restore, pause rendering
    function handleContextLost(e: Event) {
      e.preventDefault();
      const currentRefs = chartRef.current;
      currentRefs.contextLost = true;
      // GPU resources are implicitly destroyed on context loss.
      currentRefs.renderState = null;
      currentRefs.renderStateCache = createRenderStateCache();
      currentRefs.gl = null;
    }

    // Handle context restore: re-acquire GL and rebuild render state
    function handleContextRestored() {
      initContext();
      // Re-create render state for current data
      const currentRefs = chartRef.current;
      const gl = currentRefs.gl;
      const currentData = currentRefs.data;
      if (!gl || !currentData) return;

      currentRefs.renderState = syncRenderState(gl, currentRefs, currentData);
      scheduleRender();
    }

    const effectRefs = chartRef.current;

    canvas.addEventListener("webglcontextlost", handleContextLost);
    canvas.addEventListener("webglcontextrestored", handleContextRestored);

    return () => {
      canvas.removeEventListener("webglcontextlost", handleContextLost);
      canvas.removeEventListener("webglcontextrestored", handleContextRestored);
      cancelAnimationFrame(effectRefs.animFrameId);
      effectRefs.animFrameId = 0;
      if (effectRefs.gl) {
        destroyRenderStateCache(effectRefs.gl, effectRefs.renderStateCache);
      }
      effectRefs.renderStateCache = createRenderStateCache();
      effectRefs.renderState = null;
      effectRefs.gl = null;
    };
  }, [scheduleRender]);

  // Recreate render state when data changes
  useEffect(() => {
    const r = chartRef.current;
    const gl = r.gl;
    if (!gl) return;

    const interactionChanged =
      r.lastResolvedInteractionKey !== r.interactionKey;

    const nextDefaultZoom = getDefaultZoomState();
    const nextZoomState = resolveNextZoomState({
      liveMode,
      data,
      previousZoomState: r.zoomState,
      followsDefaultView: r.followsDefaultView,
      interactionChanged,
    });
    const nextFollowsDefaultView = isSameZoomState(
      nextZoomState,
      nextDefaultZoom,
    );
    const nextViewBounds = resolveNextViewBounds({
      data,
      followsDefaultView: nextFollowsDefaultView,
      previousViewBounds: r.viewBounds,
    });

    if (!data) {
      r.renderState = null;
      r.defaultZoomState = nextDefaultZoom;
      r.zoomState = nextZoomState;
      r.followsDefaultView = true;
      r.viewBounds = nextViewBounds;
      r.lastResolvedInteractionKey = r.interactionKey;
      r.zoomController?.setDefaultState(nextDefaultZoom);
      r.zoomController?.syncState(nextZoomState);
      scheduleRender();
      return;
    }

    r.renderState = syncRenderState(gl, r, data);

    r.defaultZoomState = nextDefaultZoom;
    r.zoomState = nextZoomState;
    r.followsDefaultView = nextFollowsDefaultView;
    r.viewBounds = nextViewBounds;
    r.lastResolvedInteractionKey = r.interactionKey;
    r.zoomController?.setDefaultState(nextDefaultZoom);
    r.zoomController?.syncState(nextZoomState);

    scheduleRender();
  }, [data, interactionKey, liveMode, scheduleRender]);

  // Re-render on theme change
  useEffect(() => {
    scheduleRender();
  }, [theme, numericLabelFormat, scheduleRender]);

  // Re-render on liveMode change
  useEffect(() => {
    scheduleRender();
  }, [liveMode, scheduleRender]);

  // Zoom controller, crosshair, and brush overlay
  // Keep the controller stable across live data refreshes so active drags are not interrupted,
  // but reset controller-local gesture state when the interaction identity changes.
  useEffect(() => {
    const currentRefs = chartRef.current;
    const svgEl = svgRef.current;
    const canvas = canvasRef.current;
    const currentData = currentRefs.data;
    if (!svgEl || !canvas || !currentData) {
      currentRefs.crosshairController?.destroy();
      currentRefs.crosshairController = null;
      currentRefs.zoomController?.destroy();
      currentRefs.zoomController = null;
      return;
    }

    const margin =
      currentData.type === "heatmap" ? HEATMAP_MARGIN : DEFAULT_MARGIN;
    const chartW = canvas.clientWidth - margin.left - margin.right;
    const chartH = canvas.clientHeight - margin.top - margin.bottom;

    // Brush overlay SVG group (managed by React hook, driven by zoom controller callback)
    const brushGroup = select(svgEl)
      .append("g")
      .attr("class", "brush-rect")
      .style("display", "none");
    const brushRect = brushGroup
      .append("rect")
      .attr("fill", "rgba(37, 99, 235, 0.15)")
      .attr("stroke", "rgba(37, 99, 235, 0.6)")
      .attr("stroke-width", 1);

    function handleBrushChange(rect: BrushRect | null) {
      if (!rect) {
        brushGroup.style("display", "none");
        return;
      }
      brushGroup.style("display", null);
      brushRect
        .attr("x", rect.x + margin.left)
        .attr("y", rect.y + margin.top)
        .attr("width", rect.width)
        .attr("height", rect.height);
    }

    const controller = attachZoom(
      svgEl,
      chartW,
      chartH,
      margin,
      (state, reason) => {
        currentRefs.zoomState = state;
        const followsDefault =
          reason === "reset"
            ? true
            : isSameZoomState(state, currentRefs.defaultZoomState);
        currentRefs.followsDefaultView = followsDefault;
        currentRefs.viewBounds = followsDefault
          ? getNumericBounds(currentRefs.data)
          : currentRefs.viewBounds;
        scheduleRender();
      },
      handleBrushChange,
      () => {
        currentRefs.followsDefaultView = true;
        currentRefs.viewBounds = getNumericBounds(currentRefs.data);
      },
    );
    controller.setDefaultState(currentRefs.defaultZoomState);
    controller.syncState(currentRefs.zoomState);
    currentRefs.zoomController = controller;

    // Crosshair overlay
    const crosshair = attachCrosshair(svgEl, () => {
      const d = currentRefs.data;
      if (!d) return null;

      const bounds = currentRefs.viewBounds ?? getNumericBounds(d);
      if (!bounds) return null;

      return {
        margin,
        zoomState: currentRefs.zoomState,
        ...bounds,
        theme: getOverlayTheme(currentRefs.theme),
        numericLabelFormat: currentRefs.numericLabelFormat,
      };
    });
    currentRefs.crosshairController = crosshair;

    return () => {
      crosshair.destroy();
      currentRefs.crosshairController = null;
      controller.destroy();
      currentRefs.zoomController = null;
      brushGroup.remove();
    };
  }, [chartType, interactionKey, scheduleRender]);

  // Resize observer — use devicePixelContentBoxSize when available for accurate DPR sizing
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.devicePixelContentBoxSize?.[0]) {
          const dpSize = entry.devicePixelContentBoxSize[0];
          const w = dpSize.inlineSize;
          const h = dpSize.blockSize;
          if (canvas.width !== w || canvas.height !== h) {
            canvas.width = w;
            canvas.height = h;
          }
        }
        // If devicePixelContentBoxSize is unavailable, resizeCanvas()
        // inside render() handles it via DPR multiplication.
      }
      // Update zoom translate extents so pan/zoom clamps match the new size
      const currentRefs = chartRef.current;
      if (currentRefs.zoomController) {
        const margin =
          currentRefs.data?.type === "heatmap"
            ? HEATMAP_MARGIN
            : DEFAULT_MARGIN;
        const chartW = canvas.clientWidth - margin.left - margin.right;
        const chartH = canvas.clientHeight - margin.top - margin.bottom;
        currentRefs.zoomController.updateExtents(chartW, chartH, margin);
      }
      scheduleRender();
    });

    // Request device-pixel-level notifications when supported
    try {
      observer.observe(canvas, { box: "device-pixel-content-box" });
    } catch {
      observer.observe(canvas);
    }

    return () => observer.disconnect();
  }, [scheduleRender]);

  const getInteractionState = useCallback(() => {
    const currentData = chartRef.current.data;
    if (!currentData) return null;

    const margin =
      currentData.type === "heatmap" ? HEATMAP_MARGIN : DEFAULT_MARGIN;

    if (currentData.type === "xy") {
      return {
        type: "xy" as const,
        ...(chartRef.current.viewBounds ?? lineDataBounds(currentData.series)),
        margin,
        zoomState: chartRef.current.zoomState,
      };
    }
    if (currentData.type === "heatmap") {
      const cachedState =
        chartRef.current.renderState?.type === "heatmap"
          ? chartRef.current.renderState.state
          : null;
      const layout = cachedState
        ? null
        : deriveHeatmapLayout(currentData.series);
      return {
        type: "heatmap" as const,
        ...(chartRef.current.viewBounds ??
          cachedState?.bounds ??
          layout!.bounds),
        margin,
        zoomState: chartRef.current.zoomState,
        geometry: cachedState?.geometry ?? layout!.geometry,
      };
    }

    return null;
  }, []);

  return { canvasRef, svgRef, getInteractionState };
}
interface ResolveNextZoomStateArgs {
  liveMode: boolean;
  data?: ChartOptions;
  previousZoomState: ZoomState;
  followsDefaultView: boolean;
  interactionChanged: boolean;
}

interface ResolveNextViewBoundsArgs {
  data?: ChartOptions;
  followsDefaultView: boolean;
  previousViewBounds: NumericBounds | null;
}

interface NumericBounds {
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
}

function getDefaultZoomState(): ZoomState {
  return IDENTITY_ZOOM;
}

function isSameZoomState(
  left: ZoomState,
  right: ZoomState,
  epsilon = 1e-4,
): boolean {
  return (
    Math.abs(left.scaleX - right.scaleX) <= epsilon &&
    Math.abs(left.scaleY - right.scaleY) <= epsilon &&
    Math.abs(left.translateX - right.translateX) <= epsilon &&
    Math.abs(left.translateY - right.translateY) <= epsilon
  );
}

function resolveNextZoomState({
  liveMode,
  data,
  previousZoomState,
  followsDefaultView,
  interactionChanged,
}: ResolveNextZoomStateArgs): ZoomState {
  const nextDefaultZoom = getDefaultZoomState();

  if (!data) {
    return nextDefaultZoom;
  }

  if (!liveMode || interactionChanged || followsDefaultView) {
    return nextDefaultZoom;
  }

  return previousZoomState;
}

function resolveNextViewBounds({
  data,
  followsDefaultView,
  previousViewBounds,
}: ResolveNextViewBoundsArgs): NumericBounds | null {
  if (!data) {
    return null;
  }

  if (followsDefaultView || !previousViewBounds) {
    return getNumericBounds(data);
  }

  return previousViewBounds;
}

function getNumericBounds(
  data: ChartOptions | undefined,
): NumericBounds | null {
  if (!data) return null;
  return data.type === "heatmap"
    ? deriveHeatmapLayout(data.series).bounds
    : lineDataBounds(data.series);
}

function createRenderStateCache(): RenderStateCache {
  return {
    line: null,
    scatter: null,
    heatmap: null,
  };
}

function syncRenderState(
  gl: WebGL2RenderingContext,
  refs: Pick<WebGLChartRefs, "renderStateCache">,
  data: ChartOptions,
): RenderState {
  switch (data.type) {
    case "xy": {
      const lineState = xyDrawStyleIncludesLine(data.drawStyle)
        ? (refs.renderStateCache.line ?? createLineRenderState(gl))
        : null;
      const scatterState = xyDrawStyleIncludesPoints(data.drawStyle)
        ? (refs.renderStateCache.scatter ?? createScatterRenderState(gl))
        : null;

      if (lineState) {
        refs.renderStateCache.line = lineState;
        syncLineRenderState(gl, lineState, data.series);
      }
      if (scatterState) {
        refs.renderStateCache.scatter = scatterState;
        syncScatterRenderState(gl, scatterState, data.series);
      }

      return { type: "xy", lineState, scatterState };
    }
    case "heatmap": {
      const state =
        refs.renderStateCache.heatmap ?? createHeatmapRenderState(gl);
      refs.renderStateCache.heatmap = state;
      syncHeatmapRenderState(gl, state, data.series);
      return { type: "heatmap", state };
    }
  }
}

function destroyRenderStateCache(
  gl: WebGL2RenderingContext,
  cache: RenderStateCache,
): void {
  if (cache.line) {
    destroyLineRenderState(gl, cache.line);
  }
  if (cache.scatter) {
    destroyScatterRenderState(gl, cache.scatter);
  }
  if (cache.heatmap) {
    destroyHeatmapRenderState(gl, cache.heatmap);
  }
}

function buildZoomedAxes(
  canvas: HTMLCanvasElement,
  margin: ChartMargin,
  bounds: NumericBounds,
  zoomState: ZoomState,
  theme: string | undefined,
) {
  const chartW = canvas.clientWidth - margin.left - margin.right;
  const chartH = canvas.clientHeight - margin.top - margin.bottom;
  const dataMatrix = dataToClipMatrix(
    bounds.xMin,
    bounds.xMax,
    bounds.yMin,
    bounds.yMax,
  );
  const zoomMatrix = zoomToClipMatrix(
    zoomState.scaleX,
    zoomState.translateX,
    zoomState.scaleY,
    zoomState.translateY,
    chartW,
    chartH,
  );
  const finalMatrix = mul3x3(zoomMatrix, dataMatrix);

  const overlayTheme = getOverlayTheme(theme);
  const xScale = scaleLinear()
    .domain([bounds.xMin, bounds.xMax])
    .range([0, chartW]);
  const yScale = scaleLinear()
    .domain([bounds.yMin, bounds.yMax])
    .range([chartH, 0]);

  const zoomedXScale =
    zoomState.scaleX !== 1 || zoomState.translateX !== 0
      ? xScale
          .copy()
          .domain(
            xScale
              .range()
              .map((px) =>
                xScale.invert((px - zoomState.translateX) / zoomState.scaleX),
              ),
          )
      : xScale;
  const zoomedYScale =
    zoomState.scaleY !== 1 || zoomState.translateY !== 0
      ? yScale
          .copy()
          .domain(
            yScale
              .range()
              .map((px) =>
                yScale.invert((px - zoomState.translateY) / zoomState.scaleY),
              ),
          )
      : yScale;

  return { finalMatrix, zoomedXScale, zoomedYScale, overlayTheme };
}
