import { describe, expect, it } from "vitest";
import { deriveLegendPresentation } from "./chartLegendPresentation";

describe("deriveLegendPresentation", () => {
  it("moves a shared base label into the title", () => {
    expect(
      deriveLegendPresentation([
        xySeries("a", "signal (real)"),
        xySeries("b", "signal (imag)"),
      ]),
    ).toEqual({
      items: [
        { id: "a", text: "real" },
        { id: "b", text: "imag" },
      ],
    });
  });

  it("leaves full labels alone when there is no shared base", () => {
    expect(
      deriveLegendPresentation([xySeries("a", "alpha"), xySeries("b", "beta")]),
    ).toEqual({
      items: [
        { id: "a", text: "alpha" },
        { id: "b", text: "beta" },
      ],
    });
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
