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
        { valueIndices: [0, 0], displayValues: ["1", "a"], index: 0 },
        { valueIndices: [1, 1], displayValues: ["2", "b"], index: 1 },
      ],
      columnUniqueValues: {
        colA: [
          { index: 0, displayValue: "1" },
          { index: 1, displayValue: "2" },
        ],
        colB: [
          { index: 0, displayValue: "a" },
          { index: 1, displayValue: "b" },
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
