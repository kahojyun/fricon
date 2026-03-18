import { describe, expect, it } from "vitest";
import { DATASET_PAGE_SIZE } from "../api/types";
import {
  createInitialDatasetTableFiltersState,
  datasetTableFiltersReducer,
} from "./datasetTableFiltersReducer";

describe("datasetTableFiltersReducer", () => {
  it("resets visibleCount and preserves the pre-search limit across multiple search updates", () => {
    const expanded = {
      ...createInitialDatasetTableFiltersState(),
      visibleCount: 9,
    };

    const firstSearchState = datasetTableFiltersReducer(expanded, {
      type: "set_search_query",
      next: "A",
    });
    const secondSearchState = datasetTableFiltersReducer(firstSearchState, {
      type: "set_search_query",
      next: "Al",
    });

    expect(firstSearchState.searchQuery).toBe("A");
    expect(firstSearchState.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(firstSearchState.searchTransitionVisibleCount).toBe(9);
    expect(secondSearchState.searchQuery).toBe("Al");
    expect(secondSearchState.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(secondSearchState.searchTransitionVisibleCount).toBe(9);
  });

  it("resets visibleCount for tag, status, favorite, and sorting changes", () => {
    const expanded = {
      ...createInitialDatasetTableFiltersState(),
      visibleCount: 9,
      searchTransitionVisibleCount: 9,
    };

    const afterTagToggle = datasetTableFiltersReducer(expanded, {
      type: "toggle_tag",
      tag: "vision",
    });
    const afterStatusToggle = datasetTableFiltersReducer(expanded, {
      type: "toggle_status",
      status: "Writing",
    });
    const afterFavoriteToggle = datasetTableFiltersReducer(expanded, {
      type: "set_favorite_only",
      next: true,
    });
    const afterSortingChange = datasetTableFiltersReducer(expanded, {
      type: "set_sorting",
      updater: [{ id: "name", desc: false }],
    });

    expect(afterTagToggle.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(afterTagToggle.searchTransitionVisibleCount).toBeNull();
    expect(afterTagToggle.selectedTags).toEqual(["vision"]);

    expect(afterStatusToggle.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(afterStatusToggle.searchTransitionVisibleCount).toBeNull();
    expect(afterStatusToggle.selectedStatuses).toEqual(["Writing"]);

    expect(afterFavoriteToggle.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(afterFavoriteToggle.searchTransitionVisibleCount).toBeNull();
    expect(afterFavoriteToggle.favoriteOnly).toBe(true);

    expect(afterSortingChange.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(afterSortingChange.searchTransitionVisibleCount).toBeNull();
    expect(afterSortingChange.sorting).toEqual([{ id: "name", desc: false }]);
  });

  it("returns the same state object for no-op actions", () => {
    const state = createInitialDatasetTableFiltersState();

    expect(
      datasetTableFiltersReducer(state, {
        type: "set_search_query",
        next: "",
      }),
    ).toBe(state);
    expect(
      datasetTableFiltersReducer(state, {
        type: "set_favorite_only",
        next: false,
      }),
    ).toBe(state);
    expect(
      datasetTableFiltersReducer(state, {
        type: "set_sorting",
        updater: [{ id: "id", desc: true }],
      }),
    ).toBe(state);
    expect(
      datasetTableFiltersReducer(state, {
        type: "remove_selected_tag",
        tag: "vision",
      }),
    ).toBe(state);
    expect(
      datasetTableFiltersReducer(state, {
        type: "replace_selected_tag",
        oldName: "vision",
        newName: "audio",
      }),
    ).toBe(state);
    expect(
      datasetTableFiltersReducer(state, {
        type: "clear_filters",
      }),
    ).toBe(state);
    expect(
      datasetTableFiltersReducer(state, {
        type: "clear_search_transition",
      }),
    ).toBe(state);
  });

  it("clears filters only when active, preserves sorting, and preserves search transition carry when needed", () => {
    const activeState = {
      ...createInitialDatasetTableFiltersState(),
      searchQuery: "Alpha",
      selectedTags: ["vision"],
      favoriteOnly: true,
      sorting: [{ id: "name", desc: false }],
      visibleCount: 3,
      searchTransitionVisibleCount: 9,
    };

    const nextState = datasetTableFiltersReducer(activeState, {
      type: "clear_filters",
    });

    expect(nextState.searchQuery).toBe("");
    expect(nextState.selectedTags).toEqual([]);
    expect(nextState.selectedStatuses).toEqual([]);
    expect(nextState.favoriteOnly).toBe(false);
    expect(nextState.sorting).toEqual([{ id: "name", desc: false }]);
    expect(nextState.visibleCount).toBe(DATASET_PAGE_SIZE);
    expect(nextState.searchTransitionVisibleCount).toBe(9);
  });

  it("clears the search transition carry once the debounce has settled", () => {
    const state = {
      ...createInitialDatasetTableFiltersState(),
      searchQuery: "Alpha",
      searchTransitionVisibleCount: 9,
    };

    const nextState = datasetTableFiltersReducer(state, {
      type: "clear_search_transition",
    });

    expect(nextState.searchTransitionVisibleCount).toBeNull();
    expect(nextState.searchQuery).toBe("Alpha");
  });
});
