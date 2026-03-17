import type { ComponentProps } from "react";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DatasetTagFilter } from "./DatasetTagFilter";

const { toastSuccess, toastError } = vi.hoisted(() => ({
  toastSuccess: vi.fn(),
  toastError: vi.fn(),
}));

vi.mock("sonner", () => ({
  toast: {
    success: toastSuccess,
    error: toastError,
  },
}));

function renderTagFilter(
  overrides: Partial<ComponentProps<typeof DatasetTagFilter>> = {},
) {
  const props = {
    selectedTags: [],
    allTags: ["vision", "audio"],
    isUpdatingTags: false,
    onToggleTag: vi.fn(),
    onDeleteTag: vi.fn().mockResolvedValue(undefined),
    onRenameTag: vi.fn().mockResolvedValue(undefined),
    onMergeTag: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };

  render(<DatasetTagFilter {...props} />);
  return props;
}

describe("DatasetTagFilter", () => {
  beforeEach(() => {
    toastSuccess.mockReset();
    toastError.mockReset();
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("filters tags locally from the search input", async () => {
    renderTagFilter();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));
    await user.type(screen.getByPlaceholderText("Search tags"), "aud");

    expect(screen.getByText("audio")).toBeInTheDocument();
    expect(screen.queryByText("vision")).not.toBeInTheDocument();
  });

  it("toggles a tag selection", async () => {
    const props = renderTagFilter();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));
    await user.click(screen.getByText("audio"));

    expect(props.onToggleTag).toHaveBeenCalledWith("audio");
  });

  it("clears all selected tags", async () => {
    const props = renderTagFilter({
      selectedTags: ["vision", "audio"],
    });
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));
    await user.click(screen.getByRole("button", { name: /Clear filters/i }));

    expect(props.onToggleTag).toHaveBeenCalledWith("vision");
    expect(props.onToggleTag).toHaveBeenCalledWith("audio");
  });

  it("shows an empty workspace state when no tags exist", async () => {
    renderTagFilter({ allTags: [] });
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));

    expect(screen.getByText("No tags in workspace.")).toBeInTheDocument();
  });

  it("shows a no-results state when search yields no tags", async () => {
    renderTagFilter();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));
    await user.type(screen.getByPlaceholderText("Search tags"), "missing");

    expect(screen.getByText("No tags found.")).toBeInTheDocument();
  });

  it("renders Manage Tags and opens the dialog", async () => {
    renderTagFilter();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));
    await user.click(screen.getByRole("button", { name: /Manage Tags/i }));

    const dialog = await screen.findByRole("dialog");
    expect(within(dialog).getByText("Manage Tags")).toBeInTheDocument();
  });

  it("wires tag deletion through Manage Tags", async () => {
    const props = renderTagFilter({
      allTags: ["vision"],
    });
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Tags/i }));
    await user.click(screen.getByRole("button", { name: /Manage Tags/i }));

    const dialog = await screen.findByRole("dialog");
    await user.click(
      within(dialog).getByRole("button", { name: /Delete tag vision/i }),
    );

    await waitFor(() => {
      expect(props.onDeleteTag).toHaveBeenCalledWith("vision");
    });
  });
});
