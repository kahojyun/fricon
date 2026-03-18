import * as React from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ManageTagsDialog } from "./ManageTagsDialog";

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

vi.mock("@/shared/ui/select", async () => {
  const react = await import("react");

  interface SelectContextValue {
    disabled?: boolean;
    onValueChange?: (value: string) => void;
    value?: string;
  }

  const SelectContext = react.createContext<SelectContextValue | null>(null);

  function Select({
    value,
    onValueChange,
    disabled,
    children,
  }: React.PropsWithChildren<SelectContextValue>) {
    return react.createElement(
      SelectContext.Provider,
      { value: { value, onValueChange, disabled } },
      children,
    );
  }

  function SelectTrigger({
    children,
    className,
  }: React.PropsWithChildren<{ className?: string }>) {
    const context = react.use(SelectContext);
    return react.createElement(
      "button",
      {
        type: "button",
        className,
        disabled: context?.disabled,
      },
      children,
    );
  }

  function SelectValue({ placeholder }: { placeholder?: string }) {
    const context = react.use(SelectContext);
    return react.createElement(
      "span",
      null,
      context?.value && context.value.length > 0 ? context.value : placeholder,
    );
  }

  function SelectContent({ children }: React.PropsWithChildren) {
    return react.createElement("div", null, children);
  }

  function SelectItem({
    value,
    children,
    className,
  }: React.PropsWithChildren<{ value: string; className?: string }>) {
    const context = react.use(SelectContext);
    return react.createElement(
      "button",
      {
        type: "button",
        className,
        disabled: context?.disabled,
        onClick: () => context?.onValueChange?.(value),
      },
      children,
    );
  }

  return {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
  };
});

function renderDialog() {
  const props = {
    allTags: ["vision", "audio", "archive"],
    isUpdatingTags: false,
    onDeleteTag: vi.fn().mockResolvedValue(undefined),
    onRenameTag: vi.fn().mockResolvedValue(undefined),
    onMergeTag: vi.fn().mockResolvedValue(undefined),
  };

  render(<ManageTagsDialog {...props} />);
  return props;
}

async function openDialog(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole("button", { name: /Manage Tags/i }));
  await screen.findByRole("dialog");
}

describe("ManageTagsDialog", () => {
  beforeEach(() => {
    toastSuccess.mockReset();
    toastError.mockReset();
  });

  it("closes a rename action without mutating when the name is unchanged", async () => {
    const props = renderDialog();
    const user = userEvent.setup();

    await openDialog(user);
    await user.click(
      screen.getByRole("button", { name: /Rename tag vision/i }),
    );
    await user.click(screen.getByRole("button", { name: "Confirm rename" }));

    expect(props.onRenameTag).not.toHaveBeenCalled();
    expect(
      screen.queryByRole("button", { name: "Confirm rename" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /Rename tag vision/i }),
    ).toBeVisible();
  });

  it("rejects duplicate tag renames locally", async () => {
    const props = renderDialog();
    const user = userEvent.setup();

    await openDialog(user);
    await user.click(
      screen.getByRole("button", { name: /Rename tag vision/i }),
    );

    const input = screen.getByDisplayValue("vision");
    await user.clear(input);
    await user.type(input, "audio");
    await user.click(screen.getByRole("button", { name: "Confirm rename" }));

    expect(props.onRenameTag).not.toHaveBeenCalled();
    expect(toastError).toHaveBeenCalledWith(
      'A tag named "audio" already exists.',
    );
    expect(
      screen.getByRole("button", { name: "Confirm rename" }),
    ).toBeVisible();
  });

  it("requires a merge target and calls the merge mutation once selected", async () => {
    const props = renderDialog();
    const user = userEvent.setup();

    await openDialog(user);
    await user.click(screen.getByRole("button", { name: /Merge tag vision/i }));

    const confirmMerge = screen.getByRole("button", { name: "Confirm merge" });
    expect(confirmMerge).toBeDisabled();

    await user.click(screen.getByRole("button", { name: /Pick target/i }));
    await user.click(screen.getByRole("button", { name: "archive" }));

    expect(confirmMerge).toBeEnabled();

    await user.click(confirmMerge);

    await waitFor(() => {
      expect(props.onMergeTag).toHaveBeenCalledWith("vision", "archive");
    });
    expect(toastSuccess).toHaveBeenCalledWith(
      'Tag "vision" merged into "archive".',
    );
  });

  it("shows toast feedback for rename success and merge failure", async () => {
    const props = renderDialog();
    props.onMergeTag.mockRejectedValueOnce(new Error("merge failed"));
    const user = userEvent.setup();

    await openDialog(user);

    await user.click(
      screen.getByRole("button", { name: /Rename tag vision/i }),
    );
    const input = screen.getByDisplayValue("vision");
    await user.clear(input);
    await user.type(input, "images");
    await user.click(screen.getByRole("button", { name: "Confirm rename" }));

    await waitFor(() => {
      expect(props.onRenameTag).toHaveBeenCalledWith("vision", "images");
    });
    expect(toastSuccess).toHaveBeenCalledWith('Tag renamed to "images".');

    await user.click(screen.getByRole("button", { name: /Merge tag audio/i }));
    await user.click(screen.getByRole("button", { name: /Pick target/i }));
    await user.click(screen.getByRole("button", { name: "archive" }));
    await user.click(screen.getByRole("button", { name: "Confirm merge" }));

    await waitFor(() => {
      expect(props.onMergeTag).toHaveBeenCalledWith("audio", "archive");
    });
    expect(toastError).toHaveBeenCalledWith("merge failed");
  });

  it("disables repeated actions while a tag mutation is pending", async () => {
    let resolveRename: (() => void) | undefined;
    const props = renderDialog();
    props.onRenameTag.mockImplementation(
      () =>
        new Promise<void>((resolve) => {
          resolveRename = resolve;
        }),
    );
    const user = userEvent.setup();

    await openDialog(user);
    await user.click(
      screen.getByRole("button", { name: /Rename tag vision/i }),
    );

    const input = screen.getByDisplayValue("vision");
    await user.clear(input);
    await user.type(input, "images");
    await user.click(screen.getByRole("button", { name: "Confirm rename" }));

    await waitFor(() => {
      expect(props.onRenameTag).toHaveBeenCalledWith("vision", "images");
    });

    expect(screen.getByDisplayValue("images")).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Confirm rename" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "Cancel rename" }),
    ).toBeDisabled();

    resolveRename?.();

    await waitFor(() => {
      expect(
        screen.queryByRole("button", { name: "Confirm rename" }),
      ).not.toBeInTheDocument();
    });
  });
});
