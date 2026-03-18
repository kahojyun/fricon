import { useRef, useState } from "react";
import { GitMerge, Pencil, Tags, Trash2, X, Check } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/shared/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/shared/ui/dialog";
import { Input } from "@/shared/ui/input";
import { Badge } from "@/shared/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/shared/ui/select";

interface ManageTagsDialogProps {
  allTags: string[];
  isUpdatingTags: boolean;
  onDeleteTag: (tag: string) => Promise<void>;
  onRenameTag: (oldName: string, newName: string) => Promise<void>;
  onMergeTag: (source: string, target: string) => Promise<void>;
}

type TagAction =
  | { type: "rename"; tag: string }
  | { type: "merge"; tag: string };

async function runTagAction({
  tag,
  setBusyTag,
  action,
  onSuccess,
  onError,
}: {
  tag: string;
  setBusyTag: (tag: string | null) => void;
  action: () => Promise<void>;
  onSuccess: () => void;
  onError: (error: unknown) => void;
}) {
  setBusyTag(tag);
  try {
    await action();
    onSuccess();
  } catch (error) {
    onError(error);
  } finally {
    setBusyTag(null);
  }
}

export function ManageTagsDialog({
  allTags,
  isUpdatingTags,
  onDeleteTag,
  onRenameTag,
  onMergeTag,
}: ManageTagsDialogProps) {
  const [open, setOpen] = useState(false);
  const [tagSearch, setTagSearch] = useState("");
  const [pendingAction, setPendingAction] = useState<Record<string, TagAction>>(
    {},
  );
  const [busyTag, setBusyTag] = useState<string | null>(null);
  const [renameValues, setRenameValues] = useState<Record<string, string>>({});
  const [mergeTargets, setMergeTargets] = useState<Record<string, string>>({});
  const renameInputRef = useRef<HTMLInputElement>(null);

  const filteredTags = tagSearch.trim()
    ? allTags.filter((t) =>
        t.toLowerCase().includes(tagSearch.trim().toLowerCase()),
      )
    : allTags;

  const clearAction = (tag: string) => {
    setPendingAction((prev) => {
      const next = { ...prev };
      delete next[tag];
      return next;
    });
  };

  const startRename = (tag: string) => {
    setRenameValues((prev) => ({ ...prev, [tag]: tag }));
    setPendingAction((prev) => ({ ...prev, [tag]: { type: "rename", tag } }));
    // Focus after state flush
    setTimeout(() => renameInputRef.current?.focus(), 0);
  };

  const startMerge = (tag: string) => {
    setMergeTargets((prev) => ({ ...prev, [tag]: "" }));
    setPendingAction((prev) => ({ ...prev, [tag]: { type: "merge", tag } }));
  };

  const handleDelete = (tag: string) =>
    runTagAction({
      tag,
      setBusyTag,
      action: () => onDeleteTag(tag),
      onSuccess: () => {
        toast.success(`Tag "${tag}" deleted.`);
      },
      onError: (error) => {
        toast.error(
          error instanceof Error
            ? error.message
            : `Failed to delete tag "${tag}".`,
        );
      },
    });

  const handleRenameConfirm = async (tag: string) => {
    const newName = renameValues[tag]?.trim();
    if (!newName || newName === tag) {
      clearAction(tag);
      return;
    }
    if (allTags.includes(newName)) {
      toast.error(`A tag named "${newName}" already exists.`);
      return;
    }
    await runTagAction({
      tag,
      setBusyTag,
      action: () => onRenameTag(tag, newName),
      onSuccess: () => {
        toast.success(`Tag renamed to "${newName}".`);
        clearAction(tag);
      },
      onError: (error) => {
        toast.error(
          error instanceof Error
            ? error.message
            : `Failed to rename tag "${tag}".`,
        );
      },
    });
  };

  const handleMergeConfirm = async (tag: string) => {
    const target = mergeTargets[tag];
    if (!target) return;
    await runTagAction({
      tag,
      setBusyTag,
      action: () => onMergeTag(tag, target),
      onSuccess: () => {
        toast.success(`Tag "${tag}" merged into "${target}".`);
        clearAction(tag);
      },
      onError: (error) => {
        toast.error(
          error instanceof Error
            ? error.message
            : `Failed to merge tag "${tag}".`,
        );
      },
    });
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger
        render={
          <Button variant="ghost" className="w-full justify-start text-xs" />
        }
      >
        <Tags data-icon="inline-start" />
        Manage Tags
      </DialogTrigger>
      <DialogContent className="max-w-sm">
        <DialogHeader>
          <DialogTitle>Manage Tags</DialogTitle>
          <DialogDescription>
            Delete, rename, or merge tags across all datasets.
          </DialogDescription>
        </DialogHeader>
        <Input
          placeholder="Search tags..."
          value={tagSearch}
          onChange={(e) => setTagSearch(e.target.value)}
          autoFocus
        />
        <div className="flex max-h-72 flex-col gap-1 overflow-y-auto">
          {filteredTags.length === 0 ? (
            <div className="py-6 text-center text-sm text-muted-foreground">
              {allTags.length === 0
                ? "No tags in workspace."
                : "No tags matched."}
            </div>
          ) : (
            filteredTags.map((tag) => {
              const action = pendingAction[tag];
              const isBusy = busyTag === tag;
              const isDisabled = isUpdatingTags || isBusy;

              if (action?.type === "rename") {
                return (
                  <div
                    key={tag}
                    className="flex items-center gap-1.5 rounded-md bg-muted/40 px-2 py-1.5"
                  >
                    <Input
                      ref={renameInputRef}
                      className="h-7 flex-1 text-sm"
                      value={renameValues[tag] ?? tag}
                      onChange={(e) =>
                        setRenameValues((prev) => ({
                          ...prev,
                          [tag]: e.target.value,
                        }))
                      }
                      onKeyDown={(e) => {
                        if (e.key === "Enter") void handleRenameConfirm(tag);
                        if (e.key === "Escape") clearAction(tag);
                      }}
                      disabled={isDisabled}
                    />
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 shrink-0 p-0 text-green-600 hover:text-green-700"
                      disabled={isDisabled}
                      onClick={() => void handleRenameConfirm(tag)}
                      aria-label="Confirm rename"
                    >
                      <Check className="size-3.5" />
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 shrink-0 p-0 text-muted-foreground"
                      disabled={isDisabled}
                      onClick={() => clearAction(tag)}
                      aria-label="Cancel rename"
                    >
                      <X className="size-3.5" />
                    </Button>
                  </div>
                );
              }

              if (action?.type === "merge") {
                const mergeOptions = allTags.filter((t) => t !== tag);
                return (
                  <div
                    key={tag}
                    className="flex items-center gap-1.5 rounded-md bg-muted/40 px-2 py-1.5"
                  >
                    <Badge
                      variant="secondary"
                      className="max-w-24 shrink-0 truncate"
                    >
                      {tag}
                    </Badge>
                    <span className="text-xs text-muted-foreground">→</span>
                    <Select
                      value={mergeTargets[tag] ?? ""}
                      onValueChange={(v) =>
                        v !== null &&
                        setMergeTargets((prev) => ({ ...prev, [tag]: v }))
                      }
                      disabled={isDisabled}
                    >
                      <SelectTrigger className="h-7 flex-1 text-xs">
                        <SelectValue placeholder="Pick target…" />
                      </SelectTrigger>
                      <SelectContent>
                        {mergeOptions.map((t) => (
                          <SelectItem key={t} value={t} className="text-xs">
                            {t}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 shrink-0 p-0 text-green-600 hover:text-green-700"
                      disabled={isDisabled || !mergeTargets[tag]}
                      onClick={() => void handleMergeConfirm(tag)}
                      aria-label="Confirm merge"
                    >
                      <Check className="size-3.5" />
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 shrink-0 p-0 text-muted-foreground"
                      disabled={isDisabled}
                      onClick={() => clearAction(tag)}
                      aria-label="Cancel merge"
                    >
                      <X className="size-3.5" />
                    </Button>
                  </div>
                );
              }

              // Default row
              return (
                <div
                  key={tag}
                  className="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 hover:bg-muted/50"
                >
                  <Badge variant="secondary" className="max-w-40 truncate">
                    {tag}
                  </Badge>
                  <div className="flex shrink-0 items-center gap-0.5">
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 p-0 text-muted-foreground hover:text-foreground"
                      disabled={isDisabled}
                      onClick={() => startRename(tag)}
                      aria-label={`Rename tag ${tag}`}
                    >
                      <Pencil className="size-3.5" />
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 p-0 text-muted-foreground hover:text-foreground"
                      disabled={isDisabled || allTags.length < 2}
                      onClick={() => startMerge(tag)}
                      aria-label={`Merge tag ${tag}`}
                    >
                      <GitMerge className="size-3.5" />
                    </Button>
                    <Button
                      type="button"
                      variant="ghost"
                      size="sm"
                      className="h-7 w-7 p-0 text-muted-foreground hover:text-destructive"
                      disabled={isDisabled}
                      onClick={() => void handleDelete(tag)}
                      aria-label={`Delete tag ${tag}`}
                    >
                      <Trash2 className="size-3.5" />
                    </Button>
                  </div>
                </div>
              );
            })
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
