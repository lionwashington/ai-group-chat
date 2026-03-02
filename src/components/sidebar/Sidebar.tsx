import { MessageSquare, Plus, Settings } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { useAppStore } from "@/stores/appStore";
import { cn } from "@/lib/utils";

interface SidebarProps {
  onNewTopic: () => void;
  onManageBots: () => void;
}

export function Sidebar({ onNewTopic, onManageBots }: SidebarProps) {
  const topics = useAppStore((s) => s.topics);
  const activeTopicId = useAppStore((s) => s.activeTopicId);
  const setActiveTopicId = useAppStore((s) => s.setActiveTopicId);

  return (
    <div className="flex h-full w-64 flex-col border-r bg-muted/30">
      {/* Header */}
      <div className="flex items-center gap-2 px-4 py-4">
        <MessageSquare className="h-5 w-5 text-primary" />
        <h1 className="text-lg font-semibold">AI Group Chat</h1>
      </div>

      <Separator />

      {/* Topic List */}
      <ScrollArea className="flex-1">
        <div className="p-2">
          {topics.length === 0 ? (
            <p className="px-2 py-4 text-center text-sm text-muted-foreground">
              No topics yet. Create one to get started.
            </p>
          ) : (
            topics.map((topic) => (
              <button
                key={topic.id}
                onClick={() => setActiveTopicId(topic.id)}
                className={cn(
                  "mb-1 flex w-full flex-col items-start rounded-md px-3 py-2 text-left text-sm transition-colors hover:bg-accent",
                  activeTopicId === topic.id &&
                    "bg-accent text-accent-foreground",
                )}
              >
                <span className="font-medium truncate w-full">
                  {topic.title}
                </span>
                {topic.last_message_preview && (
                  <span className="mt-0.5 truncate w-full text-xs text-muted-foreground">
                    {topic.last_message_preview}
                  </span>
                )}
                <span className="mt-0.5 text-xs text-muted-foreground">
                  {topic.bot_count} bot{topic.bot_count !== 1 ? "s" : ""}
                </span>
              </button>
            ))
          )}
        </div>
      </ScrollArea>

      <Separator />

      {/* Bottom actions */}
      <div className="flex flex-col gap-1 p-2">
        <Button
          variant="outline"
          className="w-full justify-start gap-2"
          onClick={onNewTopic}
        >
          <Plus className="h-4 w-4" />
          New Topic
        </Button>
        <Button
          variant="ghost"
          className="w-full justify-start gap-2"
          onClick={onManageBots}
        >
          <Settings className="h-4 w-4" />
          Manage Bots
        </Button>
      </div>
    </div>
  );
}
