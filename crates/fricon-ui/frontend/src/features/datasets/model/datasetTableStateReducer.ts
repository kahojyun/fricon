import type { SortingState } from "@tanstack/react-table";
import { DATASET_PAGE_SIZE, type DatasetStatus } from "../api/types";

export interface DatasetTableState {
  searchInput: string;
  appliedSearchQuery: string;
  activeTags: string[];
  activeStatuses: DatasetStatus[];
  showFavoritesOnly: boolean;
  sorting: SortingState;
  queryLimit: number;
}

export type DatasetTableStateAction =
  | { type: "set_search_input"; next: string }
  | { type: "commit_search_input" }
  | { type: "toggle_tag"; tag: string }
  | { type: "toggle_status"; status: DatasetStatus }
  | { type: "set_show_favorites_only"; next: boolean }
  | {
      type: "set_sorting";
      updater: SortingState | ((prev: SortingState) => SortingState);
    }
  | { type: "remove_active_tag"; tag: string }
  | { type: "replace_active_tag"; oldName: string; newName: string }
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

export function createInitialDatasetTableState(): DatasetTableState {
  return {
    searchInput: "",
    appliedSearchQuery: "",
    activeTags: [],
    activeStatuses: [],
    showFavoritesOnly: false,
    sorting: DEFAULT_SORTING,
    queryLimit: DATASET_PAGE_SIZE,
  };
}

function resetQueryLimit(state: DatasetTableState): DatasetTableState {
  return {
    ...state,
    queryLimit: DATASET_PAGE_SIZE,
  };
}

export function datasetTableStateReducer(
  state: DatasetTableState,
  action: DatasetTableStateAction,
): DatasetTableState {
  switch (action.type) {
    case "set_search_input": {
      if (action.next === state.searchInput) {
        return state;
      }

      return {
        ...state,
        searchInput: action.next,
      };
    }

    case "commit_search_input": {
      if (state.appliedSearchQuery === state.searchInput) {
        return state;
      }

      return {
        ...state,
        appliedSearchQuery: state.searchInput,
        queryLimit: DATASET_PAGE_SIZE,
      };
    }

    case "toggle_tag": {
      const nextActiveTags = state.activeTags.includes(action.tag)
        ? state.activeTags.filter((item) => item !== action.tag)
        : [...state.activeTags, action.tag];

      return resetQueryLimit({
        ...state,
        activeTags: nextActiveTags,
      });
    }

    case "toggle_status": {
      const nextActiveStatuses = state.activeStatuses.includes(action.status)
        ? state.activeStatuses.filter((item) => item !== action.status)
        : [...state.activeStatuses, action.status];

      return resetQueryLimit({
        ...state,
        activeStatuses: nextActiveStatuses,
      });
    }

    case "set_show_favorites_only": {
      if (action.next === state.showFavoritesOnly) {
        return state;
      }

      return resetQueryLimit({
        ...state,
        showFavoritesOnly: action.next,
      });
    }

    case "set_sorting": {
      const nextSorting = normalizeSortingState(action.updater, state.sorting);
      if (areSortingStatesEqual(nextSorting, state.sorting)) {
        return state;
      }

      return resetQueryLimit({
        ...state,
        sorting: nextSorting,
      });
    }

    case "remove_active_tag": {
      if (!state.activeTags.includes(action.tag)) {
        return state;
      }

      return resetQueryLimit({
        ...state,
        activeTags: state.activeTags.filter((item) => item !== action.tag),
      });
    }

    case "replace_active_tag": {
      if (
        !state.activeTags.includes(action.oldName) ||
        action.oldName === action.newName
      ) {
        return state;
      }

      const nextActiveTags = state.activeTags
        .map((item) => (item === action.oldName ? action.newName : item))
        .filter((item, index, values) => values.indexOf(item) === index);

      return resetQueryLimit({
        ...state,
        activeTags: nextActiveTags,
      });
    }

    case "clear_filters": {
      if (
        state.searchInput === "" &&
        state.appliedSearchQuery === "" &&
        state.activeTags.length === 0 &&
        state.activeStatuses.length === 0 &&
        !state.showFavoritesOnly
      ) {
        return state;
      }

      return {
        ...state,
        searchInput: "",
        appliedSearchQuery: "",
        activeTags: [],
        activeStatuses: [],
        showFavoritesOnly: false,
        queryLimit: DATASET_PAGE_SIZE,
      };
    }

    case "load_next_page":
      return {
        ...state,
        queryLimit: state.queryLimit + DATASET_PAGE_SIZE,
      };
  }
}
