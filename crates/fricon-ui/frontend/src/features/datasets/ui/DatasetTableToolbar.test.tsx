import type { ComponentProps } from "react";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { DatasetTableToolbar } from "./DatasetTableToolbar";
import { DatasetTableViewOptions } from "./DatasetTableViewOptions";
import { createMockTable, openColumnsMenu } from "./test-utils";

function renderToolbar(
  overrides: Partial<ComponentProps<typeof DatasetTableToolbar>> = {},
) {
  const props = {
    table: createMockTable(),
    hasActiveFilters: false,
    selectedTags: [],
    selectedStatuses: [],
    favoriteOnly: false,
    searchQuery: "",
    allTags: ["vision", "audio"],
    isUpdatingTags: false,
    setFavoriteOnly: vi.fn(),
    setSearchQuery: vi.fn(),
    handleTagToggle: vi.fn(),
    handleStatusToggle: vi.fn(),
    clearFilters: vi.fn(),
    resetColumnVisibilityToDefault: vi.fn(),
    showAllColumns: vi.fn(),
    onColumnVisibilityChange: vi.fn(),
    onDeleteTag: vi.fn().mockResolvedValue(undefined),
    onRenameTag: vi.fn().mockResolvedValue(undefined),
    onMergeTag: vi.fn().mockResolvedValue(undefined),
    ...overrides,
  };

  render(<DatasetTableToolbar {...props} />);
  return props;
}

function renderViewOptions(
  overrides: Partial<ComponentProps<typeof DatasetTableViewOptions>> = {},
) {
  const props = {
    table: createMockTable(),
    resetColumnVisibilityToDefault: vi.fn(),
    showAllColumns: vi.fn(),
    onColumnVisibilityChange: vi.fn(),
    ...overrides,
  };

  render(<DatasetTableViewOptions {...props} />);
  return props;
}

describe("DatasetTableToolbar", () => {
  beforeEach(() => {
    Element.prototype.scrollIntoView = vi.fn();
  });

  it("calls setSearchQuery from the search input", async () => {
    const props = renderToolbar();
    const user = userEvent.setup();

    await user.type(screen.getByPlaceholderText("Filter datasets..."), "Alpha");

    await waitFor(() => {
      expect(props.setSearchQuery).toHaveBeenCalled();
    });
  });

  it("calls setFavoriteOnly from the favorites toggle", async () => {
    const props = renderToolbar();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Favorites Only/i }));

    expect(props.setFavoriteOnly).toHaveBeenCalledWith(true, expect.any(Object));
  });

  it("shows the reset button only when filters are active", () => {
    const { rerender } = render(
      <DatasetTableToolbar
        table={createMockTable()}
        hasActiveFilters={false}
        selectedTags={[]}
        selectedStatuses={[]}
        favoriteOnly={false}
        searchQuery=""
        allTags={["vision"]}
        isUpdatingTags={false}
        setFavoriteOnly={vi.fn()}
        setSearchQuery={vi.fn()}
        handleTagToggle={vi.fn()}
        handleStatusToggle={vi.fn()}
        clearFilters={vi.fn()}
        resetColumnVisibilityToDefault={vi.fn()}
        showAllColumns={vi.fn()}
        onColumnVisibilityChange={vi.fn()}
        onDeleteTag={vi.fn().mockResolvedValue(undefined)}
        onRenameTag={vi.fn().mockResolvedValue(undefined)}
        onMergeTag={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(
      screen.queryByRole("button", { name: "Reset" }),
    ).not.toBeInTheDocument();

    rerender(
      <DatasetTableToolbar
        table={createMockTable()}
        hasActiveFilters
        selectedTags={["vision"]}
        selectedStatuses={[]}
        favoriteOnly={false}
        searchQuery="Alpha"
        allTags={["vision"]}
        isUpdatingTags={false}
        setFavoriteOnly={vi.fn()}
        setSearchQuery={vi.fn()}
        handleTagToggle={vi.fn()}
        handleStatusToggle={vi.fn()}
        clearFilters={vi.fn()}
        resetColumnVisibilityToDefault={vi.fn()}
        showAllColumns={vi.fn()}
        onColumnVisibilityChange={vi.fn()}
        onDeleteTag={vi.fn().mockResolvedValue(undefined)}
        onRenameTag={vi.fn().mockResolvedValue(undefined)}
        onMergeTag={vi.fn().mockResolvedValue(undefined)}
      />,
    );

    expect(screen.getByRole("button", { name: "Reset" })).toBeInTheDocument();
  });

  it("calls clearFilters from the reset button", async () => {
    const props = renderToolbar({
      hasActiveFilters: true,
      selectedTags: ["vision"],
      searchQuery: "Alpha",
    });
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: "Reset" }));

    expect(props.clearFilters).toHaveBeenCalledTimes(1);
  });

  it("delegates status selection changes", async () => {
    const props = renderToolbar();
    const user = userEvent.setup();

    await user.click(screen.getByRole("button", { name: /Completed/i }));

    expect(props.handleStatusToggle).toHaveBeenCalledWith("Completed");
  });

  it("renders tag filter and view option triggers", () => {
    renderToolbar();

    expect(screen.getByRole("button", { name: /Tags/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /View/i })).toBeInTheDocument();
  });
});

describe("DatasetTableViewOptions", () => {
  it("keeps non-hideable columns disabled", async () => {
    renderViewOptions();
    const user = userEvent.setup();

    const menu = await openColumnsMenu(user);
    const nameCheckbox = within(menu).getByRole("menuitemcheckbox", {
      name: "Name",
    });

    expect(nameCheckbox).toHaveAttribute("aria-disabled", "true");
  });

  it("calls onColumnVisibilityChange for hideable columns", async () => {
    const props = renderViewOptions({
      table: createMockTable([{ id: "tags", label: "Tags", visible: true }]),
    });
    const user = userEvent.setup();

    const menu = await openColumnsMenu(user);
    await user.click(
      within(menu).getByRole("menuitemcheckbox", { name: "Tags" }),
    );

    expect(props.onColumnVisibilityChange).toHaveBeenCalledWith("tags", false);
  });

  it("calls show all and reset default actions", async () => {
    const props = renderViewOptions();
    const user = userEvent.setup();

    let menu = await openColumnsMenu(user);
    await user.click(within(menu).getByRole("menuitem", { name: /Show All/i }));

    expect(props.showAllColumns).toHaveBeenCalledTimes(1);

    menu = await openColumnsMenu(user);
    await user.click(
      within(menu).getByRole("menuitem", { name: /Reset Default/i }),
    );

    expect(props.resetColumnVisibilityToDefault).toHaveBeenCalledTimes(1);
  });
});
