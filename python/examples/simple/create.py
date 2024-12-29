from fricon import Trace, Workspace

ws = Workspace.connect("path/to/workspace")
manager = ws.dataset_manager
with manager.create("example_dataset") as writer:
    writer.write(
        i=1, a="Alice", b=[1, 2], c=["A", "B"], d=Trace([1, 2, 3], x0=0.1, dx=1.1)
    )
dataset = writer.to_dataset()
print(f"Id of the dataset: {dataset.id}")
print(f"Unique id of the dataset: {dataset.uid}")
