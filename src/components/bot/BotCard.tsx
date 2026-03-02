import { Eye, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import type { Bot } from "@/lib/tauri";

interface BotCardProps {
  bot: Bot;
  onEdit: (bot: Bot) => void;
  onDelete: (bot: Bot) => void;
}

export function BotCard({ bot, onEdit, onDelete }: BotCardProps) {
  return (
    <div className="flex items-center gap-3 rounded-lg border p-3">
      {/* Avatar */}
      <div
        className="flex h-10 w-10 shrink-0 items-center justify-center rounded-full text-sm font-bold text-white"
        style={{ backgroundColor: bot.avatar_color }}
      >
        {bot.name.charAt(0).toUpperCase()}
      </div>

      {/* Info */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2">
          <span className="font-medium truncate">{bot.name}</span>
          {bot.supports_vision && (
            <Badge variant="secondary" className="gap-1 text-xs">
              <Eye className="h-3 w-3" />
              Vision
            </Badge>
          )}
        </div>
        <p className="text-xs text-muted-foreground truncate">{bot.model}</p>
      </div>

      {/* Actions */}
      <div className="flex shrink-0 gap-1">
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8"
          onClick={() => onEdit(bot)}
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="icon"
          className="h-8 w-8 text-destructive hover:text-destructive"
          onClick={() => onDelete(bot)}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
