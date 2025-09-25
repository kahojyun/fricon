from fricon import Trace, Workspace

ws = Workspace.connect("path/to/workspace")
manager = ws.dataset_manager
with manager.create("example_dataset") as writer:
    writer.write(
        i=1,
        a=42.0,
        b=[1.0, 2.0],
        c=[1 + 2j, 3 + 4j],
        d=Trace.fixed_step(0.1, 1.1, [1, 2, 3]),
    )
print(f"Id of the dataset: {writer.dataset.id}")
