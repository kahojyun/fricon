import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { DatasetNewTagForm } from "./DatasetTableTagMenu";

describe("DatasetTableTagMenu", () => {
  it("trims typed input, submits the tag, and clears the form", async () => {
    const user = userEvent.setup();
    const onSubmitTag = vi.fn();

    render(<DatasetNewTagForm onSubmitTag={onSubmitTag} />);

    const input = screen.getByPlaceholderText("New tag...");
    await user.type(input, "  fresh-tag  ");
    await user.keyboard("{Enter}");

    expect(onSubmitTag).toHaveBeenCalledWith("fresh-tag");
    expect(input).toHaveValue("");
  });

  it("ignores blank tag submissions", async () => {
    const user = userEvent.setup();
    const onSubmitTag = vi.fn();

    render(<DatasetNewTagForm onSubmitTag={onSubmitTag} />);

    const input = screen.getByPlaceholderText("New tag...");
    await user.type(input, "   ");
    await user.keyboard("{Enter}");

    expect(onSubmitTag).not.toHaveBeenCalled();
    expect(input).toHaveValue("   ");
  });
});
