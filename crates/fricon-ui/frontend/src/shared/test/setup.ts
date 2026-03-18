import "@testing-library/jest-dom/vitest";

const noop = () => undefined;

function createMemoryStorage(): Storage {
  const store = new Map<string, string>();
  return {
    get length() {
      return store.size;
    },
    clear() {
      store.clear();
    },
    getItem(key: string) {
      return store.get(key) ?? null;
    },
    key(index: number) {
      return Array.from(store.keys())[index] ?? null;
    },
    removeItem(key: string) {
      store.delete(key);
    },
    setItem(key: string, value: string) {
      store.set(key, value);
    },
  };
}

Object.defineProperty(window, "localStorage", {
  value: createMemoryStorage(),
  configurable: true,
});

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
