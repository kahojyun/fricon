---
default: patch
---

# Targeted dataset event system for UI cache invalidation

Replace the single `Updated` backend event with fine-grained dataset lifecycle events
(Created, StatusChanged, MetadataUpdated, TagsChanged, Trashed, Restored, Deleted,
Imported, GlobalTagsChanged). The frontend now performs targeted React Query cache
invalidation per-event instead of blanket refetches, and chart data refreshes
correctly during active write sessions.
