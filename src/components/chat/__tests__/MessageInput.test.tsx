import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MessageInput } from "../MessageInput";
import { makeBotFixture } from "@/test/fixtures";

// Mock crypto.randomUUID for file IDs
beforeEach(() => {
  let counter = 0;
  vi.stubGlobal("crypto", {
    randomUUID: () => `uuid-${++counter}`,
  });
});

describe("MessageInput", () => {
  const bots = [
    makeBotFixture({ id: "bot-1", name: "GPT4o", avatar_color: "#6366f1" }),
    makeBotFixture({ id: "bot-2", name: "Claude", avatar_color: "#ec4899" }),
    makeBotFixture({ id: "bot-3", name: "Gemini", avatar_color: "#f97316" }),
  ];

  const defaultProps = {
    onSend: vi.fn(),
    disabled: false,
    bots,
  };

  // UT-COMP-10: MessageInput sends on Enter (not Shift+Enter)
  it("UT-COMP-10: calls onSend on Enter, does not call on Shift+Enter", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} onSend={onSend} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);

    // Type a message
    await user.type(textarea, "Hello world");

    // Shift+Enter should NOT send
    await user.keyboard("{Shift>}{Enter}{/Shift}");
    expect(onSend).not.toHaveBeenCalled();

    // Enter should send
    await user.keyboard("{Enter}");
    expect(onSend).toHaveBeenCalledOnce();
    expect(onSend).toHaveBeenCalledWith("Hello world", []);
  });

  it("UT-COMP-10: does not send when text is empty", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} onSend={onSend} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);
    await user.click(textarea);
    await user.keyboard("{Enter}");

    expect(onSend).not.toHaveBeenCalled();
  });

  // UT-COMP-11: MessageInput shows @mention dropdown on @
  it("UT-COMP-11: shows @mention dropdown when @ is typed", async () => {
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);
    await user.type(textarea, "@");

    // All bots should appear in the dropdown
    expect(screen.getByText("GPT4o")).toBeInTheDocument();
    expect(screen.getByText("Claude")).toBeInTheDocument();
    expect(screen.getByText("Gemini")).toBeInTheDocument();
  });

  // UT-COMP-12: MessageInput filters @mention list by typed text
  it("UT-COMP-12: filters @mention list by typed text after @", async () => {
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);
    await user.type(textarea, "@Cl");

    // Only Claude should match
    expect(screen.getByText("Claude")).toBeInTheDocument();
    expect(screen.queryByText("GPT4o")).not.toBeInTheDocument();
    expect(screen.queryByText("Gemini")).not.toBeInTheDocument();
  });

  // UT-COMP-13: MessageInput inserts bot name on @mention select
  it("UT-COMP-13: inserts bot name on @mention select via mouseDown", async () => {
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);
    await user.type(textarea, "Hey @Cl");

    // The dropdown should show Claude
    const claudeButton = screen.getByText("Claude").closest("button");
    expect(claudeButton).toBeInTheDocument();

    // Use mouseDown (not click) as the component uses onMouseDown
    fireEvent.mouseDown(claudeButton!);

    // The textarea should contain the inserted mention
    expect(textarea).toHaveValue("Hey @Claude ");
  });

  // UT-COMP-14: MessageInput shows file previews
  it("UT-COMP-14: shows file previews with remove button", async () => {
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} />);

    // Create a mock file and trigger file input change
    const file = new File(["content"], "test-doc.pdf", {
      type: "application/pdf",
    });
    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    expect(fileInput).toBeTruthy();

    // Simulate file selection
    fireEvent.change(fileInput, { target: { files: [file] } });

    // File preview should show the filename
    expect(screen.getByText("test-doc.pdf")).toBeInTheDocument();

    // The file name <span> is inside a parent <span> wrapper that also contains the remove <button>
    const fileNameEl = screen.getByText("test-doc.pdf");
    // Go up to the parent wrapper span (the one with inline-flex class)
    const wrapper = fileNameEl.parentElement!;
    const removeButton = wrapper.querySelector("button");
    expect(removeButton).toBeInTheDocument();

    // Click remove button
    await user.click(removeButton!);

    // File should be gone
    expect(screen.queryByText("test-doc.pdf")).not.toBeInTheDocument();
  });

  // UT-COMP-15: MessageInput disabled during streaming
  it("UT-COMP-15: disables input and send button when disabled=true", () => {
    render(<MessageInput {...defaultProps} disabled={true} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);
    expect(textarea).toBeDisabled();

    // The send button (last button in the row) should be disabled
    const buttons = screen.getAllByRole("button");
    const sendButton = buttons[buttons.length - 1];
    expect(sendButton).toBeDisabled();
  });

  it("clears text after sending", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();

    render(<MessageInput {...defaultProps} onSend={onSend} />);

    const textarea = screen.getByPlaceholderText(/Type a message/i);
    await user.type(textarea, "Message to send");
    await user.keyboard("{Enter}");

    expect(textarea).toHaveValue("");
  });
});
