import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { useAppStore } from "./stores/appStore";
import { listBots, listTopics, type StreamEvent } from "./lib/tauri";
import { Sidebar } from "./components/sidebar/Sidebar";
import { BotManager } from "./components/bot/BotManager";
import { CreateTopicDialog } from "./components/topic/CreateTopicDialog";
import { ChatView } from "./components/chat/ChatView";
import { createTopic } from "./lib/tauri";
import { MessageSquare } from "lucide-react";

function App() {
  const setBots = useAppStore((s) => s.setBots);
  const setTopics = useAppStore((s) => s.setTopics);
  const activeTopicId = useAppStore((s) => s.activeTopicId);
  const setActiveTopicId = useAppStore((s) => s.setActiveTopicId);
  const handleStreamEvent = useAppStore((s) => s.handleStreamEvent);

  const [botManagerOpen, setBotManagerOpen] = useState(false);
  const [createTopicOpen, setCreateTopicOpen] = useState(false);

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

  // Handle bot changes (reload bots)
  const handleBotsChanged = () => {
    loadBots();
  };

  return (
    <div className="flex h-full">
      {/* Sidebar */}
      <Sidebar
        onNewTopic={() => setCreateTopicOpen(true)}
        onManageBots={() => setBotManagerOpen(true)}
      />

      {/* Main content */}
      <div className="flex-1">
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
