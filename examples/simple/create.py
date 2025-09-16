from fricon import Trace, Workspace

ws = Workspace.connect("path/to/workspace")
manager = ws.dataset_manager
with manager.create("example_dataset") as writer:
    # Only numeric scalars and traces are supported
    writer.write(
        i=1,
        value=3.14,
        trace=Trace.fixed_step(0.0, 0.5, [1.0, 2.0, 3.5]),
    )
print(writer.dataset.id)
