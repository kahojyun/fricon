import type {
  ComplexViewOption as WireComplexViewOption,
  Type as WireChartType,
} from "@/lib/bindings";

export type ChartType = WireChartType;

export type ScatterMode = "complex" | "trace_xy" | "xy";

export type ComplexViewOption = WireComplexViewOption;

export type ChartSeriesData = number[][];

export interface ChartSeries {
  name: string;
  data: ChartSeriesData;
}

export type ChartOptions =
  | {
      type: "line";
      xName: string;
      series: ChartSeries[];
    }
  | {
      type: "heatmap";
      xName: string;
      yName: string;
      xCategories: number[];
      yCategories: number[];
      series: ChartSeries[];
    }
  | {
      type: "scatter";
      xName: string;
      yName: string;
      series: ChartSeries[];
    };
