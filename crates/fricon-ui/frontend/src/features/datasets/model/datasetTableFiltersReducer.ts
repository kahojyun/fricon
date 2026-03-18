import type { SortingState } from "@tanstack/react-table";
import { DATASET_PAGE_SIZE, type DatasetStatus } from "../api/types";

export interface DatasetTableFiltersState {
  searchQuery: string;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  sorting: SortingState;
  visibleCount: number;
  searchTransitionVisibleCount: number | null;
}

export type DatasetTableFiltersAction =
  | { type: "set_search_query"; next: string }
  | { type: "clear_search_transition" }
  | { type: "toggle_tag"; tag: string }
  | { type: "toggle_status"; status: DatasetStatus }
  | { type: "set_favorite_only"; next: boolean }
  | {
      type: "set_sorting";
      updater: SortingState | ((prev: SortingState) => SortingState);
    }
  | { type: "remove_selected_tag"; tag: string }
  | { type: "replace_selected_tag"; oldName: string; newName: string }
  | { type: "clear_filters" }
  | { type: "load_next_page" };

const DEFAULT_SORTING: SortingState = [{ id: "id", desc: true }];

function areSortingStatesEqual(a: SortingState, b: SortingState): boolean {
  if (a.length !== b.length) return false;
  return a.every((entry, index) => {
    const other = b[index];
    return entry.id === other?.id && entry.desc === other?.desc;
  });
}

function normalizeSortingState(
  updater: SortingState | ((prev: SortingState) => SortingState),
  current: SortingState,
): SortingState {
  const next = typeof updater === "function" ? updater(current) : updater;
  const first = next[0];
  return first ? [first] : [];
}

export function createInitialDatasetTableFiltersState(): DatasetTableFiltersState {
  return {
    searchQuery: "",
    selectedTags: [],
    selectedStatuses: [],
    favoriteOnly: false,
    sorting: DEFAULT_SORTING,
    visibleCount: DATASET_PAGE_SIZE,
    searchTransitionVisibleCount: null,
  };
}

function resetVisibleCount(
  state: DatasetTableFiltersState,
): DatasetTableFiltersState {
  return {
    ...state,
    visibleCount: DATASET_PAGE_SIZE,
    searchTransitionVisibleCount: null,
  };
}

export function datasetTableFiltersReducer(
  state: DatasetTableFiltersState,
  action: DatasetTableFiltersAction,
): DatasetTableFiltersState {
  switch (action.type) {
    case "set_search_query": {
      if (action.next === state.searchQuery) {
        return state;
      }

      return {
        ...state,
        searchQuery: action.next,
        visibleCount: DATASET_PAGE_SIZE,
        searchTransitionVisibleCount:
          state.searchTransitionVisibleCount ?? state.visibleCount,
      };
    }

    case "clear_search_transition": {
      if (state.searchTransitionVisibleCount === null) {
        return state;
      }

      return {
        ...state,
        searchTransitionVisibleCount: null,
      };
    }

    case "toggle_tag": {
      const nextSelectedTags = state.selectedTags.includes(action.tag)
        ? state.selectedTags.filter((item) => item !== action.tag)
        : [...state.selectedTags, action.tag];

      return resetVisibleCount({
        ...state,
        selectedTags: nextSelectedTags,
      });
    }

    case "toggle_status": {
      const nextSelectedStatuses = state.selectedStatuses.includes(
        action.status,
      )
        ? state.selectedStatuses.filter((item) => item !== action.status)
        : [...state.selectedStatuses, action.status];

      return resetVisibleCount({
        ...state,
        selectedStatuses: nextSelectedStatuses,
      });
    }

    case "set_favorite_only": {
      if (action.next === state.favoriteOnly) {
        return state;
      }

      return resetVisibleCount({
        ...state,
        favoriteOnly: action.next,
      });
    }

    case "set_sorting": {
      const nextSorting = normalizeSortingState(action.updater, state.sorting);
      if (areSortingStatesEqual(nextSorting, state.sorting)) {
        return state;
      }

      return resetVisibleCount({
        ...state,
        sorting: nextSorting,
      });
    }

    case "remove_selected_tag": {
      if (!state.selectedTags.includes(action.tag)) {
        return state;
      }

      return resetVisibleCount({
        ...state,
        selectedTags: state.selectedTags.filter((item) => item !== action.tag),
      });
    }

    case "replace_selected_tag": {
      if (
        !state.selectedTags.includes(action.oldName) ||
        action.oldName === action.newName
      ) {
        return state;
      }

      const nextSelectedTags = state.selectedTags
        .map((item) => (item === action.oldName ? action.newName : item))
        .filter((item, index, values) => values.indexOf(item) === index);

      return resetVisibleCount({
        ...state,
        selectedTags: nextSelectedTags,
      });
    }

    case "clear_filters": {
      if (
        state.searchQuery === "" &&
        state.selectedTags.length === 0 &&
        state.selectedStatuses.length === 0 &&
        !state.favoriteOnly
      ) {
        return state;
      }

      return {
        ...state,
        searchQuery: "",
        selectedTags: [],
        selectedStatuses: [],
        favoriteOnly: false,
        visibleCount: DATASET_PAGE_SIZE,
        searchTransitionVisibleCount:
          state.searchQuery === ""
            ? null
            : (state.searchTransitionVisibleCount ?? state.visibleCount),
      };
    }

    case "load_next_page":
      return {
        ...state,
        visibleCount: state.visibleCount + DATASET_PAGE_SIZE,
      };
  }
}
