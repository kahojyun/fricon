"""Create explicit datasets for dataset feature testing.

The dataset names use `testcase_dataset_XX_<purpose>` so each case is obvious:
- testcase_dataset_01_basic_scalars_for_table_and_filters
- testcase_dataset_02_multi_index_for_cascade_filters
- testcase_dataset_03_complex_signal_for_complex_view
- testcase_dataset_04_trace_signal_for_line_and_trace_xy
- testcase_dataset_05_heatmap_grid_for_xy_heatmap
- testcase_dataset_06_column_name_variants_for_field_rendering

Run:
    uv run python examples/dataset/create_feature_test_cases.py

Column naming convention:
- `idx_*`: index-like dimensions for filtering/grouping.
- `scalar_regular_*`: regular scalar measurement columns.
- `trace_variable_*` / `trace_fixed_*`: trace columns.
- `complex_*`: complex-valued columns.
"""

from __future__ import annotations

import argparse
import math
import os

import numpy as np
from dotenv import find_dotenv, load_dotenv
from fricon import DatasetManager, Trace, Workspace


def _load_workspace_from_env() -> str | None:
    load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def case_01_basic_scalars(manager: DatasetManager) -> None:
    with manager.create(
        "testcase_dataset_01_basic_scalars_for_table_and_filters",
        description="Scalar columns for table rendering, sorting, and range filtering",
        tags=["testcase", "table", "filter", "scalar"],
    ) as writer:
        for scalar_regular_row_id in range(1, 31):
            scalar_regular_product_line_id = (scalar_regular_row_id % 3) + 1
            scalar_regular_test_temperature_c = 20 + (scalar_regular_row_id % 5) * 10
            scalar_regular_gain_db = (
                9.5 + 0.15 * scalar_regular_row_id + 0.4 * scalar_regular_product_line_id
            )
            scalar_regular_noise_dbm = -78 + 0.8 * math.sin(scalar_regular_row_id / 3)
            writer.write(
                scalar_regular_row_id=scalar_regular_row_id,
                scalar_regular_product_line_id=scalar_regular_product_line_id,
                scalar_regular_test_temperature_c=scalar_regular_test_temperature_c,
                scalar_regular_gain_db=scalar_regular_gain_db,
                scalar_regular_noise_dbm=scalar_regular_noise_dbm,
            )


def case_02_multi_index(manager: DatasetManager) -> None:
    with manager.create(
        "testcase_dataset_02_multi_index_for_cascade_filters",
        description="Multi-index fields to verify cascade filter behavior",
        tags=["testcase", "multi-index", "filter"],
    ) as writer:
        for idx_site in range(1, 4):
            for idx_tool in range(1, 4):
                for idx_recipe in range(1, 5):
                    for idx_wafer_slot in range(1, 6):
                        yield_pct = (
                            89 + idx_site * 1.8 + idx_tool * 0.9 + idx_recipe * 0.5
                        )
                        yield_pct -= idx_wafer_slot * 0.25
                        writer.write(
                            idx_site=idx_site,
                            idx_tool=idx_tool,
                            idx_recipe=idx_recipe,
                            idx_wafer_slot=idx_wafer_slot,
                            scalar_regular_yield_pct=yield_pct,
                        )


def case_03_complex_signal(manager: DatasetManager) -> None:
    with manager.create(
        "testcase_dataset_03_complex_signal_for_complex_view",
        description="Complex columns for real/imag/mag/arg visualization checks",
        tags=["testcase", "complex", "line", "scatter"],
    ) as writer:
        for idx_device in range(1, 4):
            for idx_sample in range(0, 120):
                scalar_regular_stimulus_freq_hz = 1e6 + idx_sample * 2e6
                amplitude = 0.2 + 0.03 * idx_device
                phase = idx_sample / 18 + idx_device * 0.25
                transfer_function_complex = amplitude * np.exp(1j * phase)
                writer.write(
                    idx_device=idx_device,
                    idx_sample=idx_sample,
                    scalar_regular_stimulus_freq_hz=scalar_regular_stimulus_freq_hz,
                    complex_transfer_function=transfer_function_complex,
                )


def case_04_trace_signal(manager: DatasetManager) -> None:
    with manager.create(
        "testcase_dataset_04_trace_signal_for_line_and_trace_xy",
        description="Trace columns for line mode and trace_xy scatter mode",
        tags=["testcase", "trace", "line", "scatter"],
    ) as writer:
        idx_time_ms_regular = np.linspace(0.0, 800.0, 160)
        for idx_run in range(1, 5):
            for idx_channel in range(1, 3):
                voltage_trace_v = (
                    3.3
                    - 0.001 * idx_time_ms_regular
                    + 0.03 * np.sin(idx_time_ms_regular / 60 + idx_run)
                )
                current_trace_a = 0.8 + 0.07 * np.cos(idx_time_ms_regular / 45 + idx_channel)
                writer.write(
                    idx_run=idx_run,
                    idx_channel=idx_channel,
                    scalar_regular_sample_interval_ms=5.0,
                    trace_variable_voltage_v=Trace.variable_step(
                        idx_time_ms_regular, voltage_trace_v
                    ),
                    trace_variable_current_a=Trace.variable_step(
                        idx_time_ms_regular, current_trace_a
                    ),
                    trace_fixed_harmonic_ratio=Trace.fixed_step(
                        1.0,
                        1.0,
                        np.array([1.0, 0.10, 0.05, 0.03]),
                    ),
                )


def case_05_heatmap_grid(manager: DatasetManager) -> None:
    with manager.create(
        "testcase_dataset_05_heatmap_grid_for_xy_heatmap",
        description="Regular x/y grid to verify heatmap mode and color scaling",
        tags=["testcase", "heatmap"],
    ) as writer:
        for idx_chamber in range(1, 3):
            for idx_grid_x_regular in range(0, 10):
                for idx_grid_y_regular in range(0, 10):
                    center_distance = math.sqrt(
                        (idx_grid_x_regular - 4.5) ** 2 + (idx_grid_y_regular - 4.5) ** 2
                    )
                    deposition_rate_nm_min = (
                        45 + idx_chamber * 1.3 - center_distance * 0.7
                    )
                    deposition_rate_nm_min += 0.9 * math.sin(
                        (idx_grid_x_regular + idx_grid_y_regular) / 2
                    )
                    writer.write(
                        idx_chamber=idx_chamber,
                        idx_grid_x_regular=idx_grid_x_regular,
                        idx_grid_y_regular=idx_grid_y_regular,
                        scalar_regular_deposition_rate_nm_min=deposition_rate_nm_min,
                    )


def case_06_column_name_variants(manager: DatasetManager) -> None:
    with manager.create(
        "testcase_dataset_06_column_name_variants_for_field_rendering",
        description="Column naming variants for field list and rendering checks",
        tags=["testcase", "field-name"],
    ) as writer:
        for scalar_regular_case_id in range(1, 11):
            writer.write_dict(
                {
                    "scalar_regular_case_id": scalar_regular_case_id,
                    "name_variant_space_value": scalar_regular_case_id * 10,
                    "name_variant_hyphen-value": scalar_regular_case_id * 100,
                    "nameVariantCamelValue": float(scalar_regular_case_id) / 10.0,
                    "scalar_regular_category_id": scalar_regular_case_id % 3,
                }
            )


def main() -> None:
    default_workspace = _load_workspace_from_env() or ".dev/ws"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--workspace",
        default=default_workspace,
        help=(
            "Workspace path (default: FRICON_WORKSPACE if set, otherwise .dev/ws)"
        ),
    )
    args = parser.parse_args()

    ws = Workspace.connect(args.workspace)
    manager = ws.dataset_manager

    case_01_basic_scalars(manager)
    case_02_multi_index(manager)
    case_03_complex_signal(manager)
    case_04_trace_signal(manager)
    case_05_heatmap_grid(manager)
    case_06_column_name_variants(manager)


if __name__ == "__main__":
    main()
