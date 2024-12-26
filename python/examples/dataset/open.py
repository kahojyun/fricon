"""Demonstrate how to open a dataset using fricon."""

from fricon import Workspace

workspace = Workspace.open(".dev/ws")
manager = workspace.dataset_manager

df_index = manager.list_all()
uid = df_index.loc[0, "uid"]
assert isinstance(uid, str)
dataset = manager.open(uid)
print(dataset.uid)
df_dset = dataset.to_pandas()
print(df_dset)
df_dset = dataset.to_polars()
print(df_dset)
