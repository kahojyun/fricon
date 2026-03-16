import { useState } from "react";
import { Tag } from "lucide-react";
import { Input } from "@/shared/ui/input";
import {
  ContextMenuItem,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
} from "@/shared/ui/context-menu";
import type { DatasetTagMenuTarget } from "../model/datasetTableTagMenuLogic";

interface DatasetNewTagFormProps {
  onSubmitTag: (tag: string) => void;
}

interface DatasetRowTagMenusProps {
  allTags: string[];
  isUpdatingTags: boolean;
  target: DatasetTagMenuTarget;
  onAddTag: (tag: string) => void;
  onRemoveTag: (tag: string) => void;
}

export function DatasetNewTagForm({ onSubmitTag }: DatasetNewTagFormProps) {
  const [newTagInput, setNewTagInput] = useState("");

  const handleSubmit = () => {
    const tag = newTagInput.trim();
    if (!tag) return;
    onSubmitTag(tag);
    setNewTagInput("");
  };

  return (
    <div
      className="px-2 pb-1"
      onPointerDown={(event) => {
        event.stopPropagation();
      }}
    >
      <form
        onSubmit={(event) => {
          event.preventDefault();
          handleSubmit();
        }}
        className="flex gap-1"
      >
        <Input
          placeholder="New tag..."
          value={newTagInput}
          onChange={(event) => setNewTagInput(event.target.value)}
          onClick={(event) => {
            event.stopPropagation();
          }}
          onKeyDown={(event) => {
            event.stopPropagation();
          }}
          className="h-7 text-xs"
        />
      </form>
    </div>
  );
}

export function DatasetRowTagMenus({
  allTags,
  isUpdatingTags,
  target,
  onAddTag,
  onRemoveTag,
}: DatasetRowTagMenusProps) {
  return (
    <>
      <ContextMenuSub>
        <ContextMenuSubTrigger>
          <Tag data-icon="inline-start" className="size-3.5" />
          Add Tags{target.targetLabel}
        </ContextMenuSubTrigger>
        <ContextMenuSubContent className="w-56">
          <DatasetNewTagForm onSubmitTag={onAddTag} />
          {allTags.length > 0 && (
            <div className="flex max-h-40 flex-col overflow-y-auto">
              {allTags.map((tag) => (
                <ContextMenuItem
                  key={tag}
                  disabled={isUpdatingTags}
                  onClick={() => {
                    onAddTag(tag);
                  }}
                >
                  {tag}
                </ContextMenuItem>
              ))}
            </div>
          )}
        </ContextMenuSubContent>
      </ContextMenuSub>

      {target.removableTags.length > 0 && (
        <ContextMenuSub>
          <ContextMenuSubTrigger>
            <Tag data-icon="inline-start" className="size-3.5" />
            Remove Tags{target.targetLabel}
          </ContextMenuSubTrigger>
          <ContextMenuSubContent className="w-56">
            <div className="flex max-h-40 flex-col overflow-y-auto">
              {target.removableTags.map((tag) => (
                <ContextMenuItem
                  key={tag}
                  disabled={isUpdatingTags}
                  onClick={() => {
                    onRemoveTag(tag);
                  }}
                >
                  {tag}
                </ContextMenuItem>
              ))}
            </div>
          </ContextMenuSubContent>
        </ContextMenuSub>
      )}
    </>
  );
}
