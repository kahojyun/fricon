"""Generate one jagged heatmap dataset for non-shared X-grid testing.

This creates exactly one dataset:
- `testcase_dataset_08_heatmap_jagged_trace_for_non_shared_x_grid`

The dataset includes two trace quantities that are useful for heatmap testing:
- `trace_variable_jagged_intensity`: explicit per-row X arrays with jagged spacing
- `trace_fixed_row_scan_intensity`: per-row fixed-step scans with different x0/step

Run:
    uv run python examples/dataset/create_jagged_heatmap_test_case.py
"""

from __future__ import annotations

import argparse
import math
import os
from typing import cast

import numpy as np
from dotenv import find_dotenv, load_dotenv
from fricon import DatasetManager, Trace, Workspace

DATASET_NAME = "testcase_dataset_08_heatmap_jagged_trace_for_non_shared_x_grid"


def _load_workspace_from_env() -> str | None:
    _ = load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def _build_variable_scan_x(row_index: int) -> np.ndarray:
    base = 1e-10 * row_index
    if row_index % 3 == 1:
        offsets = np.array([0.0, 1.0e-10, 1.8e-10, 4.1e-10, 7.2e-10])
    elif row_index % 3 == 2:
        offsets = np.array([0.0, 0.8e-10, 2.7e-10, 3.3e-10, 6.9e-10, 8.4e-10])
    else:
        offsets = np.array([0.0, 1.6e-10, 2.2e-10, 5.5e-10])
    return base + offsets


def _build_variable_scan_values(
    x_values: np.ndarray,
    row_index: int,
    wafer_id: int,
) -> np.ndarray:
    scaled_x = x_values * 1e10
    envelope = 62.0 + wafer_id * 1.7 - row_index * 0.8
    ripple = 3.8 * np.sin(scaled_x * 0.9 + wafer_id * 0.35)
    tilt = -0.45 * scaled_x
    return envelope + ripple + tilt


def _build_fixed_scan_values(length: int, row_index: int, wafer_id: int) -> np.ndarray:
    sample_index = np.arange(length, dtype=np.float64)
    baseline = 54.0 + wafer_id * 2.1 - row_index * 0.65
    ripple = 4.2 * np.cos(sample_index / 1.7 + row_index * 0.4)
    shoulder = 0.9 * np.sin(sample_index * 0.8 + wafer_id * 0.25)
    return baseline + ripple + shoulder


def create_jagged_heatmap_case(manager: DatasetManager) -> None:
    print(f"Creating {DATASET_NAME}")

    with manager.create(
        DATASET_NAME,
        description=(
            "Trace heatmap test case with per-row jagged X arrays and per-row "
            "fixed-step scans using different x0/step values"
        ),
        tags=["testcase", "heatmap", "trace", "jagged", "non-shared-x-grid"],
    ) as writer:
        for idx_wafer in range(1, 3):
            for idx_scan_row in range(1, 7):
                variable_x = _build_variable_scan_x(idx_scan_row)
                variable_values = _build_variable_scan_values(
                    variable_x, idx_scan_row, idx_wafer
                )

                fixed_x0_m = (idx_scan_row - 1) * 1.4e-10 + idx_wafer * 0.3e-10
                fixed_step_m = 0.7e-10 + (idx_scan_row % 3) * 0.35e-10
                fixed_length = 4 + (idx_scan_row % 3)
                fixed_values = _build_fixed_scan_values(
                    fixed_length, idx_scan_row, idx_wafer
                )

                center_mm = 140.0 + idx_scan_row * 23.0 + idx_wafer * 4.5
                row_metric = 90.0 + idx_wafer * 2.5 - idx_scan_row * 1.3
                row_metric += 1.2 * math.sin(idx_scan_row / 2)

                writer.write(
                    idx_wafer=idx_wafer,
                    idx_scan_row=idx_scan_row,
                    idx_scan_center_mm=center_mm,
                    scalar_regular_row_metric=row_metric,
                    trace_variable_jagged_intensity=Trace.variable_step(
                        variable_x, variable_values
                    ),
                    trace_fixed_row_scan_intensity=Trace.fixed_step(
                        fixed_x0_m,
                        fixed_step_m,
                        fixed_values,
                    ),
                )

    print(f"Finished {DATASET_NAME}")


def main() -> None:
    default_workspace = _load_workspace_from_env() or ".dev/ws"
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument(
        "--workspace",
        default=default_workspace,
        help=("Workspace path (default: FRICON_WORKSPACE if set, otherwise .dev/ws)"),
    )
    args = parser.parse_args()
    workspace_path = cast(str, args.workspace)

    ws = Workspace.connect(workspace_path)
    create_jagged_heatmap_case(ws.dataset_manager)


if __name__ == "__main__":
    main()
