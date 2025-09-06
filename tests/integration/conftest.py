"""
Integration test configuration and utilities.
"""

import pytest
import tempfile
import os
from collections.abc import Generator, Callable
from typing import Optional
import fricon._core


@pytest.fixture
def temp_workspace_dir() -> Generator[str, None, None]:
    """Provide a temporary directory for workspace testing."""
    with tempfile.TemporaryDirectory() as tmpdir:
        yield tmpdir


@pytest.fixture
def workspace_factory() -> Generator[
    Callable[[Optional[str]], fricon._core.Workspace], None, None
]:
    """Factory fixture for creating workspaces."""
    workspaces: list[tuple[str, fricon._core.Workspace]] = []

    def _create_workspace(base_dir: Optional[str] = None) -> fricon._core.Workspace:
        if base_dir is None:
            base_dir = tempfile.mkdtemp()

        workspace_path = os.path.join(base_dir, "test_workspace")
        workspace = fricon._core.serve_workspace(workspace_path)
        workspaces.append((workspace_path, workspace))
        return workspace

    yield _create_workspace

    # Cleanup: Note that servers continue running in background
    # In a real scenario, you might want to add server cleanup logic


@pytest.fixture
def sample_workspace() -> Generator[fricon._core.Workspace, None, None]:
    """Provide a pre-configured workspace with sample data."""

    with tempfile.TemporaryDirectory() as tmpdir:
        workspace_path = os.path.join(tmpdir, "sample_workspace")
        workspace = fricon._core.serve_workspace(workspace_path)
        dm = workspace.dataset_manager

        # Create sample datasets
        with dm.create(
            "users", description="User data", tags=["user", "data"]
        ) as writer:
            writer.write(id=1, name="Alice", age=25, active=True)
            writer.write(id=2, name="Bob", age=30, active=False)
            writer.write(id=3, name="Charlie", age=35, active=True)

        with dm.create(
            "products", description="Product catalog", tags=["product", "catalog"]
        ) as writer:
            writer.write(id=1, name="Laptop", price=999.99, category="Electronics")
            writer.write(id=2, name="Mouse", price=29.99, category="Electronics")
            writer.write(id=3, name="Book", price=19.99, category="Books")

        with dm.create(
            "logs", description="System logs", tags=["log", "system"]
        ) as writer:
            writer.write(
                id=1,
                level="INFO",
                message="System started",
                timestamp="2024-01-01T00:00:00",
            )
            writer.write(
                id=2,
                level="WARN",
                message="Disk space low",
                timestamp="2024-01-01T01:00:00",
            )
            writer.write(
                id=3,
                level="ERROR",
                message="Connection failed",
                timestamp="2024-01-01T02:00:00",
            )

        yield workspace


@pytest.fixture
def empty_workspace() -> Generator[fricon._core.Workspace, None, None]:
    """Provide an empty workspace for testing."""

    with tempfile.TemporaryDirectory() as tmpdir:
        workspace_path = os.path.join(tmpdir, "empty_workspace")
        workspace = fricon._core.serve_workspace(workspace_path)
        yield workspace
