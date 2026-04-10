import { describe, expect, it } from "vitest";
import { attachCrosshair } from "./crosshairOverlay";
import { LIGHT_THEME } from "./d3Overlay";

describe("attachCrosshair", () => {
  it("formats crosshair axis readouts with the shared formatter", () => {
    const svgEl = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    document.body.appendChild(svgEl);

    Object.defineProperty(svgEl, "clientWidth", {
      configurable: true,
      value: 300,
    });
    Object.defineProperty(svgEl, "clientHeight", {
      configurable: true,
      value: 200,
    });
    Object.defineProperty(svgEl, "getBoundingClientRect", {
      configurable: true,
      value: () => ({
        left: 0,
        top: 0,
        width: 300,
        height: 200,
        right: 300,
        bottom: 200,
        x: 0,
        y: 0,
        toJSON: () => undefined,
      }),
    });
    Object.defineProperty(SVGElement.prototype, "getBBox", {
      configurable: true,
      value: () => ({
        x: 0,
        y: 0,
        width: 24,
        height: 10,
      }),
    });

    const controller = attachCrosshair(svgEl, () => ({
      margin: {
        top: 20,
        right: 20,
        bottom: 40,
        left: 60,
      },
      zoomState: {
        scaleX: 1,
        scaleY: 1,
        translateX: 0,
        translateY: 0,
      },
      xMin: 0,
      xMax: 0.001,
      yMin: 12000,
      yMax: 13000,
      theme: LIGHT_THEME,
      numericLabelFormat: {
        mode: "scientific",
        significantDigits: 4,
      },
    }));

    svgEl.dispatchEvent(
      new PointerEvent("pointermove", {
        clientX: 180,
        clientY: 80,
        bubbles: true,
      }),
    );

    const labels = Array.from(svgEl.querySelectorAll(".crosshair text")).map(
      (node) => node.textContent,
    );

    expect(labels).toContain("5.455e-4");
    expect(labels).toContain("1.257e+4");

    controller.destroy();
  });
});
