---
default: minor
---

# Add explicit logical scan indices

Datasets can now store explicit per-row logical scan indices through
`DatasetWriter.write_dict(..., logical_indices=...)`, preserving scan semantics
for shuffled, sparse, resumed, and duplicate writes without adding index columns
to the visible payload.
