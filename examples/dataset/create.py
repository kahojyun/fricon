from fricon import DatasetManager, Workspace


def simple(manager: DatasetManager) -> None:
    with manager.create("example", description="demo", tags=["tagA", "tagB"]) as writer:
        for i in range(5):
            for j in range(5):
                writer.write(i=i, j=j, prod=i * j, sum=i + j)

    d = writer.dataset
    assert d.name == "example"


if __name__ == "__main__":
    ws = Workspace.connect(".dev/ws")
    simple(ws.dataset_manager)
