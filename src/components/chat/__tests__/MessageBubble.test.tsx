import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { MessageBubble } from "../MessageBubble";
import {
  makeBotFixture,
  makeMessageFixture,
  makeAttachmentFixture,
} from "@/test/fixtures";

// Mock react-markdown to render children as HTML so we can test markdown output
vi.mock("react-markdown", () => ({
  default: ({ children }: { children: string }) => {
    // Simple mock that renders markdown-like content as HTML elements
    // Convert # headers, ```code```, and - lists to actual HTML
    let html = children || "";
    // Headers
    html = html.replace(/^# (.+)$/gm, "<h1>$1</h1>");
    html = html.replace(/^## (.+)$/gm, "<h2>$1</h2>");
    // Code blocks
    html = html.replace(/```(\w*)\n([\s\S]*?)```/g, "<pre><code>$2</code></pre>");
    // Inline code
    html = html.replace(/`([^`]+)`/g, "<code>$1</code>");
    // List items
    html = html.replace(/^- (.+)$/gm, "<li>$1</li>");
    // Wrap any <li> in <ul>
    if (html.includes("<li>")) {
      html = html.replace(/((?:<li>.*<\/li>\n?)+)/g, "<ul>$1</ul>");
    }
    return <div dangerouslySetInnerHTML={{ __html: html }} />;
  },
}));

vi.mock("remark-gfm", () => ({ default: () => {} }));
vi.mock("rehype-highlight", () => ({ default: () => {} }));

describe("MessageBubble", () => {
  const bot = makeBotFixture({
    id: "bot-1",
    name: "GPT-4o",
    avatar_color: "#6366f1",
  });
  const bots = [bot];

  // UT-COMP-03: MessageBubble renders human message (right-aligned)
  it("UT-COMP-03: renders human message right-aligned with 'You' avatar", () => {
    const message = makeMessageFixture({
      sender_type: "human",
      sender_bot_id: null,
      content: "Hello from human",
    });

    render(<MessageBubble message={message} bots={bots} />);

    // "You" avatar present
    expect(screen.getByText("You")).toBeInTheDocument();

    // Container is right-aligned (justify-end)
    const container = screen.getByText("You").closest("div.flex.gap-3");
    expect(container?.className).toContain("justify-end");

    // Primary background on the bubble
    const bubble = screen.getByText("Hello from human").closest("div.rounded-xl");
    expect(bubble?.className).toContain("bg-primary");
  });

  // UT-COMP-04: MessageBubble renders bot message (left-aligned)
  it("UT-COMP-04: renders bot message left-aligned with bot avatar and name", () => {
    const message = makeMessageFixture({
      sender_type: "bot",
      sender_bot_id: "bot-1",
      content: "Hello from bot",
    });

    render(<MessageBubble message={message} bots={bots} />);

    // Bot avatar shows first letter
    expect(screen.getByText("G")).toBeInTheDocument();

    // Bot name displayed
    expect(screen.getByText("GPT-4o")).toBeInTheDocument();

    // Container is left-aligned (justify-start)
    const container = screen.getByText("G").closest("div.flex.gap-3");
    expect(container?.className).toContain("justify-start");

    // Muted background on the bubble
    const bubble = screen.getByText("Hello from bot").closest("div.rounded-xl");
    expect(bubble?.className).toContain("bg-muted");

    // Bot avatar has correct background color
    const avatar = screen.getByText("G");
    expect(avatar).toHaveStyle({ backgroundColor: "#6366f1" });

    // "You" avatar should NOT be present
    expect(screen.queryByText("You")).not.toBeInTheDocument();
  });

  // UT-COMP-05: MessageBubble renders markdown content
  it("UT-COMP-05: renders markdown content (headers, code, lists)", () => {
    const markdownContent = `# Hello World

\`\`\`js
console.log("test");
\`\`\`

- Item one
- Item two`;

    const message = makeMessageFixture({
      sender_type: "human",
      content: markdownContent,
    });

    const { container } = render(
      <MessageBubble message={message} bots={bots} />,
    );

    // Check for rendered HTML elements
    expect(container.querySelector("h1")).toBeInTheDocument();
    expect(container.querySelector("h1")?.textContent).toBe("Hello World");
    expect(container.querySelector("code")).toBeInTheDocument();
    expect(container.querySelectorAll("li")).toHaveLength(2);
  });

  // UT-COMP-06: MessageBubble shows attachment indicators
  it("UT-COMP-06: shows attachment indicators with filenames", () => {
    const message = makeMessageFixture({
      sender_type: "human",
      content: "See attached",
      attachments: [
        makeAttachmentFixture({
          id: "att-1",
          file_name: "report.pdf",
          file_type: "file",
        }),
        makeAttachmentFixture({
          id: "att-2",
          file_name: "screenshot.png",
          file_type: "image",
          mime_type: "image/png",
        }),
      ],
    });

    render(<MessageBubble message={message} bots={bots} />);

    expect(screen.getByText("report.pdf")).toBeInTheDocument();
    expect(screen.getByText("screenshot.png")).toBeInTheDocument();
  });

  it("does not show attachment section when there are no attachments", () => {
    const message = makeMessageFixture({
      sender_type: "human",
      content: "No files",
      attachments: [],
    });

    const { container } = render(
      <MessageBubble message={message} bots={bots} />,
    );

    // No Paperclip icons (attachment indicators) should be rendered
    expect(container.querySelectorAll("[class*='Paperclip']")).toHaveLength(0);
  });
});
