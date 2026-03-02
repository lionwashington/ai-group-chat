import { useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useAppStore } from "@/stores/appStore";

interface CreateTopicDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSubmit: (title: string, botIds: string[]) => void;
}

export function CreateTopicDialog({
  open,
  onOpenChange,
  onSubmit,
}: CreateTopicDialogProps) {
  const bots = useAppStore((s) => s.bots);
  const [title, setTitle] = useState("");
  const [selectedBotIds, setSelectedBotIds] = useState<Set<string>>(new Set());

  const toggleBot = (botId: string) => {
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

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim() || selectedBotIds.size === 0) return;
    onSubmit(title.trim(), Array.from(selectedBotIds));
    setTitle("");
    setSelectedBotIds(new Set());
  };

  const handleOpenChange = (value: boolean) => {
    if (!value) {
      setTitle("");
      setSelectedBotIds(new Set());
    }
    onOpenChange(value);
  };

  const isValid = title.trim().length > 0 && selectedBotIds.size > 0;

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>New Topic</DialogTitle>
          <DialogDescription>
            Create a new conversation topic and select the bots to include.
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Title */}
          <div>
            <label htmlFor="topic-title" className="text-sm font-medium">
              Title
            </label>
            <Input
              id="topic-title"
              placeholder="What do you want to discuss?"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="mt-1"
              autoFocus
            />
          </div>

          {/* Bot selection */}
          <div>
            <label className="text-sm font-medium">
              Select Bots ({selectedBotIds.size} selected)
            </label>
            {bots.length === 0 ? (
              <p className="mt-2 text-sm text-muted-foreground">
                No bots available. Add a bot first.
              </p>
            ) : (
              <ScrollArea className="mt-1.5 max-h-[200px]">
                <div className="space-y-1 pr-3">
                  {bots.map((bot) => (
                    <label
                      key={bot.id}
                      className="flex cursor-pointer items-center gap-3 rounded-md border p-2.5 transition-colors hover:bg-accent"
                    >
                      <input
                        type="checkbox"
                        checked={selectedBotIds.has(bot.id)}
                        onChange={() => toggleBot(bot.id)}
                        className="h-4 w-4 rounded border-input"
                      />
                      <div
                        className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full text-xs font-bold text-white"
                        style={{ backgroundColor: bot.avatar_color }}
                      >
                        {bot.name.charAt(0).toUpperCase()}
                      </div>
                      <div className="flex-1 min-w-0">
                        <span className="text-sm font-medium">{bot.name}</span>
                        <p className="text-xs text-muted-foreground truncate">
                          {bot.model}
                        </p>
                      </div>
                    </label>
                  ))}
                </div>
              </ScrollArea>
            )}
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => handleOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!isValid}>
              Create Topic
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
