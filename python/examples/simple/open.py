from fricon import Workspace

ws = Workspace.connect("path/to/workspace")
manager = ws.dataset_manager
df_index = manager.list_all()  # Returns a pandas DataFrame
id_ = df_index.loc[0, "id"]  # Records are in descending order of creation
assert isinstance(id_, int)
dataset = manager.open(id_)
print(dataset.id)
pl_dset = dataset.to_polars()
