import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CreateTopicDialog } from "../CreateTopicDialog";
import { useAppStore } from "@/stores/appStore";
import { makeBotFixture } from "@/test/fixtures";

const initialState = useAppStore.getState();

beforeEach(() => {
  useAppStore.setState(initialState, true);
});

describe("CreateTopicDialog", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    onSubmit: vi.fn(),
  };

  // UT-COMP-19: CreateTopicDialog shows bot checkboxes
  it("UT-COMP-19: shows all bots listed with checkboxes", () => {
    const bots = [
      makeBotFixture({ id: "bot-1", name: "GPT-4o", model: "gpt-4o" }),
      makeBotFixture({ id: "bot-2", name: "Claude", model: "claude-3.5-sonnet" }),
      makeBotFixture({ id: "bot-3", name: "Gemini", model: "gemini-pro" }),
    ];
    useAppStore.setState({ bots });

    render(<CreateTopicDialog {...defaultProps} />);

    // All bot names should be visible
    expect(screen.getByText("GPT-4o")).toBeInTheDocument();
    expect(screen.getByText("Claude")).toBeInTheDocument();
    expect(screen.getByText("Gemini")).toBeInTheDocument();

    // Each bot should have a checkbox
    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes).toHaveLength(3);

    // All checkboxes should be unchecked initially
    checkboxes.forEach((cb) => {
      expect(cb).not.toBeChecked();
    });
  });

  // UT-COMP-20: CreateTopicDialog disables submit without title or bots
  it("UT-COMP-20: disables submit button when title is empty", () => {
    const bots = [
      makeBotFixture({ id: "bot-1", name: "GPT-4o" }),
    ];
    useAppStore.setState({ bots });

    render(<CreateTopicDialog {...defaultProps} />);

    const submitButton = screen.getByText("Create Topic");
    expect(submitButton).toBeDisabled();
  });

  it("UT-COMP-20: disables submit button when no bots selected", async () => {
    const user = userEvent.setup();
    const bots = [
      makeBotFixture({ id: "bot-1", name: "GPT-4o" }),
    ];
    useAppStore.setState({ bots });

    render(<CreateTopicDialog {...defaultProps} />);

    // Type a title but don't select any bots
    const titleInput = screen.getByLabelText("Title");
    await user.type(titleInput, "My Topic");

    const submitButton = screen.getByText("Create Topic");
    expect(submitButton).toBeDisabled();
  });

  it("UT-COMP-20: enables submit button when title and bots are provided", async () => {
    const user = userEvent.setup();
    const bots = [
      makeBotFixture({ id: "bot-1", name: "GPT-4o" }),
      makeBotFixture({ id: "bot-2", name: "Claude" }),
    ];
    useAppStore.setState({ bots });

    render(<CreateTopicDialog {...defaultProps} />);

    // Type a title
    const titleInput = screen.getByLabelText("Title");
    await user.type(titleInput, "My Topic");

    // Select a bot
    const checkboxes = screen.getAllByRole("checkbox");
    await user.click(checkboxes[0]);

    const submitButton = screen.getByText("Create Topic");
    expect(submitButton).toBeEnabled();
  });

  it("shows 'No bots available' when bot list is empty", () => {
    useAppStore.setState({ bots: [] });

    render(<CreateTopicDialog {...defaultProps} />);

    expect(screen.getByText(/No bots available/i)).toBeInTheDocument();
  });
});
