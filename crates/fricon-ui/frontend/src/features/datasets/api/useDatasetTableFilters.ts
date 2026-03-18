import { useReducer } from "react";
import type { SortingState } from "@tanstack/react-table";
import type { DatasetStatus } from "./types";
import {
  createInitialDatasetTableFiltersState,
  datasetTableFiltersReducer,
} from "../model/datasetTableFiltersReducer";

interface DatasetTableFiltersResult {
  searchQuery: string;
  setSearchQuery: (next: string) => void;
  selectedTags: string[];
  selectedStatuses: DatasetStatus[];
  favoriteOnly: boolean;
  setFavoriteOnly: (next: boolean) => void;
  sorting: SortingState;
  setSorting: (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => void;
  visibleCount: number;
  searchTransitionVisibleCount: number | null;
  clearSearchTransition: () => void;
  loadNextPage: () => Promise<void>;
  handleTagToggle: (tag: string) => void;
  handleStatusToggle: (status: DatasetStatus) => void;
  removeSelectedTag: (tag: string) => void;
  replaceSelectedTag: (oldName: string, newName: string) => void;
  clearFilters: () => void;
  hasActiveFilters: boolean;
}

export function useDatasetTableFilters(): DatasetTableFiltersResult {
  const [state, dispatch] = useReducer(
    datasetTableFiltersReducer,
    undefined,
    createInitialDatasetTableFiltersState,
  );

  const handleSearchQueryChange = (next: string) => {
    dispatch({ type: "set_search_query", next });
  };

  const setSorting = (
    updater: SortingState | ((prev: SortingState) => SortingState),
  ) => {
    dispatch({ type: "set_sorting", updater });
  };

  const handleTagToggle = (tag: string) => {
    dispatch({ type: "toggle_tag", tag });
  };

  const handleStatusToggle = (status: DatasetStatus) => {
    dispatch({ type: "toggle_status", status });
  };

  const clearFilters = () => {
    dispatch({ type: "clear_filters" });
  };

  const removeSelectedTag = (tag: string) => {
    dispatch({ type: "remove_selected_tag", tag });
  };

  const replaceSelectedTag = (oldName: string, newName: string) => {
    dispatch({ type: "replace_selected_tag", oldName, newName });
  };

  const handleFavoriteOnlyChange = (next: boolean) => {
    dispatch({ type: "set_favorite_only", next });
  };

  const hasActiveFilters =
    state.searchQuery.trim().length > 0 ||
    state.favoriteOnly ||
    state.selectedTags.length > 0 ||
    state.selectedStatuses.length > 0;

  const loadNextPage = () => {
    dispatch({ type: "load_next_page" });
    return Promise.resolve();
  };

  const clearSearchTransition = () => {
    dispatch({ type: "clear_search_transition" });
  };

  return {
    searchQuery: state.searchQuery,
    setSearchQuery: handleSearchQueryChange,
    selectedTags: state.selectedTags,
    selectedStatuses: state.selectedStatuses,
    favoriteOnly: state.favoriteOnly,
    setFavoriteOnly: handleFavoriteOnlyChange,
    sorting: state.sorting,
    setSorting,
    visibleCount: state.visibleCount,
    searchTransitionVisibleCount: state.searchTransitionVisibleCount,
    clearSearchTransition,
    loadNextPage,
    handleTagToggle,
    handleStatusToggle,
    removeSelectedTag,
    replaceSelectedTag,
    clearFilters,
    hasActiveFilters,
  };
}
