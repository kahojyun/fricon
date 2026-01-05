import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import FilterTable from "../FilterTable.vue";
import { tableFromArrays } from "apache-arrow";
import PrimeVue from "primevue/config";

// Mock sub-components to avoid PrimeVue dependency issues in unit tests
const DataTable = {
  template: '<div class="p-datatable"><slot></slot></div>',
  props: [
    "value",
    "selection",
    "selectionMode",
    "dataKey",
    "virtualScrollerOptions",
  ],
  emits: ["update:selection"],
};
const Column = {
  template: '<div class="p-column"></div>',
  props: ["field", "header"],
};
const ToggleSwitch = {
  template: '<input type="checkbox" class="p-toggle-switch" />',
  props: ["modelValue"],
  emits: ["update:modelValue"],
};
const Divider = { template: '<hr class="p-divider" />' };

describe("FilterTable.vue", () => {
  const createTestTable = () => {
    return tableFromArrays({
      colA: [1, 2, 1],
      colB: ["a", "b", "a"],
      x: [10, 20, 30],
    });
  };

  it("renders correctly with data", () => {
    const wrapper = mount(FilterTable, {
      props: {
        indexTable: createTestTable(),
        xColumnName: "x",
        datasetId: "test-ds",
      },
      global: {
        plugins: [PrimeVue],
        stubs: {
          DataTable,
          Column,
          ToggleSwitch,
          Divider,
        },
      },
    });

    expect(wrapper.exists()).toBe(true);
    expect(wrapper.find(".p-datatable").exists()).toBe(true);
  });
});
