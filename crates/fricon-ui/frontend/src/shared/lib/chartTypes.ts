export type ChartView = "xy" | "heatmap";

export type XYPlotMode = "quantity_vs_sweep" | "xy" | "complex_plane";

export type XYDrawStyle = "line" | "points" | "line_points";

export type ComplexViewOption = "real" | "imag" | "mag" | "arg";

export type NumericLabelFormatMode = "auto" | "decimal" | "scientific" | "si";

export interface NumericLabelFormatOptions {
  mode: NumericLabelFormatMode;
  significantDigits: number;
}

export interface ChartSeries {
  id: string;
  label: string;
  values: Float64Array;
  pointCount: number;
}

export interface HeatmapSeries {
  id: string;
  label: string;
  values: Float64Array;
  pointCount: number;
}

export interface XYChartModel {
  type: "xy";
  plotMode: XYPlotMode;
  drawStyle: XYDrawStyle;
  xName: string;
  yName: string | null;
  series: ChartSeries[];
}

export interface HeatmapChartModel {
  type: "heatmap";
  xName: string;
  yName: string;
  xCategories: number[];
  yCategories: number[];
  series: HeatmapSeries[];
}

export type ChartModel = XYChartModel | HeatmapChartModel;

export type ChartOptions = ChartModel;

export function xyDrawStyleIncludesLine(style: XYDrawStyle) {
  return style === "line" || style === "line_points";
}

export function xyDrawStyleIncludesPoints(style: XYDrawStyle) {
  return style === "points" || style === "line_points";
}

export function getXYPoint(series: ChartSeries, index: number) {
  const offset = index * 2;
  return {
    x: series.values[offset] ?? 0,
    y: series.values[offset + 1] ?? 0,
  };
}

export function getXYZPoint(series: HeatmapSeries, index: number) {
  const offset = index * 3;
  return {
    x: series.values[offset] ?? 0,
    y: series.values[offset + 1] ?? 0,
    z: series.values[offset + 2] ?? 0,
  };
}
