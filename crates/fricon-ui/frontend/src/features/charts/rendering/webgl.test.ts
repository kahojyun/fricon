import { describe, expect, it } from "vitest";
import { dataToClipMatrix, mul3x3, zoomToClipMatrix } from "./webgl";

function applyMatrix(
  matrix: Float32Array,
  x: number,
  y: number,
): { x: number; y: number } {
  return {
    x: matrix[0] * x + matrix[3] * y + matrix[6],
    y: matrix[1] * x + matrix[4] * y + matrix[7],
  };
}

describe("zoomToClipMatrix", () => {
  it("matches d3 pixel-space zoom on the x axis", () => {
    const width = 100;
    const height = 80;
    const baseMatrix = dataToClipMatrix(0, 10, 0, 10);
    const zoomMatrix = zoomToClipMatrix(2, -25, 1, 0, width, height);
    const finalMatrix = mul3x3(zoomMatrix, baseMatrix);

    const clipPoint = applyMatrix(finalMatrix, 5, 5);

    // D3 pixel-space zoom: xScale(5) = 50, then 2 * 50 - 25 = 75 px.
    // Convert back to clip: 2 * 75 / 100 - 1 = 0.5.
    expect(clipPoint.x).toBeCloseTo(0.5);
  });

  it("matches d3 pixel-space zoom on the y axis", () => {
    const width = 100;
    const height = 80;
    const baseMatrix = dataToClipMatrix(0, 10, 0, 10);
    const zoomMatrix = zoomToClipMatrix(1, 0, 2, -10, width, height);
    const finalMatrix = mul3x3(zoomMatrix, baseMatrix);

    const clipPoint = applyMatrix(finalMatrix, 5, 5);

    // D3 pixel-space zoom: yScale(5) = 40, then 2 * 40 - 10 = 70 px from top.
    // Convert back to clip: 1 - 2 * 70 / 80 = -0.75.
    expect(clipPoint.y).toBeCloseTo(-0.75);
  });

  it("supports anisotropic scaling on both axes", () => {
    const width = 100;
    const height = 80;
    const baseMatrix = dataToClipMatrix(0, 10, 0, 10);
    const zoomMatrix = zoomToClipMatrix(3, -20, 0.5, 10, width, height);
    const finalMatrix = mul3x3(zoomMatrix, baseMatrix);

    const clipPoint = applyMatrix(finalMatrix, 5, 5);

    // xScale(5)=50 -> 3*50-20 = 130 px -> 2*130/100-1 = 1.6
    expect(clipPoint.x).toBeCloseTo(1.6);
    // yScale(5)=40 -> 0.5*40+10 = 30 px from top -> 1-2*30/80 = 0.25
    expect(clipPoint.y).toBeCloseTo(0.25);
  });
});
