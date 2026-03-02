import { useRef, useState, useCallback, useEffect } from "react";
import { Paperclip, Send, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import type { Bot } from "@/lib/tauri";

interface FileWithPreview {
  file: File;
  id: string;
}

interface MessageInputProps {
  onSend: (content: string, files: File[]) => void;
  disabled: boolean;
  bots: Bot[];
}

export function MessageInput({ onSend, disabled, bots }: MessageInputProps) {
  const [text, setText] = useState("");
  const [files, setFiles] = useState<FileWithPreview[]>([]);
  const [mentionOpen, setMentionOpen] = useState(false);
  const [mentionFilter, setMentionFilter] = useState("");
  const [mentionIndex, setMentionIndex] = useState(0);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const filteredBots = bots.filter((b) =>
    b.name.toLowerCase().includes(mentionFilter.toLowerCase()),
  );

  // Reset mention index when filter changes
  useEffect(() => {
    setMentionIndex(0);
  }, [mentionFilter]);

  const handleTextChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const value = e.target.value;
    setText(value);

    // Detect @ mention
    const cursorPos = e.target.selectionStart;
    const textBeforeCursor = value.slice(0, cursorPos);
    const atMatch = textBeforeCursor.match(/@(\w*)$/);

    if (atMatch) {
      setMentionOpen(true);
      setMentionFilter(atMatch[1]);
    } else {
      setMentionOpen(false);
      setMentionFilter("");
    }
  };

  const insertMention = useCallback(
    (bot: Bot) => {
      const textarea = textareaRef.current;
      if (!textarea) return;

      const cursorPos = textarea.selectionStart;
      const textBeforeCursor = text.slice(0, cursorPos);
      const textAfterCursor = text.slice(cursorPos);
      const atMatch = textBeforeCursor.match(/@(\w*)$/);

      if (atMatch) {
        const beforeAt = textBeforeCursor.slice(
          0,
          textBeforeCursor.length - atMatch[0].length,
        );
        const newText = `${beforeAt}@${bot.name} ${textAfterCursor}`;
        setText(newText);
      }

      setMentionOpen(false);
      setMentionFilter("");
      textarea.focus();
    },
    [text],
  );

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (mentionOpen && filteredBots.length > 0) {
      if (e.key === "ArrowDown") {
        e.preventDefault();
        setMentionIndex((prev) =>
          prev < filteredBots.length - 1 ? prev + 1 : 0,
        );
        return;
      }
      if (e.key === "ArrowUp") {
        e.preventDefault();
        setMentionIndex((prev) =>
          prev > 0 ? prev - 1 : filteredBots.length - 1,
        );
        return;
      }
      if (e.key === "Enter" || e.key === "Tab") {
        e.preventDefault();
        insertMention(filteredBots[mentionIndex]);
        return;
      }
      if (e.key === "Escape") {
        e.preventDefault();
        setMentionOpen(false);
        return;
      }
    }

    // Send on Enter (without Shift)
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleSend = () => {
    const trimmed = text.trim();
    if (!trimmed || disabled) return;
    onSend(
      trimmed,
      files.map((f) => f.file),
    );
    setText("");
    setFiles([]);
    setMentionOpen(false);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files;
    if (!selected) return;
    const newFiles: FileWithPreview[] = Array.from(selected).map((file) => ({
      file,
      id: crypto.randomUUID(),
    }));
    setFiles((prev) => [...prev, ...newFiles]);
    // Reset the input so the same file can be re-selected
    e.target.value = "";
  };

  const removeFile = (id: string) => {
    setFiles((prev) => prev.filter((f) => f.id !== id));
  };

  return (
    <div className="border-t bg-background p-3">
      {/* File previews */}
      {files.length > 0 && (
        <div className="mb-2 flex flex-wrap gap-1.5">
          {files.map((f) => (
            <span
              key={f.id}
              className="inline-flex items-center gap-1 rounded-md border bg-muted px-2 py-1 text-xs"
            >
              <Paperclip className="h-3 w-3" />
              <span className="max-w-[120px] truncate">{f.file.name}</span>
              <button
                type="button"
                onClick={() => removeFile(f.id)}
                className="ml-0.5 rounded hover:bg-accent"
              >
                <X className="h-3 w-3" />
              </button>
            </span>
          ))}
        </div>
      )}

      {/* Mention dropdown */}
      {mentionOpen && filteredBots.length > 0 && (
        <div className="mb-2 rounded-md border bg-popover p-1 shadow-md">
          {filteredBots.map((bot, i) => (
            <button
              key={bot.id}
              className={`flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm text-left transition-colors ${
                i === mentionIndex ? "bg-accent" : "hover:bg-accent/50"
              }`}
              onMouseDown={(e) => {
                e.preventDefault();
                insertMention(bot);
              }}
            >
              <div
                className="flex h-5 w-5 shrink-0 items-center justify-center rounded-full text-[10px] font-bold text-white"
                style={{ backgroundColor: bot.avatar_color }}
              >
                {bot.name.charAt(0).toUpperCase()}
              </div>
              {bot.name}
            </button>
          ))}
        </div>
      )}

      {/* Input row */}
      <div className="flex items-end gap-2">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="shrink-0"
          disabled={disabled}
          onClick={() => fileInputRef.current?.click()}
        >
          <Paperclip className="h-4 w-4" />
        </Button>
        <input
          ref={fileInputRef}
          type="file"
          multiple
          className="hidden"
          onChange={handleFileChange}
        />

        <textarea
          ref={textareaRef}
          value={text}
          onChange={handleTextChange}
          onKeyDown={handleKeyDown}
          placeholder="Type a message... (@ to mention a bot)"
          disabled={disabled}
          rows={1}
          className="flex-1 resize-none rounded-md border bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
          style={{
            minHeight: "40px",
            maxHeight: "120px",
          }}
          onInput={(e) => {
            const target = e.target as HTMLTextAreaElement;
            target.style.height = "auto";
            target.style.height = `${Math.min(target.scrollHeight, 120)}px`;
          }}
        />

        <Button
          type="button"
          size="icon"
          className="shrink-0"
          disabled={disabled || !text.trim()}
          onClick={handleSend}
        >
          <Send className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}
