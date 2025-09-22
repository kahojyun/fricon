"""
Integration tests for dataset operations.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import fricon._core


class TestDatasetOperations:
    """Integration tests for dataset operations."""

    def test_dataset_writer_context_manager(self) -> None:
        """Test dataset writer with context manager."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset using context manager
            with dm.create(
                "context_test", description="Context manager test"
            ) as writer:
                writer.write(id=1, value=42.0, name="test")
                writer.write(id=2, value=84.0, name="test2")

            # Verify dataset was created
            datasets = dm.list_all()
            assert len(datasets) == 1
            assert datasets.iloc[0]["name"] == "context_test"

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_manual_close(self) -> None:
        """Test dataset writer with manual close."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset manually
            writer = dm.create("manual_test", description="Manual close test")
            writer.write(id=1, value=42.0, name="test")
            writer.close()

            # Verify dataset was created
            datasets = dm.list_all()
            assert len(datasets) == 1
            assert datasets.iloc[0]["name"] == "manual_test"

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_with_tags(self) -> None:
        """Test dataset creation with tags."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset with tags
            writer = dm.create(
                "tagged_test",
                description="Dataset with tags",
                tags=["test", "integration", "example"],
            )
            writer.write(id=1, value=42.0, name="test")
            writer.close()

            # Verify dataset was created with tags
            datasets = dm.list_all()
            assert len(datasets) == 1
            dataset_info = datasets.iloc[0]  # pyright: ignore[reportUnknownVariableType]
            assert dataset_info["name"] == "tagged_test"
            tags = dataset_info["tags"]  # pyright: ignore[reportUnknownVariableType]
            assert "test" in tags
            assert "integration" in tags

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_schema_inference(self) -> None:
        """Test automatic schema inference."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset without specifying schema
            writer = dm.create("schema_test", description="Schema inference test")

            # Write various data types
            writer.write(
                id=1,  # integer
                value=3.14,  # float
                name="test",  # string
                active=True,  # boolean
                count=42,  # another integer
            )
            writer.close()

            # Verify dataset was created
            datasets = dm.list_all()
            assert len(datasets) == 1
            assert datasets.iloc[0]["name"] == "schema_test"

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_multiple_datasets_in_workspace(self) -> None:
        """Test creating multiple datasets in the same workspace."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create first dataset
            writer1 = dm.create("dataset1", description="First dataset")
            writer1.write(id=1, value=100.0, name="item1")
            writer1.close()

            # Create second dataset
            writer2 = dm.create("dataset2", description="Second dataset")
            writer2.write(id=2, value=200.0, name="item2")
            writer2.close()

            # Create third dataset with tags
            writer3 = dm.create(
                "dataset3", description="Third dataset", tags=["special"]
            )
            writer3.write(id=3, value=300.0, name="item3")
            writer3.close()

            # Verify all datasets were created
            datasets = dm.list_all()
            expected_dataset_count = 3
            assert len(datasets) == expected_dataset_count

            # Verify dataset names
            names = datasets["name"].tolist()  # pyright: ignore[reportUnknownVariableType]
            assert "dataset1" in names
            assert "dataset2" in names
            assert "dataset3" in names

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_metadata_operations(self) -> None:
        """Test dataset metadata operations."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset
            writer = dm.create("metadata_test", description="Original description")
            writer.write(id=1, value=42.0, name="test")
            writer.close()

            # Get dataset and verify metadata
            datasets = dm.list_all()
            dataset_id = datasets.index[0]  # pyright: ignore[reportUnknownVariableType,reportUnknownMemberType]
            dataset = dm.open(dataset_id)  # pyright: ignore[reportUnknownArgumentType]

            assert dataset.name == "metadata_test"
            assert dataset.description == "Original description"
            assert not dataset.favorite

            # Note: update_metadata, add_tags, remove_tags would be tested here
            # but they require async runtime context which needs additional setup

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running
