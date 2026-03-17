import { useMemo, useState } from "react";
import { PlusCircle } from "lucide-react";
import { ManageTagsDialog } from "./ManageTagsDialog";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/shared/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/shared/ui/popover";
import { Separator } from "@/shared/ui/separator";

interface DatasetTagFilterProps {
  selectedTags: string[];
  allTags: string[];
  isUpdatingTags: boolean;
  onToggleTag: (tag: string) => void;
  onDeleteTag: (tag: string) => Promise<void>;
  onRenameTag: (oldName: string, newName: string) => Promise<void>;
  onMergeTag: (source: string, target: string) => Promise<void>;
}

export function DatasetTagFilter({
  selectedTags,
  allTags,
  isUpdatingTags,
  onToggleTag,
  onDeleteTag,
  onRenameTag,
  onMergeTag,
}: DatasetTagFilterProps) {
  const [open, setOpen] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");

  const filteredTags = useMemo(() => {
    const normalizedQuery = searchQuery.trim().toLowerCase();
    if (!normalizedQuery) {
      return allTags;
    }

    return allTags.filter((tag) => tag.toLowerCase().includes(normalizedQuery));
  }, [allTags, searchQuery]);

  const emptyLabel =
    allTags.length === 0 ? "No tags in workspace." : "No tags found.";

  return (
    <Popover
      open={open}
      onOpenChange={(nextOpen) => {
        setOpen(nextOpen);
        if (!nextOpen) {
          setSearchQuery("");
        }
      }}
    >
      <PopoverTrigger
        render={<Button variant="outline" className="border-dashed" />}
      >
        <PlusCircle data-icon="inline-start" />
        Tags
        {selectedTags.length > 0 && (
          <>
            <Separator orientation="vertical" className="mx-2 h-4" />
            <Badge variant="secondary" className="lg:hidden">
              {selectedTags.length}
            </Badge>
            <div className="hidden flex-wrap gap-1 lg:flex">
              {selectedTags.length > 2 ? (
                <Badge variant="secondary">
                  {selectedTags.length} selected
                </Badge>
              ) : (
                selectedTags.map((tag) => (
                  <Badge
                    key={tag}
                    variant="secondary"
                    className="max-w-24 truncate"
                  >
                    {tag}
                  </Badge>
                ))
              )}
            </div>
          </>
        )}
      </PopoverTrigger>
      <PopoverContent align="start" className="w-72 gap-0 p-0">
        <Command
          shouldFilter={false}
          className="rounded-none bg-transparent p-0"
        >
          <CommandInput
            placeholder="Search tags"
            value={searchQuery}
            onValueChange={setSearchQuery}
          />
          <CommandList className="max-h-56">
            <CommandEmpty>{emptyLabel}</CommandEmpty>
            <CommandGroup>
              {filteredTags.map((tag) => (
                <CommandItem
                  key={tag}
                  value={tag}
                  data-checked={selectedTags.includes(tag) ? "true" : undefined}
                  onSelect={() => onToggleTag(tag)}
                >
                  {tag}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
          {selectedTags.length > 0 && (
            <>
              <CommandSeparator />
              <div className="p-1 pt-0">
                <Button
                  variant="ghost"
                  className="w-full justify-center"
                  onClick={() => {
                    selectedTags.forEach((tag) => onToggleTag(tag));
                  }}
                >
                  Clear filters
                </Button>
              </div>
            </>
          )}
        </Command>
        <Separator />
        <ManageTagsDialog
          allTags={allTags}
          isUpdatingTags={isUpdatingTags}
          onDeleteTag={onDeleteTag}
          onRenameTag={onRenameTag}
          onMergeTag={onMergeTag}
        />
      </PopoverContent>
    </Popover>
  );
}
