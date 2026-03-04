import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BotCard } from "../BotCard";
import { makeBotFixture } from "@/test/fixtures";

describe("BotCard", () => {
  const defaultProps = {
    onEdit: vi.fn(),
    onDelete: vi.fn(),
  };

  // UT-COMP-16: BotCard displays bot info
  it("UT-COMP-16: displays bot name, model, avatar color, and vision badge", () => {
    const bot = makeBotFixture({
      name: "GPT-4o",
      model: "gpt-4o",
      avatar_color: "#6366f1",
      supports_vision: true,
    });

    render(<BotCard bot={bot} {...defaultProps} />);

    // Bot name
    expect(screen.getByText("GPT-4o")).toBeInTheDocument();

    // Model name
    expect(screen.getByText("gpt-4o")).toBeInTheDocument();

    // Avatar with first letter and correct color
    const avatar = screen.getByText("G");
    expect(avatar).toBeInTheDocument();
    expect(avatar).toHaveStyle({ backgroundColor: "#6366f1" });

    // Vision badge present when supports_vision=true
    expect(screen.getByText("Vision")).toBeInTheDocument();
  });

  it("UT-COMP-16: does not show Vision badge when supports_vision=false", () => {
    const bot = makeBotFixture({
      name: "Claude",
      supports_vision: false,
    });

    render(<BotCard bot={bot} {...defaultProps} />);

    expect(screen.queryByText("Vision")).not.toBeInTheDocument();
  });

  it("calls onEdit when edit button is clicked", async () => {
    const user = userEvent.setup();
    const onEdit = vi.fn();
    const bot = makeBotFixture({ name: "TestBot" });

    render(<BotCard bot={bot} onEdit={onEdit} onDelete={vi.fn()} />);

    // The edit button is the first icon button (Pencil icon)
    const buttons = screen.getAllByRole("button");
    await user.click(buttons[0]); // edit button

    expect(onEdit).toHaveBeenCalledOnce();
    expect(onEdit).toHaveBeenCalledWith(bot);
  });

  it("calls onDelete when delete button is clicked", async () => {
    const user = userEvent.setup();
    const onDelete = vi.fn();
    const bot = makeBotFixture({ name: "TestBot" });

    render(<BotCard bot={bot} onEdit={vi.fn()} onDelete={onDelete} />);

    // The delete button is the second icon button (Trash icon)
    const buttons = screen.getAllByRole("button");
    await user.click(buttons[1]); // delete button

    expect(onDelete).toHaveBeenCalledOnce();
    expect(onDelete).toHaveBeenCalledWith(bot);
  });
});
