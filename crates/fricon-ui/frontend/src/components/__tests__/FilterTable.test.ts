import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import FilterTable from "../FilterTable.vue";
import PrimeVue from "primevue/config";
import type { FilterTableData } from "@/backend";

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
  const createTestFilterTableData = (): FilterTableData => {
    return {
      fields: ["colA", "colB"],
      rows: [
        { values: [1, "a"], index: 0 },
        { values: [2, "b"], index: 1 },
      ],
      columnUniqueValues: {
        colA: [
          { value: 1, displayValue: "1" },
          { value: 2, displayValue: "2" },
        ],
        colB: [
          { value: "a", displayValue: "a" },
          { value: "b", displayValue: "b" },
        ],
      },
    };
  };

  it("renders correctly with data", () => {
    const wrapper = mount(FilterTable, {
      props: {
        filterTableData: createTestFilterTableData(),
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
