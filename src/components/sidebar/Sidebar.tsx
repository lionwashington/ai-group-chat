import { useState, useRef, useEffect } from "react";
import { MessageSquare, Plus, Settings, Trash2, Upload } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { useAppStore } from "@/stores/appStore";
import { cn } from "@/lib/utils";

interface SidebarProps {
  onNewTopic: () => void;
  onManageBots: () => void;
  onDeleteTopic: (id: string) => void;
  onImportTopic: () => void;
  onRenameTopic: (id: string, title: string) => void;
  onUpdateBots: (id: string) => void;
  onExportTopic: (id: string) => void;
}

export function Sidebar({ onNewTopic, onManageBots, onDeleteTopic, onImportTopic, onRenameTopic, onUpdateBots, onExportTopic }: SidebarProps) {
  const topics = useAppStore((s) => s.topics);
  const activeTopicId = useAppStore((s) => s.activeTopicId);
  const setActiveTopicId = useAppStore((s) => s.setActiveTopicId);

  const [deleteTarget, setDeleteTarget] = useState<{ id: string; title: string } | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState("");
  const editInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (editingId && editInputRef.current) {
      editInputRef.current.focus();
      editInputRef.current.select();
    }
  }, [editingId]);

  const handleRenameSubmit = (id: string) => {
    const trimmed = editingTitle.trim();
    if (trimmed && trimmed !== topics.find((t) => t.id === id)?.title) {
      onRenameTopic(id, trimmed);
    }
    setEditingId(null);
  };

  return (
    <div className="flex h-full flex-col bg-muted/30">
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
              <ContextMenu key={topic.id}>
                <ContextMenuTrigger asChild>
                  <div
                    className={cn(
                      "group mb-1 flex items-center rounded-md transition-colors hover:bg-accent",
                      activeTopicId === topic.id &&
                        "bg-accent text-accent-foreground",
                    )}
                  >
                    {editingId === topic.id ? (
                      <div className="flex-1 px-3 py-2">
                        <input
                          ref={editInputRef}
                          value={editingTitle}
                          onChange={(e) => setEditingTitle(e.target.value)}
                          onBlur={() => handleRenameSubmit(topic.id)}
                          onKeyDown={(e) => {
                            if (e.key === "Enter") handleRenameSubmit(topic.id);
                            if (e.key === "Escape") setEditingId(null);
                          }}
                          className="w-full rounded border bg-background px-1.5 py-0.5 text-sm font-medium outline-none focus:ring-1 focus:ring-ring"
                        />
                      </div>
                    ) : (
                      <button
                        onClick={() => setActiveTopicId(topic.id)}
                        onDoubleClick={(e) => {
                          e.preventDefault();
                          setEditingId(topic.id);
                          setEditingTitle(topic.title);
                        }}
                        className="flex flex-1 flex-col items-start px-3 py-2 text-left text-sm min-w-0"
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
                    )}
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        setDeleteTarget({ id: topic.id, title: topic.title });
                      }}
                      className="mr-1 shrink-0 rounded p-1 opacity-0 transition-opacity hover:bg-destructive/10 hover:text-destructive group-hover:opacity-100"
                      title="Delete topic"
                    >
                      <Trash2 className="h-3.5 w-3.5" />
                    </button>
                  </div>
                </ContextMenuTrigger>
                <ContextMenuContent>
                  <ContextMenuItem
                    onClick={() => {
                      setEditingId(topic.id);
                      setEditingTitle(topic.title);
                    }}
                  >
                    Rename
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => onUpdateBots(topic.id)}>
                    Update Bots
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => onExportTopic(topic.id)}>
                    Export Topic
                  </ContextMenuItem>
                  <ContextMenuItem
                    className="text-destructive focus:text-destructive"
                    onClick={() => setDeleteTarget({ id: topic.id, title: topic.title })}
                  >
                    Delete
                  </ContextMenuItem>
                </ContextMenuContent>
              </ContextMenu>
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
          onClick={onImportTopic}
        >
          <Upload className="h-4 w-4" />
          Import Topic
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

      {/* Delete confirmation dialog */}
      <AlertDialog
        open={!!deleteTarget}
        onOpenChange={(open) => { if (!open) setDeleteTarget(null); }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Topic</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete "{deleteTarget?.title}"? All messages in this topic will be permanently deleted.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              variant="destructive"
              onClick={() => {
                if (deleteTarget) {
                  onDeleteTopic(deleteTarget.id);
                  setDeleteTarget(null);
                }
              }}
            >
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
