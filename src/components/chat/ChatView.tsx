import { useEffect, useRef, useCallback, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { Download, Pencil, Check } from "lucide-react";
import { save } from "@tauri-apps/plugin-dialog";
import { useAppStore } from "@/stores/appStore";
import {
  listMessages,
  sendHumanMessage,
  saveAttachment,
  chatWithBots,
  getTopic,
  exportTopic,
  updateTopicBots,
  type Bot,
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
  const allBots = useAppStore((s) => s.bots);

  const scrollRef = useRef<HTMLDivElement>(null);
  const prevStreamingRef = useRef(false);
  // Set to true after user sends a message to force scroll to bottom
  const forceScrollRef = useRef(true);

  const botsPopoverRequested = useAppStore((s) => s.botsPopoverRequested);
  const setBotsPopoverRequested = useAppStore((s) => s.setBotsPopoverRequested);

  const [botsPopoverOpen, setBotsPopoverOpen] = useState(false);
  const [selectedBotIds, setSelectedBotIds] = useState<Set<string>>(new Set());

  // Sync selectedBotIds when popover opens
  useEffect(() => {
    if (botsPopoverOpen && activeTopic) {
      setSelectedBotIds(new Set(activeTopic.bots.map((b) => b.id)));
    }
  }, [botsPopoverOpen, activeTopic]);

  // Open bots popover when requested from sidebar context menu
  useEffect(() => {
    if (botsPopoverRequested && activeTopic) {
      setBotsPopoverOpen(true);
      setBotsPopoverRequested(false);
    }
  }, [botsPopoverRequested, activeTopic, setBotsPopoverRequested]);

  const handleToggleBot = (botId: string) => {
    setSelectedBotIds((prev) => {
      const next = new Set(prev);
      if (next.has(botId)) {
        next.delete(botId);
      } else {
        next.add(botId);
      }
      return next;
    });
  };

  const handleSaveBots = async () => {
    if (!activeTopicId) return;
    try {
      const updated = await updateTopicBots(activeTopicId, Array.from(selectedBotIds));
      setActiveTopic(updated);
      setBotsPopoverOpen(false);
    } catch (err) {
      console.error("Failed to update topic bots:", err);
    }
  };

  // No scroll event listener needed. Auto-scroll is determined by checking
  // the current scroll position directly before each scroll decision.

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

  // Auto-scroll: scroll to bottom only if already near the bottom (or forced after send).
  // No scroll event listeners needed — just check position directly each time.
  // If user scrolled up, distFromBottom is large → no scroll.
  // If user is near bottom, distFromBottom is small → scroll to keep up.
  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    if (forceScrollRef.current) {
      el.scrollTop = el.scrollHeight;
      forceScrollRef.current = false;
      return;
    }
    const distFromBottom = el.scrollHeight - el.scrollTop - el.clientHeight;
    if (distFromBottom < 80) {
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

        // 4. Force scroll to bottom — user wants to see bot responses
        forceScrollRef.current = true;

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
        <div className="flex flex-wrap items-center gap-1">
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
        <div className="ml-auto flex items-center gap-1">
          <Popover open={botsPopoverOpen} onOpenChange={setBotsPopoverOpen}>
            <PopoverTrigger asChild>
              <Button variant="ghost" size="icon" title="Edit bots">
                <Pencil className="h-4 w-4" />
              </Button>
            </PopoverTrigger>
            <PopoverContent className="w-64 p-0" align="end">
              <div className="border-b px-3 py-2">
                <p className="text-sm font-medium">Manage Bots</p>
              </div>
              <div className="max-h-60 overflow-y-auto p-1">
                {allBots.map((bot: Bot) => {
                  const selected = selectedBotIds.has(bot.id);
                  return (
                    <button
                      key={bot.id}
                      onClick={() => handleToggleBot(bot.id)}
                      className="flex w-full items-center gap-2 rounded-sm px-2 py-1.5 text-sm hover:bg-accent"
                    >
                      <span
                        className="inline-block h-3 w-3 shrink-0 rounded-full"
                        style={{ backgroundColor: bot.avatar_color }}
                      />
                      <span className="flex-1 truncate text-left">{bot.name}</span>
                      {selected && <Check className="h-4 w-4 shrink-0 text-primary" />}
                    </button>
                  );
                })}
                {allBots.length === 0 && (
                  <p className="px-2 py-3 text-center text-xs text-muted-foreground">
                    No bots available
                  </p>
                )}
              </div>
              <div className="border-t p-2">
                <Button size="sm" className="w-full" onClick={handleSaveBots}>
                  Save
                </Button>
              </div>
            </PopoverContent>
          </Popover>
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
