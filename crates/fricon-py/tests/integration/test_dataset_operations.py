"""
Integration tests for dataset operations.
"""

from __future__ import annotations

import gc
import json
import tempfile
import time
from pathlib import Path
from typing import cast

import fricon
import fricon._core
import pytest


class TestDatasetOperations:
    """Integration tests for dataset operations."""

    @staticmethod
    def _raise_runtime_error() -> None:
        message = "boom"
        raise RuntimeError(message)

    @staticmethod
    def _wait_dataset_status(
        dm: fricon._core.DatasetManager,
        name: str,
        expected_status: str,
        timeout_sec: float = 2.0,
    ) -> None:
        deadline = time.monotonic() + timeout_sec
        while time.monotonic() < deadline:
            datasets = dm.list_all()
            matched = datasets[datasets["name"] == name]
            if len(matched) == 1:
                dataset_id = matched.index[0]  # pyright: ignore[reportAny]
                dataset = dm.open(dataset_id)  # pyright: ignore[reportAny]
                if dataset.status == expected_status:
                    return
            time.sleep(0.05)
        message = f"Dataset '{name}' did not reach status '{expected_status}' in time"
        raise AssertionError(message)

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
                writer.write(id=1, value=42.0, measurement=3.14 + 2j)
                writer.write(id=2, value=84.0, measurement=1.618 - 1j)

            # Verify dataset was created
            datasets = dm.list_all()
            assert len(datasets) == 1
            assert datasets.iloc[0]["name"] == "context_test"

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_column_metadata_round_trips_to_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            with dm.create(
                "column_metadata",
                columns={
                    "voltage": fricon.Column(
                        float,
                        unit="V",
                        label="Voltage",
                        hidden_by_default=True,
                        chart_axis=True,
                    ),
                    "measurement": complex,
                },
            ) as writer:
                writer.write(measurement=1.0 + 2.0j, voltage=0.25)
                dataset = writer.finish()

            manifest_path = Path(dataset.path) / "dataset_manifest.json"
            manifest = cast("dict[str, object]", json.loads(manifest_path.read_text()))
            columns = cast("dict[str, object]", manifest["columns"])
            voltage = cast("dict[str, object]", columns["voltage"])
            assert voltage["dtype"] == {"kind": "float64"}
            assert voltage["unit"] == "V"
            assert voltage["label"] == "Voltage"
            assert voltage["hidden_by_default"] is True
            assert voltage["chart_axis"] is True
            measurement = cast("dict[str, object]", columns["measurement"])
            assert measurement["dtype"] == {"kind": "complex128"}
            assert dataset.to_arrow().column_names == ["voltage", "measurement"]

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_column_metadata_can_infer_dtype(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            with dm.create(
                "metadata_only",
                columns={"phase": fricon.Column(unit="rad")},
            ) as writer:
                writer.write(phase=1.5)
                dataset = writer.finish()

            manifest = cast(
                "dict[str, object]",
                json.loads((Path(dataset.path) / "dataset_manifest.json").read_text()),
            )
            columns = cast("dict[str, object]", manifest["columns"])
            phase = cast("dict[str, object]", columns["phase"])
            assert phase["dtype"] == {"kind": "float64"}
            assert phase["unit"] == "rad"

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_scan_metadata_round_trips_to_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            with dm.create(
                "scan_metadata",
                scan={
                    "gate": [-0.2, -0.1],
                    "bias": [0, 1],
                },
            ) as writer:
                writer.write(signal=1.0)
                dataset = writer.finish()

            manifest = cast(
                "dict[str, object]",
                json.loads((Path(dataset.path) / "dataset_manifest.json").read_text()),
            )
            scan_plan = cast("dict[str, object]", manifest["scan_plan"])
            axes = cast("list[dict[str, object]]", scan_plan["axes"])
            assert axes[0]["name"] == "gate"
            assert axes[1]["name"] == "bias"
            assert cast("dict[str, object]", manifest["realization"])[
                "index_realization"
            ] == {"kind": "implicit"}

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_scan_index_axis_round_trips_to_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            with dm.create(
                "index_scan",
                scan={"step": fricon.IndexAxis(label="Step")},
            ) as writer:
                writer.write(loss=1.0)
                dataset = writer.finish()

            manifest = cast(
                "dict[str, object]",
                json.loads((Path(dataset.path) / "dataset_manifest.json").read_text()),
            )
            scan_plan = cast("dict[str, object]", manifest["scan_plan"])
            axes = cast("list[dict[str, object]]", scan_plan["axes"])
            assert axes == [
                {
                    "name": "step",
                    "label": "Step",
                    "mode": {"kind": "implicit_index"},
                }
            ]

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_scan_validation_rejects_invalid_specs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            with pytest.raises(ValueError, match="scan must not be empty"):
                _ = dm.create("empty_scan", scan={})
            with pytest.raises(ValueError, match="static scan axis gate"):
                _ = dm.create("empty_axis", scan={"gate": []})
            with pytest.raises(ValueError, match="mixed static and unknown"):
                _ = dm.create("mixed_axis", scan={"gate": [0.0], "step": None})
            with pytest.raises(ValueError, match="reserved system prefix"):
                _ = dm.create("reserved_axis", scan={"__ds_step": [0]})

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_declared_columns_require_exact_first_row(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            writer = dm.create("missing_column", columns={"voltage": float})
            with pytest.raises(ValueError, match="Missing declared column 'voltage'"):
                writer.write(current=1.0)

            writer = dm.create("extra_column", columns={"voltage": float})
            with pytest.raises(ValueError, match="Unexpected column 'current'"):
                writer.write(voltage=1.0, current=2.0)

            writer = dm.create("wrong_dtype", columns={"measurement": complex})
            with pytest.raises(ValueError, match="declared dtype"):
                writer.write(measurement=1.0)

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_writer_context_manager_exception_aborts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            try:
                with dm.create(
                    "context_abort_dataset", description="exception flow"
                ) as writer:
                    writer.write(id=1, value=9.0, measurement=9.0 + 0.5j)
                    self._raise_runtime_error()
            except RuntimeError:
                pass

            self._wait_dataset_status(dm, "context_abort_dataset", "aborted")

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_writer_finish_returns_completed_dataset(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            writer = dm.create("finish_dataset", description="finish flow")
            writer.write(id=1, value=1.0, measurement=1.0 + 0.5j)
            dataset = writer.finish()

            assert dataset.name == "finish_dataset"
            assert dataset.status == "completed"

            reopened = dm.open(dataset.id)
            assert reopened.status == "completed"

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_writer_idle_gap_still_finishes_cleanly(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            writer = dm.create("idle_gap_dataset", description="idle gap flow")
            writer.write(id=1, value=7.0, measurement=7.0 + 0.5j)
            time.sleep(1.1)
            writer.write(id=2, value=8.0, measurement=8.0 + 0.25j)

            dataset = writer.finish()
            assert dataset.status == "completed"

            reopened = dm.open(dataset.id)
            expected_rows = 2
            table = reopened.to_arrow()
            assert table.num_rows == expected_rows
            assert "__ds_record_id" not in table.column_names
            assert "__ds_record_id" not in reopened.to_polars().collect().columns

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_writer_abort_returns_aborted_dataset(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            writer = dm.create("abort_dataset", description="abort flow")
            writer.write(id=1, value=2.0, measurement=2.0 + 0.5j)
            dataset = writer.abort()

            assert dataset.name == "abort_dataset"
            assert dataset.status == "aborted"

            reopened = dm.open(dataset.id)
            assert reopened.status == "aborted"
            assert reopened.to_arrow().num_rows == 1

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_writer_drop_without_finalize_is_aborted(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            writer = dm.create("drop_abort_dataset", description="drop flow")
            writer.write(id=1, value=3.0, measurement=3.0 + 0.5j)
            del writer
            _ = gc.collect()

            self._wait_dataset_status(dm, "drop_abort_dataset", "aborted")

            reopened = dm.open(
                dm.list_all()[dm.list_all()["name"] == "drop_abort_dataset"].index[0]  # pyright: ignore[reportAny]
            )
            assert reopened.status == "aborted"
            assert reopened.to_arrow().num_rows == 1

            server_handle.shutdown()
            assert not server_handle.is_running

    def test_dataset_manual_close(self) -> None:
        """Test dataset writer with manual close."""
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            # Create dataset using context manager
            with dm.create("manual_test", description="Manual close test") as writer:
                writer.write(id=1, value=42.0, measurement=3.14 + 2j)

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

            # Create dataset with tags using context manager
            with dm.create(
                "tagged_test",
                description="Dataset with tags",
                tags=["test", "integration", "example"],
            ) as writer:
                writer.write(id=1, value=42.0, measurement=3.14 + 2j)

            # Verify dataset was created with tags
            datasets = dm.list_all()
            assert len(datasets) == 1
            dataset_info = datasets.iloc[0]
            assert dataset_info["name"] == "tagged_test"
            tags = dataset_info["tags"]  # pyright: ignore[reportAny]
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

            # Create dataset using context manager
            with dm.create(
                "schema_test", description="Schema inference test"
            ) as writer:
                # Write supported data types (int, float and complex)
                writer.write(
                    id=1,  # int (will be inferred as Int64)
                    value=3.14,  # float
                    measurement=2.5 + 1j,  # complex
                )

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

            # Create first dataset using context manager
            with dm.create("dataset1", description="First dataset") as writer1:
                writer1.write(id=1, value=100.0, measurement=1.0 + 2j)

            # Create second dataset using context manager
            with dm.create("dataset2", description="Second dataset") as writer2:
                writer2.write(id=2, value=200.0, measurement=2.0 + 3j)

            # Create third dataset with tags using context manager
            with dm.create(
                "dataset3", description="Third dataset", tags=["special"]
            ) as writer3:
                writer3.write(id=3, value=300.0, measurement=3.0 + 4j)

            # Verify all datasets were created
            datasets = dm.list_all()
            expected_dataset_count = 3
            assert len(datasets) == expected_dataset_count

            # Verify dataset names
            names = datasets["name"].tolist()
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

            # Create dataset using context manager
            with dm.create(
                "metadata_test", description="Original description"
            ) as writer:
                writer.write(id=1, value=42.0, measurement=3.14 + 2j)

            # Get dataset and verify metadata
            datasets = dm.list_all()
            dataset_id = datasets.index[0]  # pyright: ignore[reportAny]
            dataset = dm.open(dataset_id)  # pyright: ignore[reportAny]

            assert dataset.name == "metadata_test"
            assert dataset.description == "Original description"
            assert not dataset.favorite

            # Note: update_metadata, add_tags, remove_tags would be tested here
            # but they require async runtime context which needs additional setup

            # Explicitly shutdown the server
            server_handle.shutdown()
            assert not server_handle.is_running

    def test_open_missing_dataset_raises_semantic_dataset_error(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            workspace_path = Path(tmpdir) / "test_workspace"
            workspace, server_handle = fricon._core.serve_workspace(workspace_path)
            dm = workspace.dataset_manager

            with pytest.raises(fricon.FriconDatasetError) as exc_info:
                _ = dm.open(999999)

            exc = exc_info.value
            assert exc.code == "dataset_not_found"
            assert exc.message == "Dataset not found"

            server_handle.shutdown()
            assert not server_handle.is_running
