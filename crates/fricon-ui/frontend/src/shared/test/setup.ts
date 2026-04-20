import "@testing-library/jest-dom/vitest";

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
  // A few intentionally retained unit tests still exercise chart hooks/components
  // that subscribe to ResizeObserver under jsdom.
  window.ResizeObserver = ResizeObserverStub;
}

// Tauri's official mockIPC will be configured in tests as needed.
