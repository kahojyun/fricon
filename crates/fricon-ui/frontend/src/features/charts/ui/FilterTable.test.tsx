import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { FilterTableData } from "../api/types";
import { FilterTable } from "./FilterTable";

vi.mock("@tanstack/react-virtual", () => ({
  useVirtualizer: ({ count }: { count: number }) => ({
    getTotalSize: () => count * 32,
    getVirtualItems: () =>
      Array.from({ length: count }, (_, index) => ({
        index,
        start: index * 32,
        end: (index + 1) * 32,
      })),
    measureElement: () => undefined,
    scrollToIndex: () => undefined,
  }),
}));

function makeData(): FilterTableData {
  return {
    fields: ["A", "B"],
    rows: [
      { index: 1, displayValues: ["A1", "B1"], valueIndices: [1, 1] },
      { index: 2, displayValues: ["A2", "B2"], valueIndices: [2, 2] },
    ],
    columnUniqueValues: {
      A: [
        { index: 1, displayValue: "A1" },
        { index: 2, displayValue: "A2" },
      ],
      B: [
        { index: 1, displayValue: "B1" },
        { index: 2, displayValue: "B2" },
      ],
    },
  };
}

describe("FilterTable", () => {
  it("supports keyboard selection in split mode", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={null}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const option = screen.getByText("A2").closest("tr");
    if (!(option instanceof HTMLElement)) {
      throw new Error("Split mode option row not found");
    }

    option.focus();
    await user.keyboard("{Enter}");
    await user.keyboard(" ");

    expect(onSelectFieldValue).toHaveBeenNthCalledWith(1, 0, 2);
    expect(onSelectFieldValue).toHaveBeenNthCalledWith(2, 0, 2);
  });

  it("moves row-mode selection with ArrowUp and ArrowDown", async () => {
    const user = userEvent.setup();
    const onSelectRow = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="row"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={onSelectRow}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={() => undefined}
        />
      </div>,
    );

    const firstRow = screen.getByText("A1").closest("tr");
    const secondRow = screen.getByText("A2").closest("tr");

    if (
      !(firstRow instanceof HTMLElement) ||
      !(secondRow instanceof HTMLElement)
    ) {
      throw new Error("Row mode rows not found");
    }

    firstRow.focus();
    await user.keyboard("{ArrowDown}");

    expect(onSelectRow).toHaveBeenCalledWith(2);
    expect(secondRow).toHaveFocus();

    onSelectRow.mockClear();
    secondRow.focus();
    await user.keyboard("{ArrowUp}");

    expect(onSelectRow).toHaveBeenCalledWith(1);
    expect(firstRow).toHaveFocus();
  });

  it("keeps arrow navigation bounded in row mode", async () => {
    const user = userEvent.setup();
    const onSelectRow = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="row"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={onSelectRow}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={() => undefined}
        />
      </div>,
    );

    const firstRow = screen.getByText("A1").closest("tr");
    const secondRow = screen.getByText("A2").closest("tr");

    if (
      !(firstRow instanceof HTMLElement) ||
      !(secondRow instanceof HTMLElement)
    ) {
      throw new Error("Row mode rows not found");
    }

    firstRow.focus();
    await user.keyboard("{ArrowUp}");
    secondRow.focus();
    await user.keyboard("{ArrowDown}");

    expect(onSelectRow).not.toHaveBeenCalled();
  });

  it("moves split-mode selection with ArrowUp and ArrowDown", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const firstOption = screen.getByText("A1").closest("tr");
    const secondOption = screen.getByText("A2").closest("tr");

    if (
      !(firstOption instanceof HTMLElement) ||
      !(secondOption instanceof HTMLElement)
    ) {
      throw new Error("Split mode option rows not found");
    }

    firstOption.focus();
    await user.keyboard("{ArrowDown}");

    expect(onSelectFieldValue).toHaveBeenCalledWith(0, 2);
    expect(secondOption).toHaveFocus();

    onSelectFieldValue.mockClear();
    secondOption.focus();
    await user.keyboard("{ArrowUp}");

    expect(onSelectFieldValue).toHaveBeenCalledWith(0, 1);
    expect(firstOption).toHaveFocus();
  });

  it("keeps arrow navigation bounded in split mode", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const firstOption = screen.getByText("A1").closest("tr");
    const secondOption = screen.getByText("A2").closest("tr");

    if (
      !(firstOption instanceof HTMLElement) ||
      !(secondOption instanceof HTMLElement)
    ) {
      throw new Error("Split mode option rows not found");
    }

    firstOption.focus();
    await user.keyboard("{ArrowUp}");
    secondOption.focus();
    await user.keyboard("{ArrowDown}");

    expect(onSelectFieldValue).not.toHaveBeenCalled();
  });

  it("moves focus across split columns with ArrowLeft and ArrowRight", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const leftColumnItem = screen.getByText("A2").closest("tr");
    const leftColumnSelectedItem = screen.getByText("A1").closest("tr");
    const rightColumnSelectedItem = screen.getByText("B1").closest("tr");

    if (
      !(leftColumnItem instanceof HTMLElement) ||
      !(leftColumnSelectedItem instanceof HTMLElement) ||
      !(rightColumnSelectedItem instanceof HTMLElement)
    ) {
      throw new Error("Split mode rows not found");
    }

    leftColumnItem.focus();
    await user.keyboard("{ArrowRight}");

    expect(rightColumnSelectedItem).toHaveFocus();
    expect(onSelectFieldValue).not.toHaveBeenCalled();

    await user.keyboard("{ArrowLeft}");

    expect(leftColumnSelectedItem).toHaveFocus();
    expect(onSelectFieldValue).not.toHaveBeenCalled();
  });

  it("moves focus to the target column's selected row before vertical navigation", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={() => undefined}
          selectedValueIndices={[2, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const leftSelectedItem = screen.getByText("A2").closest("tr");
    const rightSelectedItem = screen.getByText("B1").closest("tr");

    if (
      !(leftSelectedItem instanceof HTMLElement) ||
      !(rightSelectedItem instanceof HTMLElement)
    ) {
      throw new Error("Split mode rows not found");
    }

    leftSelectedItem.focus();
    await user.keyboard("{ArrowRight}");

    expect(rightSelectedItem).toHaveFocus();
    expect(onSelectFieldValue).not.toHaveBeenCalled();

    await user.keyboard("{ArrowDown}");

    expect(onSelectFieldValue).toHaveBeenCalledWith(1, 2);
  });

  it("keeps left-right split focus movement bounded at the edges", async () => {
    const user = userEvent.setup();
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const leftColumnItem = screen.getByText("A1").closest("tr");
    const rightColumnItem = screen.getByText("B1").closest("tr");

    if (
      !(leftColumnItem instanceof HTMLElement) ||
      !(rightColumnItem instanceof HTMLElement)
    ) {
      throw new Error("Split mode rows not found");
    }

    leftColumnItem.focus();
    await user.keyboard("{ArrowLeft}");
    expect(leftColumnItem).toHaveFocus();

    rightColumnItem.focus();
    await user.keyboard("{ArrowRight}");
    expect(rightColumnItem).toHaveFocus();
    expect(onSelectFieldValue).not.toHaveBeenCalled();
  });

  it("prevents default scrolling for handled arrow keys in row mode", () => {
    const onSelectRow = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="row"
          onModeChange={() => undefined}
          selectedRowIndex={1}
          onSelectRow={onSelectRow}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={() => undefined}
        />
      </div>,
    );

    const firstRow = screen.getByText("A1").closest("tr");
    if (!(firstRow instanceof HTMLElement)) {
      throw new Error("Row mode row not found");
    }

    const event = new KeyboardEvent("keydown", {
      bubbles: true,
      cancelable: true,
      key: "ArrowDown",
    });
    const prevented = !firstRow.dispatchEvent(event);

    expect(prevented || event.defaultPrevented).toBe(true);
    expect(onSelectRow).toHaveBeenCalledWith(2);
  });

  it("prevents scroll behavior when using Space to select", () => {
    const onSelectFieldValue = vi.fn();

    render(
      <div className="h-60">
        <FilterTable
          data={makeData()}
          mode="split"
          onModeChange={() => undefined}
          selectedRowIndex={null}
          onSelectRow={() => undefined}
          selectedValueIndices={[1, 1]}
          onSelectFieldValue={onSelectFieldValue}
        />
      </div>,
    );

    const option = screen.getByText("A2").closest("tr");
    if (!(option instanceof HTMLElement)) {
      throw new Error("Split mode option row not found");
    }

    const event = new KeyboardEvent("keydown", {
      bubbles: true,
      cancelable: true,
      key: " ",
    });
    const prevented = !option.dispatchEvent(event);

    expect(prevented || event.defaultPrevented).toBe(true);
    expect(onSelectFieldValue).toHaveBeenCalledWith(0, 2);
  });
});
