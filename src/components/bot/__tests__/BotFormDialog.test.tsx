import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { BotFormDialog } from "../BotFormDialog";
import { makeBotFixture } from "@/test/fixtures";

describe("BotFormDialog", () => {
  const defaultProps = {
    open: true,
    onOpenChange: vi.fn(),
    onSubmit: vi.fn(),
  };

  // UT-COMP-17: BotFormDialog pre-fills fields when editing
  it("UT-COMP-17: pre-fills all fields when editBot is provided", () => {
    const editBot = makeBotFixture({
      name: "GPT-4o",
      base_url: "https://api.openai.com/v1",
      api_key: "sk-secret-key",
      model: "gpt-4o",
      system_prompt: "You are helpful.",
      avatar_color: "#ec4899",
      supports_vision: true,
    });

    render(<BotFormDialog {...defaultProps} editBot={editBot} />);

    // Title should say "Edit Bot"
    expect(screen.getByText("Edit Bot")).toBeInTheDocument();

    // Check name field
    const nameInput = screen.getByLabelText("Name") as HTMLInputElement;
    expect(nameInput.value).toBe("GPT-4o");

    // Check base URL field
    const urlInput = screen.getByLabelText("Base URL") as HTMLInputElement;
    expect(urlInput.value).toBe("https://api.openai.com/v1");

    // Check API key field
    const apiKeyInput = screen.getByLabelText("API Key") as HTMLInputElement;
    expect(apiKeyInput.value).toBe("sk-secret-key");

    // Check model field
    const modelInput = screen.getByLabelText("Model") as HTMLInputElement;
    expect(modelInput.value).toBe("gpt-4o");

    // Check system prompt field
    const promptInput = screen.getByLabelText("System Prompt") as HTMLTextAreaElement;
    expect(promptInput.value).toBe("You are helpful.");

    // Check supports vision checkbox
    const visionCheckbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(visionCheckbox.checked).toBe(true);

    // Submit button should say "Save Changes"
    expect(screen.getByText("Save Changes")).toBeInTheDocument();
  });

  // UT-COMP-18: BotFormDialog empty fields when creating
  it("UT-COMP-18: shows empty/default fields when editBot is null", () => {
    render(<BotFormDialog {...defaultProps} editBot={null} />);

    // "Add Bot" appears as both the dialog title and submit button
    const addBotElements = screen.getAllByText("Add Bot");
    expect(addBotElements.length).toBeGreaterThanOrEqual(2);

    // All text fields should be empty
    const nameInput = screen.getByLabelText("Name") as HTMLInputElement;
    expect(nameInput.value).toBe("");

    const urlInput = screen.getByLabelText("Base URL") as HTMLInputElement;
    expect(urlInput.value).toBe("");

    const apiKeyInput = screen.getByLabelText("API Key") as HTMLInputElement;
    expect(apiKeyInput.value).toBe("");

    const modelInput = screen.getByLabelText("Model") as HTMLInputElement;
    expect(modelInput.value).toBe("");

    const promptInput = screen.getByLabelText("System Prompt") as HTMLTextAreaElement;
    expect(promptInput.value).toBe("");

    // Supports vision should be unchecked
    const visionCheckbox = screen.getByRole("checkbox") as HTMLInputElement;
    expect(visionCheckbox.checked).toBe(false);

    // Submit button should say "Add Bot" and be of type submit
    const buttons = screen.getAllByRole("button");
    const submitButton = buttons.find(
      (b) => b.textContent === "Add Bot" && b.getAttribute("type") === "submit",
    );
    expect(submitButton).toBeTruthy();
  });
});
