import "@testing-library/jest-dom/vitest";

const noop = () => undefined;

if (!window.matchMedia) {
  window.matchMedia = (query) =>
    ({
      matches: false,
      media: query,
      addEventListener: noop,
      removeEventListener: noop,
      onchange: null,
      addListener: noop,
      removeListener: noop,
      dispatchEvent: () => false,
    }) as MediaQueryList;
}

class ResizeObserverStub {
  observe() {
    return undefined;
  }
  unobserve() {
    return undefined;
  }
  disconnect() {
    return undefined;
  }
}

if (!window.ResizeObserver) {
  window.ResizeObserver = ResizeObserverStub;
}

// Tauri's official mockIPC will be configured in tests as needed.

if (!Element.prototype.getAnimations) {
  Element.prototype.getAnimations = () => [];
}
