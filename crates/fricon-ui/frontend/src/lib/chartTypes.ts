export type ChartType = "line" | "heatmap" | "scatter";

export type ScatterMode = "complex" | "trace_xy" | "xy";

export type ComplexViewOption = "real" | "imag" | "mag" | "arg";

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
      series: ChartSeries[];
    }
  | {
      type: "scatter";
      xName: string;
      yName: string;
      series: ChartSeries[];
    };
