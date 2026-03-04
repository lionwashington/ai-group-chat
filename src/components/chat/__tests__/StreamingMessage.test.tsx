import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { StreamingMessage } from "../StreamingMessage";
import { makeBotFixture, makeStreamingStateFixture } from "@/test/fixtures";

// Mock react-markdown to render plain text
vi.mock("react-markdown", () => ({
  default: ({ children }: { children: string }) => <span>{children}</span>,
}));

vi.mock("remark-gfm", () => ({ default: () => {} }));
vi.mock("rehype-highlight", () => ({ default: () => {} }));

describe("StreamingMessage", () => {
  const bot = makeBotFixture({
    id: "bot-1",
    name: "TestBot",
    avatar_color: "#6366f1",
  });
  const bots = [bot];

  // UT-COMP-07: StreamingMessage shows cursor animation while streaming
  it("UT-COMP-07: shows pulsing cursor when done=false", () => {
    const state = makeStreamingStateFixture({
      content: "Partial response...",
      done: false,
    });

    const { container } = render(
      <StreamingMessage state={state} bots={bots} />,
    );

    // The pulsing cursor is a span with animate-pulse class
    const cursor = container.querySelector(".animate-pulse");
    expect(cursor).toBeInTheDocument();
  });

  it("UT-COMP-07: shows 'Thinking...' when no content and not done", () => {
    const state = makeStreamingStateFixture({
      content: "",
      done: false,
    });

    render(<StreamingMessage state={state} bots={bots} />);

    expect(screen.getByText("Thinking...")).toBeInTheDocument();
  });

  // UT-COMP-08: StreamingMessage shows error message
  it("UT-COMP-08: shows error text in red when error is set", () => {
    const state = makeStreamingStateFixture({
      content: "",
      done: true,
      error: "Rate limit exceeded",
    });

    render(<StreamingMessage state={state} bots={bots} />);

    const errorEl = screen.getByText("Rate limit exceeded");
    expect(errorEl).toBeInTheDocument();
    expect(errorEl.className).toContain("text-destructive");
  });

  // UT-COMP-09: StreamingMessage removes cursor when done
  it("UT-COMP-09: removes cursor when done=true", () => {
    const state = makeStreamingStateFixture({
      content: "Complete response",
      done: true,
    });

    const { container } = render(
      <StreamingMessage state={state} bots={bots} />,
    );

    // No cursor should be present when done
    const cursor = container.querySelector(".animate-pulse");
    expect(cursor).not.toBeInTheDocument();
  });

  it("displays bot name and avatar", () => {
    const state = makeStreamingStateFixture({
      botId: "bot-1",
      botName: "TestBot",
      content: "Hello",
      done: false,
    });

    render(<StreamingMessage state={state} bots={bots} />);

    expect(screen.getByText("TestBot")).toBeInTheDocument();
    expect(screen.getByText("T")).toBeInTheDocument(); // first letter avatar
  });
});
