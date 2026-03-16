import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ChartOptions } from "@/shared/lib/chartTypes";
import { ChartWrapper } from "./ChartWrapper";

const capturedOptions: unknown[] = [];

vi.mock("echarts-for-react/esm/core", () => ({
  default: ({ option }: { option: unknown }) => {
    capturedOptions.push(option);
    return <div data-testid="echarts-instance" />;
  },
}));

describe("ChartWrapper", () => {
  beforeEach(() => {
    capturedOptions.length = 0;
  });

  it("uses backend heatmap categories instead of inferring from series indexes", () => {
    const data: ChartOptions = {
      type: "heatmap",
      xName: "x",
      yName: "y",
      xCategories: [1, 2],
      yCategories: [10],
      series: [
        {
          name: "z",
          data: [
            [0, 0, 100],
            [1, 0, 200],
          ],
        },
      ],
    };

    render(<ChartWrapper data={data} />);
    expect(screen.getByTestId("echarts-instance")).toBeInTheDocument();

    const option = capturedOptions.at(-1) as {
      xAxis: { data: number[] };
      yAxis: { data: number[] };
      series: { data: number[][] }[];
    };
    expect(option.xAxis.data).toEqual([1, 2]);
    expect(option.yAxis.data).toEqual([10]);
    expect(option.series[0].data).toEqual([
      [0, 0, 100],
      [1, 0, 200],
    ]);
  });
});
