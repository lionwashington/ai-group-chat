import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import { Paperclip } from "lucide-react";
import type { Message, Bot } from "@/lib/tauri";
import { cn } from "@/lib/utils";

interface MessageBubbleProps {
  message: Message;
  bots: Bot[];
}

export function MessageBubble({ message, bots }: MessageBubbleProps) {
  const isHuman = message.sender_type === "human";
  const senderBot = message.sender_bot_id
    ? bots.find((b) => b.id === message.sender_bot_id)
    : null;

  return (
    <div
      className={cn(
        "flex gap-3 px-4",
        isHuman ? "justify-end" : "justify-start",
      )}
    >
      {/* Bot avatar (left) */}
      {!isHuman && (
        <div
          className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-xs font-bold text-white mt-1"
          style={{
            backgroundColor: senderBot?.avatar_color ?? "#6b7280",
          }}
        >
          {senderBot
            ? senderBot.name.charAt(0).toUpperCase()
            : "?"}
        </div>
      )}

      <div
        className={cn(
          "max-w-[75%] rounded-xl px-4 py-2.5",
          isHuman
            ? "bg-primary text-primary-foreground"
            : "bg-muted",
        )}
      >
        {/* Bot name */}
        {!isHuman && senderBot && (
          <p className="mb-1 text-xs font-semibold" style={{ color: senderBot.avatar_color }}>
            {senderBot.name}
          </p>
        )}

        {/* Content */}
        <div
          className={cn(
            "prose prose-sm max-w-none break-words",
            isHuman && "prose-invert",
          )}
        >
          <Markdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeHighlight]}>
            {message.content}
          </Markdown>
        </div>

        {/* Attachments */}
        {message.attachments.length > 0 && (
          <div className="mt-2 flex flex-wrap gap-1">
            {message.attachments.map((att) => (
              <span
                key={att.id}
                className="inline-flex items-center gap-1 rounded bg-background/50 px-2 py-0.5 text-xs"
              >
                <Paperclip className="h-3 w-3" />
                {att.file_name}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Human avatar (right) */}
      {isHuman && (
        <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-primary text-xs font-bold text-primary-foreground mt-1">
          You
        </div>
      )}
    </div>
  );
}
