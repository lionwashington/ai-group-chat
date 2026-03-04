import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sidebar } from "../Sidebar";
import { useAppStore } from "@/stores/appStore";
import { makeTopicSummaryFixture } from "@/test/fixtures";

const initialState = useAppStore.getState();

beforeEach(() => {
  useAppStore.setState(initialState, true);
});

describe("Sidebar", () => {
  const defaultProps = {
    onNewTopic: vi.fn(),
    onManageBots: vi.fn(),
    onDeleteTopic: vi.fn(),
    onImportTopic: vi.fn(),
    onRenameTopic: vi.fn(),
    onUpdateBots: vi.fn(),
    onExportTopic: vi.fn(),
  };

  // UT-COMP-01: Sidebar renders topic list
  it("UT-COMP-01: renders topic list in order", () => {
    const topics = [
      makeTopicSummaryFixture({ id: "t1", title: "First Topic", bot_count: 1 }),
      makeTopicSummaryFixture({ id: "t2", title: "Second Topic", bot_count: 3 }),
      makeTopicSummaryFixture({ id: "t3", title: "Third Topic", bot_count: 2 }),
    ];
    useAppStore.setState({ topics });

    render(<Sidebar {...defaultProps} />);

    const buttons = screen.getAllByRole("button").filter((btn) =>
      topics.some((t) => btn.textContent?.includes(t.title)),
    );
    expect(buttons).toHaveLength(3);
    expect(buttons[0]).toHaveTextContent("First Topic");
    expect(buttons[1]).toHaveTextContent("Second Topic");
    expect(buttons[2]).toHaveTextContent("Third Topic");
  });

  it("UT-COMP-01: shows 'No topics yet' when list is empty", () => {
    useAppStore.setState({ topics: [] });

    render(<Sidebar {...defaultProps} />);

    expect(screen.getByText(/No topics yet/i)).toBeInTheDocument();
  });

  // UT-COMP-02: Sidebar highlights active topic
  it("UT-COMP-02: highlights active topic with accent background", () => {
    const topics = [
      makeTopicSummaryFixture({ id: "t1", title: "Active Topic" }),
      makeTopicSummaryFixture({ id: "t2", title: "Inactive Topic" }),
    ];
    useAppStore.setState({ topics, activeTopicId: "t1" });

    render(<Sidebar {...defaultProps} />);

    // bg-accent is on the wrapper div, not the button itself
    const activeDiv = screen.getByText("Active Topic").closest("button")?.parentElement;
    const inactiveDiv = screen.getByText("Inactive Topic").closest("button")?.parentElement;

    expect(activeDiv?.className).toContain("bg-accent");
    expect(inactiveDiv?.className).not.toContain("bg-accent text-accent-foreground");
  });

  it("clicking a topic calls setActiveTopicId", async () => {
    const user = userEvent.setup();
    const topics = [
      makeTopicSummaryFixture({ id: "t1", title: "Click Me" }),
    ];
    useAppStore.setState({ topics, activeTopicId: null });

    render(<Sidebar {...defaultProps} />);

    await user.click(screen.getByText("Click Me"));
    expect(useAppStore.getState().activeTopicId).toBe("t1");
  });

  it("renders New Topic and Manage Bots buttons", async () => {
    const user = userEvent.setup();
    const onNewTopic = vi.fn();
    const onManageBots = vi.fn();

    render(<Sidebar onNewTopic={onNewTopic} onManageBots={onManageBots} onDeleteTopic={vi.fn()} onImportTopic={vi.fn()} onRenameTopic={vi.fn()} onUpdateBots={vi.fn()} onExportTopic={vi.fn()} />);

    await user.click(screen.getByText("New Topic"));
    expect(onNewTopic).toHaveBeenCalledOnce();

    await user.click(screen.getByText("Manage Bots"));
    expect(onManageBots).toHaveBeenCalledOnce();
  });

  it("displays bot count and preview for topics", () => {
    const topics = [
      makeTopicSummaryFixture({
        id: "t1",
        title: "Topic With Preview",
        bot_count: 3,
        last_message_preview: "Last message here...",
      }),
    ];
    useAppStore.setState({ topics });

    render(<Sidebar {...defaultProps} />);

    expect(screen.getByText("3 bots")).toBeInTheDocument();
    expect(screen.getByText("Last message here...")).toBeInTheDocument();
  });
});
