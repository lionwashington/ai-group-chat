import { useState } from "react";
import { Plus } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { useAppStore } from "@/stores/appStore";
import { createBot, updateBot, deleteBot } from "@/lib/tauri";
import type { Bot } from "@/lib/tauri";
import { BotCard } from "./BotCard";
import { BotFormDialog } from "./BotFormDialog";

interface BotManagerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onBotsChanged: () => void;
}

export function BotManager({ open, onOpenChange, onBotsChanged }: BotManagerProps) {
  const bots = useAppStore((s) => s.bots);
  const addBotToStore = useAppStore((s) => s.addBot);
  const removeBotFromStore = useAppStore((s) => s.removeBot);
  const updateBotInStore = useAppStore((s) => s.updateBotInStore);

  const [formOpen, setFormOpen] = useState(false);
  const [editingBot, setEditingBot] = useState<Bot | null>(null);

  const handleAddClick = () => {
    setEditingBot(null);
    setFormOpen(true);
  };

  const handleEditClick = (bot: Bot) => {
    setEditingBot(bot);
    setFormOpen(true);
  };

  const handleDeleteClick = async (bot: Bot) => {
    try {
      await deleteBot(bot.id);
      removeBotFromStore(bot.id);
      onBotsChanged();
    } catch (err) {
      console.error("Failed to delete bot:", err);
    }
  };

  const handleFormSubmit = async (data: {
    name: string;
    base_url: string;
    model: string;
    avatar_color: string;
    api_key: string;
    system_prompt: string;
    supports_vision: boolean;
  }) => {
    try {
      if (editingBot) {
        const updated = await updateBot(editingBot.id, {
          name: data.name,
          base_url: data.base_url,
          model: data.model,
          avatar_color: data.avatar_color,
          api_key: data.api_key,
          system_prompt: data.system_prompt,
          supports_vision: data.supports_vision,
        });
        updateBotInStore(updated);
      } else {
        const bot = await createBot({
          name: data.name,
          base_url: data.base_url,
          model: data.model,
          avatar_color: data.avatar_color,
          api_key: data.api_key,
          system_prompt: data.system_prompt,
          supports_vision: data.supports_vision,
        });
        addBotToStore(bot);
      }
      setFormOpen(false);
      onBotsChanged();
    } catch (err) {
      console.error("Failed to save bot:", err);
    }
  };

  return (
    <>
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-lg">
          <DialogHeader>
            <DialogTitle>Manage Bots</DialogTitle>
            <DialogDescription>
              Add, edit, or remove AI bots that can participate in your group
              chats.
            </DialogDescription>
          </DialogHeader>

          <ScrollArea className="max-h-[400px]">
            <div className="space-y-2 pr-3">
              {bots.length === 0 ? (
                <p className="py-8 text-center text-sm text-muted-foreground">
                  No bots configured yet. Add one to get started.
                </p>
              ) : (
                bots.map((bot) => (
                  <BotCard
                    key={bot.id}
                    bot={bot}
                    onEdit={handleEditClick}
                    onDelete={handleDeleteClick}
                  />
                ))
              )}
            </div>
          </ScrollArea>

          <Button onClick={handleAddClick} className="w-full gap-2">
            <Plus className="h-4 w-4" />
            Add Bot
          </Button>
        </DialogContent>
      </Dialog>

      <BotFormDialog
        open={formOpen}
        onOpenChange={setFormOpen}
        editBot={editingBot}
        onSubmit={handleFormSubmit}
      />
    </>
  );
}
