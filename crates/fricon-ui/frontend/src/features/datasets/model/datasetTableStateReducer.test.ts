import { describe, expect, it } from "vitest";
import { DATASET_PAGE_SIZE } from "../api/types";
import {
  createInitialDatasetTableState,
  datasetTableStateReducer,
} from "./datasetTableStateReducer";

describe("datasetTableStateReducer", () => {
  it("updates the search input immediately and applies it later with a limit reset", () => {
    const expanded = {
      ...createInitialDatasetTableState(),
      queryLimit: 9,
    };

    const inputState = datasetTableStateReducer(expanded, {
      type: "set_search_input",
      next: "Alpha",
    });
    const appliedState = datasetTableStateReducer(inputState, {
      type: "commit_search_input",
    });

    expect(inputState.searchInput).toBe("Alpha");
    expect(inputState.appliedSearchQuery).toBe("");
    expect(inputState.queryLimit).toBe(9);

    expect(appliedState.searchInput).toBe("Alpha");
    expect(appliedState.appliedSearchQuery).toBe("Alpha");
    expect(appliedState.queryLimit).toBe(DATASET_PAGE_SIZE);
  });

  it("resets queryLimit for tag, status, favorites, and sorting changes", () => {
    const expanded = {
      ...createInitialDatasetTableState(),
      queryLimit: 9,
    };

    const afterTagToggle = datasetTableStateReducer(expanded, {
      type: "toggle_tag",
      tag: "vision",
    });
    const afterStatusToggle = datasetTableStateReducer(expanded, {
      type: "toggle_status",
      status: "Writing",
    });
    const afterFavoritesToggle = datasetTableStateReducer(expanded, {
      type: "set_show_favorites_only",
      next: true,
    });
    const afterSortingChange = datasetTableStateReducer(expanded, {
      type: "set_sorting",
      updater: [{ id: "name", desc: false }],
    });

    expect(afterTagToggle.queryLimit).toBe(DATASET_PAGE_SIZE);
    expect(afterTagToggle.activeTags).toEqual(["vision"]);

    expect(afterStatusToggle.queryLimit).toBe(DATASET_PAGE_SIZE);
    expect(afterStatusToggle.activeStatuses).toEqual(["Writing"]);

    expect(afterFavoritesToggle.queryLimit).toBe(DATASET_PAGE_SIZE);
    expect(afterFavoritesToggle.showFavoritesOnly).toBe(true);

    expect(afterSortingChange.queryLimit).toBe(DATASET_PAGE_SIZE);
    expect(afterSortingChange.sorting).toEqual([{ id: "name", desc: false }]);
  });

  it("returns the same state object for no-op actions", () => {
    const state = createInitialDatasetTableState();

    expect(
      datasetTableStateReducer(state, {
        type: "set_search_input",
        next: "",
      }),
    ).toBe(state);
    expect(
      datasetTableStateReducer(state, {
        type: "commit_search_input",
      }),
    ).toBe(state);
    expect(
      datasetTableStateReducer(state, {
        type: "set_show_favorites_only",
        next: false,
      }),
    ).toBe(state);
    expect(
      datasetTableStateReducer(state, {
        type: "set_sorting",
        updater: [{ id: "id", desc: true }],
      }),
    ).toBe(state);
    expect(
      datasetTableStateReducer(state, {
        type: "remove_active_tag",
        tag: "vision",
      }),
    ).toBe(state);
    expect(
      datasetTableStateReducer(state, {
        type: "replace_active_tag",
        oldName: "vision",
        newName: "audio",
      }),
    ).toBe(state);
    expect(
      datasetTableStateReducer(state, {
        type: "clear_filters",
      }),
    ).toBe(state);
  });

  it("clears filters while preserving sorting and resetting input and applied search", () => {
    const activeState = {
      ...createInitialDatasetTableState(),
      searchInput: "Alpha",
      appliedSearchQuery: "Alpha",
      activeTags: ["vision"],
      activeStatuses: ["Writing" as const],
      showFavoritesOnly: true,
      sorting: [{ id: "name", desc: false }],
      queryLimit: 9,
    };

    const nextState = datasetTableStateReducer(activeState, {
      type: "clear_filters",
    });

    expect(nextState.searchInput).toBe("");
    expect(nextState.appliedSearchQuery).toBe("");
    expect(nextState.activeTags).toEqual([]);
    expect(nextState.activeStatuses).toEqual([]);
    expect(nextState.showFavoritesOnly).toBe(false);
    expect(nextState.sorting).toEqual([{ id: "name", desc: false }]);
    expect(nextState.queryLimit).toBe(DATASET_PAGE_SIZE);
  });
});
