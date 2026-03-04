import { useEffect, useRef, useCallback, useMemo } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Download } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/stores/appStore";
import {
  listMessages,
  sendHumanMessage,
  saveAttachment,
  chatWithBots,
  getTopic,
  exportTopic,
} from "@/lib/tauri";
import { MessageBubble } from "./MessageBubble";
import { StreamingMessage } from "./StreamingMessage";
import { MessageInput } from "./MessageInput";

export function ChatView() {
  const activeTopicId = useAppStore((s) => s.activeTopicId);
  const activeTopic = useAppStore((s) => s.activeTopic);
  const setActiveTopic = useAppStore((s) => s.setActiveTopic);
  const messages = useAppStore((s) => s.messages);
  const setMessages = useAppStore((s) => s.setMessages);
  const streamingStates = useAppStore((s) => s.streamingStates);
  const clearStreaming = useAppStore((s) => s.clearStreaming);
  const isAnyBotStreaming = useAppStore((s) => s.isAnyBotStreaming);

  const scrollRef = useRef<HTMLDivElement>(null);
  const prevStreamingRef = useRef(false);
  // true = auto-scroll enabled; false = user is reading history
  const shouldAutoScrollRef = useRef(true);

  // Detect user-initiated scrolls via wheel/touch — these never fire from programmatic scrolls
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (e.deltaY < 0) {
        // Scrolling UP — immediately disable auto-scroll (no async delay)
        shouldAutoScrollRef.current = false;
      } else if (e.deltaY > 0) {
        // Scrolling DOWN — re-enable if user reaches the bottom
        requestAnimationFrame(() => {
          const threshold = 80;
          if (el.scrollHeight - el.scrollTop - el.clientHeight < threshold) {
            shouldAutoScrollRef.current = true;
          }
        });
      }
    };
    const onTouchMove = () => {
      // On touch, conservatively disable auto-scroll; re-enabled on next send
      shouldAutoScrollRef.current = false;
    };
    el.addEventListener("wheel", onWheel, { passive: true });
    el.addEventListener("touchmove", onTouchMove, { passive: true });
    return () => {
      el.removeEventListener("wheel", onWheel);
      el.removeEventListener("touchmove", onTouchMove);
    };
  }, []);

  // Load topic and messages when activeTopicId changes
  useEffect(() => {
    if (!activeTopicId) {
      setActiveTopic(null);
      setMessages([]);
      return;
    }

    const load = async () => {
      try {
        const [topic, msgs] = await Promise.all([
          getTopic(activeTopicId),
          listMessages(activeTopicId),
        ]);
        setActiveTopic(topic);
        setMessages(msgs);
      } catch (err) {
        console.error("Failed to load topic:", err);
      }
    };

    load();
    clearStreaming();
  }, [activeTopicId, setActiveTopic, setMessages, clearStreaming]);

  // Auto-scroll to bottom only when user hasn't scrolled up
  useEffect(() => {
    const el = scrollRef.current;
    if (el && shouldAutoScrollRef.current) {
      el.scrollTop = el.scrollHeight;
    }
  }, [messages, streamingStates]);

  // When all bots finish streaming, reload messages from DB
  useEffect(() => {
    const currentlyStreaming = isAnyBotStreaming();
    const wasStreaming = prevStreamingRef.current;
    prevStreamingRef.current = currentlyStreaming;

    // Detect transition from streaming -> not streaming
    if (wasStreaming && !currentlyStreaming && activeTopicId) {
      const reload = async () => {
        try {
          const msgs = await listMessages(activeTopicId);
          setMessages(msgs);
          clearStreaming();
        } catch (err) {
          console.error("Failed to reload messages:", err);
        }
      };
      // Small delay to ensure DB writes are complete
      const timer = setTimeout(reload, 300);
      return () => clearTimeout(timer);
    }
  }, [streamingStates, activeTopicId, setMessages, clearStreaming, isAnyBotStreaming]);

  const handleSend = useCallback(
    async (content: string, files: File[]) => {
      if (!activeTopicId) return;

      try {
        // 1. Save human message
        const humanMsg = await sendHumanMessage({
          topic_id: activeTopicId,
          content,
        });

        // 2. Save attachments
        for (const file of files) {
          const arrayBuffer = await file.arrayBuffer();
          const fileData = Array.from(new Uint8Array(arrayBuffer));
          await saveAttachment(
            humanMsg.id,
            file.name,
            fileData,
            file.type || "application/octet-stream",
          );
        }

        // 3. Reload messages to show the new message
        const msgs = await listMessages(activeTopicId);
        setMessages(msgs);

        // 4. Re-enable auto-scroll — user wants to see bot responses
        shouldAutoScrollRef.current = true;

        // 5. Clear previous streaming and call chatWithBots
        clearStreaming();

        // Extract @mentioned bot names — if any, only those bots respond
        const mentionedBotIds = activeTopic?.bots
          .filter((bot) => content.includes(`@${bot.name}`))
          .map((bot) => bot.id);

        await chatWithBots({
          topic_id: activeTopicId,
          bot_ids: mentionedBotIds && mentionedBotIds.length > 0
            ? mentionedBotIds
            : undefined,
        });
      } catch (err) {
        console.error("Failed to send message:", err);
      }
    },
    [activeTopicId, activeTopic, setMessages, clearStreaming],
  );

  const handleExport = useCallback(async () => {
    if (!activeTopicId || !activeTopic) return;
    try {
      const filePath = await save({
        defaultPath: `${activeTopic.title.replace(/[^a-zA-Z0-9]/g, "_")}.aigc.json`,
        filters: [{ name: "AI Group Chat Export", extensions: ["aigc.json"] }],
      });
      if (filePath) {
        await exportTopic(activeTopicId, filePath);
      }
    } catch (err) {
      console.error("Failed to export topic:", err);
    }
  }, [activeTopicId, activeTopic]);

  // Memoize bots array so MessageBubble/StreamingMessage memo works
  const topicBots = useMemo(
    () => activeTopic?.bots ?? [],
    [activeTopic?.bots],
  );

  if (!activeTopic) {
    return null;
  }

  const streamingArray = Object.values(streamingStates).filter(
    (s) => s.content || !s.done || s.error,
  );

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="flex items-center gap-3 border-b px-4 py-3">
        <h2 className="text-lg font-semibold">{activeTopic.title}</h2>
        <div className="flex flex-wrap gap-1">
          {activeTopic.bots.map((bot) => (
            <Badge
              key={bot.id}
              variant="secondary"
              className="gap-1"
            >
              <span
                className="inline-block h-2 w-2 rounded-full"
                style={{ backgroundColor: bot.avatar_color }}
              />
              {bot.name}
            </Badge>
          ))}
        </div>
        <div className="ml-auto">
          <Button variant="ghost" size="icon" onClick={handleExport} title="Export topic">
            <Download className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Messages */}
      <div className="min-h-0 flex-1 overflow-y-auto [overflow-anchor:none]" ref={scrollRef}>
        <div className="space-y-4 py-4">
          {messages.length === 0 && streamingArray.length === 0 ? (
            <p className="px-4 py-12 text-center text-sm text-muted-foreground">
              No messages yet. Start the conversation!
            </p>
          ) : (
            <>
              {messages.map((msg) => (
                <MessageBubble
                  key={msg.id}
                  message={msg}
                  bots={topicBots}
                />
              ))}

              {streamingArray.length > 0 && (
                <>
                  <Separator className="mx-4" />
                  {streamingArray.map((state) => (
                    <StreamingMessage
                      key={state.botId}
                      state={state}
                      bots={topicBots}
                    />
                  ))}
                </>
              )}
            </>
          )}
        </div>
      </div>

      {/* Input */}
      <MessageInput
        onSend={handleSend}
        disabled={isAnyBotStreaming()}
        bots={activeTopic.bots}
      />
    </div>
  );
}
