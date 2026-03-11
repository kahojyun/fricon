import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { initialCascadeState } from "../model/cascadeReducer";
import { useCascadeSelection } from "./useCascadeSelection";

describe("useCascadeSelection", () => {
  it("ignores field selection when data is unavailable", () => {
    const { result } = renderHook(() => useCascadeSelection(undefined));

    act(() => {
      result.current.selectFieldValue(0, 0, null);
    });

    expect(result.current.state).toEqual(initialCascadeState);
    expect(result.current.resolvedRow).toBeUndefined();
    expect(result.current.selectedValueIndices).toBeUndefined();
  });
});
