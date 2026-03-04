import { memo } from "react";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import type { StreamingState } from "@/stores/appStore";
import type { Bot } from "@/lib/tauri";

interface StreamingMessageProps {
  state: StreamingState;
  bots: Bot[];
}

export const StreamingMessage = memo(function StreamingMessage({ state, bots }: StreamingMessageProps) {
  const bot = bots.find((b) => b.id === state.botId);

  return (
    <div className="flex gap-3 px-4 justify-start">
      {/* Bot avatar */}
      <div
        className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full text-xs font-bold text-white mt-1"
        style={{
          backgroundColor: bot?.avatar_color ?? "#6b7280",
        }}
      >
        {bot ? bot.name.charAt(0).toUpperCase() : "?"}
      </div>

      <div className="max-w-[75%] rounded-xl bg-muted px-4 py-2.5">
        {/* Bot name */}
        <p
          className="mb-1 text-xs font-semibold"
          style={{ color: bot?.avatar_color ?? "#6b7280" }}
        >
          {state.botName}
        </p>

        {/* Error state */}
        {state.error ? (
          <p className="text-sm text-destructive">{state.error}</p>
        ) : (
          <>
            {/* Content */}
            {state.content ? (
              <div className="prose prose-sm max-w-none break-words">
                <Markdown
                  remarkPlugins={[remarkGfm]}
                  rehypePlugins={[rehypeHighlight]}
                >
                  {state.content}
                </Markdown>
              </div>
            ) : !state.done ? (
              <span className="flex gap-1 py-1">
                <span className="h-2 w-2 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:0ms]" />
                <span className="h-2 w-2 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:150ms]" />
                <span className="h-2 w-2 rounded-full bg-muted-foreground/60 animate-bounce [animation-delay:300ms]" />
              </span>
            ) : null}

            {/* Retry status */}
            {state.status === "retrying" && state.retryInfo && (
              <span className="block text-xs text-amber-500 mt-1">{state.retryInfo}</span>
            )}

            {/* Pulsing cursor when streaming */}
            {!state.done && state.content && (
              <span className="inline-block ml-0.5 w-2 h-4 bg-foreground/60 animate-pulse rounded-sm align-text-bottom" />
            )}
          </>
        )}
      </div>
    </div>
  );
});
