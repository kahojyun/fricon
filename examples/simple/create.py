from fricon import Trace, Workspace

ws = Workspace.connect("path/to/workspace")
manager = ws.dataset_manager
with manager.create("example_dataset") as writer:
    writer.write(
        i=1,
        a="Alice",
        b=[1, 2],
        c=["A", "B"],
        d=Trace.fixed_step(0.1, 1.1, [1, 2, 3]),
    )
print(f"Id of the dataset: {writer.dataset.id}")
