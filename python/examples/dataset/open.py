"""Demonstrate how to open a dataset using fricon."""

from fricon import Workspace

workspace = Workspace.open(".dev/ws")
manager = workspace.dataset_manager

df = manager.list_all()
uid = df.iloc[0].uid
dataset = manager.open(uid)
print(dataset.uid)
df = dataset.to_pandas()
print(df)
df = dataset.to_polars()
print(df)
