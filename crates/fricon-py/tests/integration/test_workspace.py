"""
Integration tests for fricon workspace functionality.
"""

from __future__ import annotations

import tempfile
from pathlib import Path

import fricon._core


class TestWorkspaceIntegration:
    """Integration tests for workspace creation and management."""

    def test_serve_workspace_creates_new_workspace(self) -> None:
        """Test that serve_workspace creates a new workspace."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"

            # Create workspace and start server
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)

            # Verify workspace was created
            assert workspace_path.exists()
            assert isinstance(workspace, fricon._core.Workspace)

            # Verify workspace is functional
            dm = workspace.dataset_manager
            assert isinstance(dm, fricon._core.DatasetManager)

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_serve_multiple_workspaces(self) -> None:
        """Test creating multiple independent workspaces."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create first workspace
            workspace1_path = Path(tmpdir) / "workspace1"
            workspace1, server_handle1 = fricon._core.serve_workspace(workspace1_path)

            # Create second workspace
            workspace2_path = Path(tmpdir) / "workspace2"
            workspace2, server_handle2 = fricon._core.serve_workspace(workspace2_path)

            # Both workspaces should exist and be independent
            assert workspace1_path.exists()
            assert workspace2_path.exists()
            assert workspace1_path != workspace2_path

            # Both should have functional dataset managers
            dm1 = workspace1.dataset_manager
            dm2 = workspace2.dataset_manager
            assert isinstance(dm1, fricon._core.DatasetManager)
            assert isinstance(dm2, fricon._core.DatasetManager)

            # Explicitly shutdown both servers
            server_handle1.shutdown()
            server_handle2.shutdown()
            assert not server_handle1.is_running
            assert not server_handle2.is_running

    def test_workspace_dataset_creation(self) -> None:
        """Test dataset creation in workspace."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset
            writer = dm.create("test_dataset", description="Test dataset")
            assert isinstance(writer, fricon._core.DatasetWriter)

            # Write data and close
            writer.write(id=1, value=42.0, name="test_item")
            writer.close()

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_workspace_dataset_listing(self) -> None:
        """Test dataset listing in workspace."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Initially no datasets
            datasets = dm.list_all()
            assert len(datasets) == 0

            # Create a dataset
            writer = dm.create("test_dataset", description="Test dataset")
            writer.write(id=1, value=42.0, name="test_item")
            writer.close()

            # Should have one dataset now
            datasets = dm.list_all()
            assert len(datasets) == 1
            assert datasets.iloc[0]["name"] == "test_dataset"

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_workspace_with_context_manager(self) -> None:
        """Test workspace operations with context manager."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Use context manager for dataset writer
            with dm.create(
                "context_test", description="Context manager test"
            ) as writer:
                writer.write(id=1, value=100.0, name="context_item")
                writer.write(id=2, value=200.0, name="context_item2")

            # Dataset should be automatically closed and completed
            datasets = dm.list_all()
            assert len(datasets) == 1
            assert datasets.iloc[0]["name"] == "context_test"

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running
