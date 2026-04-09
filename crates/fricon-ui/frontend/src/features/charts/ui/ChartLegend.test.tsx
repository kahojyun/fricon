import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ChartLegend } from "./ChartLegend";
import { deriveLegendPresentation } from "./chartLegendPresentation";

describe("ChartLegend", () => {
  it("renders only the varying parts for shared-base series", () => {
    const presentation = deriveLegendPresentation([
      xySeries("a", "signal (real)"),
      xySeries("b", "signal (imag)"),
    ]);

    render(<ChartLegend items={presentation.items} />);

    expect(screen.getByText("real")).toBeInTheDocument();
    expect(screen.getByText("imag")).toBeInTheDocument();
    expect(screen.queryByText("signal")).not.toBeInTheDocument();
  });

  it("keeps varying group labels when they distinguish the entries", () => {
    const presentation = deriveLegendPresentation([
      xySeries("a", "signal [idx=1]"),
      xySeries("b", "signal [idx=2]"),
    ]);

    render(<ChartLegend items={presentation.items} />);

    expect(screen.getByText("[idx=1]")).toBeInTheDocument();
    expect(screen.getByText("[idx=2]")).toBeInTheDocument();
    expect(screen.queryByText("signal")).not.toBeInTheDocument();
  });
});

function xySeries(id: string, label: string) {
  return {
    id,
    label,
    pointCount: 1,
    values: new Float64Array([0, 0]),
  };
}
