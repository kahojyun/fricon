import { fireEvent } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  attachZoom,
  IDENTITY_ZOOM,
  scaleZoomStateAroundPoint,
} from "./zoomController";

describe("scaleZoomStateAroundPoint", () => {
  it("scales around the cursor on the x axis", () => {
    const zoomState = scaleZoomStateAroundPoint(
      IDENTITY_ZOOM,
      { x: 80, y: 45 },
      1.5,
      1,
    );

    expect(zoomState).toEqual({
      scaleX: 1.5,
      scaleY: 1,
      translateX: -40,
      translateY: 0,
    });
  });

  it("preserves the anchor for anisotropic right-drag zoom", () => {
    const startState = {
      scaleX: 2,
      scaleY: 0.75,
      translateX: -50,
      translateY: 15,
    };
    const anchor = { x: 120, y: 35 };

    const zoomState = scaleZoomStateAroundPoint(startState, anchor, 0.5, 2);

    const inputX = 70;
    const anchoredDisplayX = startState.scaleX * inputX + startState.translateX;
    const scaledDisplayX = zoomState.scaleX * inputX + zoomState.translateX;
    expect(scaledDisplayX).toBeCloseTo(
      0.5 * (anchoredDisplayX - anchor.x) + anchor.x,
    );

    const inputY = 40;
    const anchoredDisplayY = startState.scaleY * inputY + startState.translateY;
    const scaledDisplayY = zoomState.scaleY * inputY + zoomState.translateY;
    expect(scaledDisplayY).toBeCloseTo(
      2 * (anchoredDisplayY - anchor.y) + anchor.y,
    );
  });

  it("maps right-drag horizontal and vertical motion to x/y scaling", () => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    Object.defineProperty(svg, "getBoundingClientRect", {
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
    svg.setPointerCapture = () => undefined;
    svg.releasePointerCapture = () => undefined;
    svg.hasPointerCapture = () => true;

    const states = [IDENTITY_ZOOM];
    const controller = attachZoom(
      svg,
      220,
      140,
      { left: 60, top: 20 },
      (state) => states.push(state),
    );

    fireEvent.pointerDown(svg, {
      button: 2,
      pointerId: 1,
      clientX: 120,
      clientY: 80,
    });
    fireEvent.pointerMove(svg, {
      button: 2,
      pointerId: 1,
      clientX: 170,
      clientY: 50,
    });

    const state = states.at(-1)!;
    expect(state.scaleX).toBeGreaterThan(1);
    expect(state.scaleY).toBeGreaterThan(1);
    expect(state.translateX).toBeLessThan(0);
    expect(state.translateY).toBeLessThan(0);

    controller.destroy();
  });

  it("allows dragging beyond the chart bounds after capture", () => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    Object.defineProperty(svg, "getBoundingClientRect", {
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
    svg.setPointerCapture = () => undefined;
    svg.releasePointerCapture = () => undefined;
    svg.hasPointerCapture = () => true;

    const states = [IDENTITY_ZOOM];
    const controller = attachZoom(
      svg,
      220,
      140,
      { left: 60, top: 20 },
      (state) => states.push(state),
    );

    fireEvent.pointerDown(svg, {
      button: 0,
      pointerId: 1,
      clientX: 120,
      clientY: 80,
    });
    fireEvent.pointerMove(svg, {
      button: 0,
      pointerId: 1,
      clientX: 340,
      clientY: 240,
    });

    const state = states.at(-1)!;
    expect(state.translateX).toBeGreaterThan(0);
    expect(state.translateY).toBeGreaterThan(0);

    controller.destroy();
  });

  it("allows zooming out beyond the previous minimum scale", () => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    Object.defineProperty(svg, "getBoundingClientRect", {
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
    svg.setPointerCapture = () => undefined;
    svg.releasePointerCapture = () => undefined;
    svg.hasPointerCapture = () => true;

    const states = [IDENTITY_ZOOM];
    const controller = attachZoom(
      svg,
      220,
      140,
      { left: 60, top: 20 },
      (state) => states.push(state),
    );

    fireEvent.wheel(svg, {
      clientX: 160,
      clientY: 90,
      deltaY: 600,
    });

    const state = states.at(-1)!;
    expect(state.scaleX).toBeLessThan(0.5);
    expect(state.scaleY).toBeLessThan(0.5);

    controller.destroy();
  });

  it("resets to the configured default view", () => {
    const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
    Object.defineProperty(svg, "getBoundingClientRect", {
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
    svg.setPointerCapture = () => undefined;
    svg.releasePointerCapture = () => undefined;
    svg.hasPointerCapture = () => true;

    const defaultState = {
      scaleX: 1.8,
      scaleY: 1.4,
      translateX: -32,
      translateY: -18,
    };
    let latestState = IDENTITY_ZOOM;

    const nowSpy = vi.spyOn(performance, "now").mockReturnValue(0);
    const originalRequestAnimationFrame = globalThis.requestAnimationFrame;
    const originalCancelAnimationFrame = globalThis.cancelAnimationFrame;
    globalThis.requestAnimationFrame = ((callback: FrameRequestCallback) => {
      callback(250);
      return 1;
    }) as typeof requestAnimationFrame;
    globalThis.cancelAnimationFrame = (() =>
      undefined) as typeof cancelAnimationFrame;

    try {
      const controller = attachZoom(
        svg,
        220,
        140,
        { left: 60, top: 20 },
        (state) => {
          latestState = state;
        },
      );

      controller.setDefaultState(defaultState);
      controller.syncState({
        scaleX: 3,
        scaleY: 2.5,
        translateX: -80,
        translateY: -50,
      });
      controller.reset();

      expect(latestState).toEqual(defaultState);

      controller.destroy();
    } finally {
      nowSpy.mockRestore();
      globalThis.requestAnimationFrame = originalRequestAnimationFrame;
      globalThis.cancelAnimationFrame = originalCancelAnimationFrame;
    }
  });
});
