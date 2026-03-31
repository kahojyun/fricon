---
default: minor
---

# Micro-batch dataset writes

Dataset rows are now buffered client-side and flushed automatically every
second or when 16 rows accumulate. This reduces transport overhead for
row-oriented write patterns while keeping the API unchanged.
