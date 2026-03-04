import { useEffect, useState, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "./stores/appStore";
import { listBots, listTopics, deleteTopic, importTopic, type StreamEvent } from "./lib/tauri";
import { Sidebar } from "./components/sidebar/Sidebar";
import { BotManager } from "./components/bot/BotManager";
import { CreateTopicDialog } from "./components/topic/CreateTopicDialog";
import { ChatView } from "./components/chat/ChatView";
import { createTopic } from "./lib/tauri";
import { open } from "@tauri-apps/plugin-dialog";
import { MessageSquare } from "lucide-react";

function App() {
  const setBots = useAppStore((s) => s.setBots);
  const setTopics = useAppStore((s) => s.setTopics);
  const activeTopicId = useAppStore((s) => s.activeTopicId);
  const setActiveTopicId = useAppStore((s) => s.setActiveTopicId);
  const handleStreamEvent = useAppStore((s) => s.handleStreamEvent);

  const [botManagerOpen, setBotManagerOpen] = useState(false);
  const [createTopicOpen, setCreateTopicOpen] = useState(false);
  const [sidebarWidth, setSidebarWidth] = useState(256);
  const isDragging = useRef(false);

  // Load bots and topics on mount
  const loadBots = useCallback(async () => {
    try {
      const bots = await listBots();
      setBots(bots);
    } catch (err) {
      console.error("Failed to load bots:", err);
    }
  }, [setBots]);

  const loadTopics = useCallback(async () => {
    try {
      const topics = await listTopics();
      setTopics(topics);
    } catch (err) {
      console.error("Failed to load topics:", err);
    }
  }, [setTopics]);

  useEffect(() => {
    loadBots();
    loadTopics();
  }, [loadBots, loadTopics]);

  // Listen for chat-stream events
  useEffect(() => {
    const unlisten = listen<StreamEvent>("chat-stream", (event) => {
      handleStreamEvent(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [handleStreamEvent]);

  // Handle topic creation
  const handleCreateTopic = async (title: string, botIds: string[]) => {
    try {
      const topic = await createTopic({ title, bot_ids: botIds });
      setCreateTopicOpen(false);
      await loadTopics();
      setActiveTopicId(topic.id);
    } catch (err) {
      console.error("Failed to create topic:", err);
    }
  };

  // Handle topic deletion
  const handleDeleteTopic = async (id: string) => {
    try {
      await deleteTopic(id);
      await loadTopics();
      if (activeTopicId === id) {
        setActiveTopicId(null);
      }
    } catch (err) {
      console.error("Failed to delete topic:", err);
    }
  };

  // Handle topic import
  const handleImportTopic = async () => {
    try {
      const filePath = await open({
        filters: [{ name: "AI Group Chat Export", extensions: ["aigc.json"] }],
        multiple: false,
      });
      if (filePath) {
        const newTopicId = await importTopic(filePath as string);
        await loadTopics();
        setActiveTopicId(newTopicId);
      }
    } catch (err) {
      console.error("Failed to import topic:", err);
    }
  };

  // Sidebar resize drag
  const handleMouseDown = useCallback(() => {
    isDragging.current = true;
    document.body.style.cursor = "col-resize";
    document.body.style.userSelect = "none";

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      const newWidth = Math.min(Math.max(e.clientX, 200), 500);
      setSidebarWidth(newWidth);
    };

    const handleMouseUp = () => {
      isDragging.current = false;
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  }, []);

  // Handle bot changes (reload bots)
  const handleBotsChanged = () => {
    loadBots();
  };

  return (
    <div className="flex h-screen">
      {/* Sidebar */}
      <div className="shrink-0 border-r" style={{ width: sidebarWidth }}>
        <Sidebar
          onNewTopic={() => setCreateTopicOpen(true)}
          onManageBots={() => setBotManagerOpen(true)}
          onDeleteTopic={handleDeleteTopic}
          onImportTopic={handleImportTopic}
        />
      </div>

      {/* Resize handle */}
      <div
        onMouseDown={handleMouseDown}
        className="w-1 shrink-0 cursor-col-resize bg-transparent transition-colors hover:bg-primary/20 active:bg-primary/40"
      />

      {/* Main content */}
      <div className="min-w-0 flex-1">
        {activeTopicId ? (
          <ChatView />
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-4 text-muted-foreground">
            <MessageSquare className="h-16 w-16 opacity-20" />
            <div className="text-center">
              <h2 className="text-xl font-semibold text-foreground">
                Select or create a topic
              </h2>
              <p className="mt-1 text-sm">
                Choose a topic from the sidebar or create a new one to start
                chatting.
              </p>
            </div>
          </div>
        )}
      </div>

      {/* Dialogs */}
      <BotManager
        open={botManagerOpen}
        onOpenChange={setBotManagerOpen}
        onBotsChanged={handleBotsChanged}
      />

      <CreateTopicDialog
        open={createTopicOpen}
        onOpenChange={setCreateTopicOpen}
        onSubmit={handleCreateTopic}
      />
    </div>
  );
}

export default App;
