import type { ComponentProps } from "react";
import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { DatasetDeleteResult } from "../api/types";
import { DatasetTableRowActions } from "./DatasetTableRowActions";
import { makeDataset } from "./test-utils";

const { toastSuccess, toastError, toastWarning } = vi.hoisted(() => ({
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
  toastWarning: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: toastSuccess,
    error: toastError,
    warning: toastWarning,
  },
}));

function renderRowActions(
  overrides: Partial<ComponentProps<typeof DatasetTableRowActions>> = {},
) {
  const selectedDatasets = [
    makeDataset({ id: 11, name: "Dataset B", tags: ["vision"] }),
  ];
  const props = {
    dataset: makeDataset({ id: 11, name: "Dataset B", tags: ["vision"] }),
    viewMode: "active" as const,
    selectedDatasets,
    allTags: ["vision", "audio"],
    isUpdatingTags: false,
    onDatasetSelected: vi.fn(),
    onTrash: vi.fn(),
    onRestore: vi.fn(),
    onPermanentDelete: vi.fn(),
    batchAddTags: vi
      .fn()
      .mockResolvedValue([
        { id: 11, success: true, error: null },
      ] satisfies DatasetDeleteResult[]),
    batchRemoveTags: vi
      .fn()
      .mockResolvedValue([
        { id: 11, success: true, error: null },
      ] satisfies DatasetDeleteResult[]),
    children: <div>Row trigger</div>,
    ...overrides,
  };

  render(<DatasetTableRowActions {...props} />);
  return props;
}

async function openRowActions(user: ReturnType<typeof userEvent.setup>) {
  await user.pointer({
    keys: "[MouseRight]",
    target: screen.getByText("Row trigger"),
  });

  const menus = await screen.findAllByRole("menu");
  return menus.at(-1)!;
}

describe("DatasetTableRowActions", () => {
  it("shows multi-row targeting labels and bulk delete when the clicked row is selected", async () => {
    const props = renderRowActions({
      selectedDatasets: [
        makeDataset({ id: 10, name: "Dataset A", tags: [] }),
        makeDataset({
          id: 11,
          name: "Dataset B",
          tags: ["vision"],
        }),
      ],
    });
    const user = userEvent.setup();

    const menu = await openRowActions(user);

    expect(within(menu).getByText(/Add Tags \(2\)/i)).toBeInTheDocument();
    expect(
      within(menu).getByRole("menuitem", {
        name: "Move Selected to Trash (2)",
      }),
    ).toBeInTheDocument();

    await user.click(
      within(menu).getByRole("menuitem", {
        name: "Move Selected to Trash (2)",
      }),
    );

    expect(props.onTrash).toHaveBeenCalledWith([10, 11]);
  });

  it("shows only single-row trash action when the clicked row is outside the multi-selection", async () => {
    renderRowActions({
      selectedDatasets: [makeDataset({ id: 10, name: "Dataset A", tags: [] })],
    });
    const user = userEvent.setup();

    const menu = await openRowActions(user);

    expect(
      within(menu).getByRole("menuitem", { name: "Move to Trash" }),
    ).toBeInTheDocument();
    expect(
      within(menu).queryByRole("menuitem", { name: /Move Selected to Trash/i }),
    ).not.toBeInTheDocument();
  });

  it("shows restore and permanent delete actions in trash view", async () => {
    renderRowActions({ viewMode: "trash" });
    const user = userEvent.setup();

    const menu = await openRowActions(user);

    expect(
      within(menu).getByRole("menuitem", { name: "Restore" }),
    ).toBeInTheDocument();
    expect(
      within(menu).getByRole("menuitem", { name: "Permanently Delete" }),
    ).toBeInTheDocument();
  });

  it("wires tag add and remove actions through the batch mutation boundary", async () => {
    const props = renderRowActions();
    const user = userEvent.setup();

    let menu = await openRowActions(user);
    const addTrigger = within(menu).getByRole("menuitem", {
      name: /^Add Tags/i,
    });
    fireEvent.pointerMove(addTrigger);
    fireEvent.click(addTrigger);
    fireEvent.click(
      (await screen.findAllByRole("menuitem", { name: "audio" })).at(-1)!,
    );

    await waitFor(() => {
      expect(props.batchAddTags).toHaveBeenCalledWith([11], ["audio"]);
    });

    menu = await openRowActions(user);
    const removeTrigger = within(menu).getByRole("menuitem", {
      name: /^Remove Tags/i,
    });
    fireEvent.pointerMove(removeTrigger);
    fireEvent.click(removeTrigger);
    fireEvent.click(
      (await screen.findAllByRole("menuitem", { name: "vision" })).at(-1)!,
    );

    await waitFor(() => {
      expect(props.batchRemoveTags).toHaveBeenCalledWith([11], ["vision"]);
    });
  });
});
