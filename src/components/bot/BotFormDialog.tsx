import { useEffect, useState } from "react";
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
import { Textarea } from "@/components/ui/textarea";
import type { Bot } from "@/lib/tauri";

const AVATAR_COLORS = [
  "#6366f1", // indigo
  "#ec4899", // pink
  "#f97316", // orange
  "#10b981", // emerald
  "#3b82f6", // blue
  "#8b5cf6", // violet
  "#ef4444", // red
  "#14b8a6", // teal
];

interface BotFormDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editBot: Bot | null;
  onSubmit: (data: {
    name: string;
    base_url: string;
    model: string;
    avatar_color: string;
    api_key: string;
    system_prompt: string;
    supports_vision: boolean;
  }) => void;
}

export function BotFormDialog({
  open,
  onOpenChange,
  editBot,
  onSubmit,
}: BotFormDialogProps) {
  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [model, setModel] = useState("");
  const [systemPrompt, setSystemPrompt] = useState("");
  const [avatarColor, setAvatarColor] = useState(AVATAR_COLORS[0]);
  const [supportsVision, setSupportsVision] = useState(false);

  // Reset / populate form when dialog opens
  useEffect(() => {
    if (open) {
      if (editBot) {
        setName(editBot.name);
        setBaseUrl(editBot.base_url);
        setApiKey(editBot.api_key);
        setModel(editBot.model);
        setSystemPrompt(editBot.system_prompt);
        setAvatarColor(editBot.avatar_color);
        setSupportsVision(editBot.supports_vision);
      } else {
        setName("");
        setBaseUrl("");
        setApiKey("");
        setModel("");
        setSystemPrompt("");
        setAvatarColor(AVATAR_COLORS[0]);
        setSupportsVision(false);
      }
    }
  }, [open, editBot]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim() || !baseUrl.trim() || !model.trim()) return;
    onSubmit({
      name: name.trim(),
      base_url: baseUrl.trim(),
      model: model.trim(),
      avatar_color: avatarColor,
      api_key: apiKey,
      system_prompt: systemPrompt,
      supports_vision: supportsVision,
    });
  };

  const isValid = name.trim() && baseUrl.trim() && model.trim();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{editBot ? "Edit Bot" : "Add Bot"}</DialogTitle>
          <DialogDescription>
            {editBot
              ? "Update the bot's configuration."
              : "Configure a new AI bot to join your conversations."}
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Avatar color picker */}
          <div>
            <label className="text-sm font-medium">Avatar Color</label>
            <div className="mt-1.5 flex gap-2">
              {AVATAR_COLORS.map((color) => (
                <button
                  key={color}
                  type="button"
                  className="h-7 w-7 rounded-full border-2 transition-all"
                  style={{
                    backgroundColor: color,
                    borderColor:
                      avatarColor === color ? "var(--foreground)" : "transparent",
                    transform: avatarColor === color ? "scale(1.15)" : "scale(1)",
                  }}
                  onClick={() => setAvatarColor(color)}
                />
              ))}
            </div>
          </div>

          {/* Name */}
          <div>
            <label htmlFor="bot-name" className="text-sm font-medium">
              Name
            </label>
            <Input
              id="bot-name"
              placeholder="e.g. GPT-4o"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="mt-1"
            />
          </div>

          {/* Base URL */}
          <div>
            <label htmlFor="bot-url" className="text-sm font-medium">
              Base URL
            </label>
            <Input
              id="bot-url"
              placeholder="https://api.openai.com/v1"
              value={baseUrl}
              onChange={(e) => setBaseUrl(e.target.value)}
              className="mt-1"
            />
          </div>

          {/* API Key */}
          <div>
            <label htmlFor="bot-key" className="text-sm font-medium">
              API Key
            </label>
            <Input
              id="bot-key"
              type="password"
              placeholder="sk-..."
              value={apiKey}
              onChange={(e) => setApiKey(e.target.value)}
              className="mt-1"
            />
          </div>

          {/* Model */}
          <div>
            <label htmlFor="bot-model" className="text-sm font-medium">
              Model
            </label>
            <Input
              id="bot-model"
              placeholder="gpt-4o"
              value={model}
              onChange={(e) => setModel(e.target.value)}
              className="mt-1"
            />
          </div>

          {/* System Prompt */}
          <div>
            <label htmlFor="bot-prompt" className="text-sm font-medium">
              System Prompt
            </label>
            <Textarea
              id="bot-prompt"
              placeholder="You are a helpful assistant..."
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              className="mt-1"
              rows={3}
            />
          </div>

          {/* Supports Vision */}
          <label className="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={supportsVision}
              onChange={(e) => setSupportsVision(e.target.checked)}
              className="h-4 w-4 rounded border-input"
            />
            <span className="text-sm">Supports vision (image input)</span>
          </label>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!isValid}>
              {editBot ? "Save Changes" : "Add Bot"}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
