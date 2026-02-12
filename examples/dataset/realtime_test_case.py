"""Create one realtime dataset test case that covers multiple data types.

Run:
    uv run python examples/dataset/realtime_test_case.py
    uv run python examples/dataset/realtime_test_case.py --total-points 1800 --sleep-seconds 0.05
"""

from __future__ import annotations

import argparse
import math
import os
from time import sleep
from typing import cast

import numpy as np
from dotenv import find_dotenv, load_dotenv
from fricon import DatasetManager, Trace, Workspace


def _load_workspace_from_env() -> str | None:
    _ = load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def realtime_case_01_multitype_stream(
    manager: DatasetManager,
    *,
    total_points: int,
    sleep_seconds: float,
    scan_x_size: int,
    scan_y_size: int,
) -> None:
    """Single realtime case with scalar/index/complex/trace data types."""
    with manager.create(
        "testcase_realtime_01_multitype_stream",
        description=(
            "Single realtime case covering idx, scalar, complex, "
            "trace_variable, and trace_fixed columns"
        ),
        tags=["testcase", "realtime", "multitype"],
    ) as writer:
        trace_x_time_s = np.linspace(0.0, 1.0, 64)
        for tick_index in range(total_points):
            idx_scan_x = tick_index % scan_x_size
            idx_scan_y = (tick_index // scan_x_size) % scan_y_size
            idx_scan_cycle = tick_index // (scan_x_size * scan_y_size)
            idx_line = (idx_scan_y % 4) + 1
            scalar_regular_temp_c = (
                52.0 + 0.08 * idx_scan_x + 0.12 * idx_scan_y + 0.8 * idx_line
            )
            scalar_regular_pressure_kpa = 98.0 + 0.2 * math.sin(tick_index / 8)
            complex_impedance = (
                0.7 + 0.02 * idx_line + 0.005 * idx_scan_cycle
            ) * complex(math.cos(tick_index / 15), math.sin(tick_index / 15))
            trace_variable_vibration = 0.4 + 0.09 * np.sin(
                2 * np.pi * trace_x_time_s + tick_index / 18
            )
            trace_fixed_harmonic = np.array([1.0, 0.13, 0.07, 0.03]) * (
                1 + idx_line * 0.05
            )

            writer.write(
                # Keep index columns ordered from least-frequent to most-frequent.
                idx_scan_cycle=idx_scan_cycle,
                idx_scan_y=idx_scan_y,
                idx_scan_x=idx_scan_x,
                scalar_regular_tick_index=tick_index,
                scalar_regular_line_id=idx_line,
                scalar_regular_temp_c=scalar_regular_temp_c,
                scalar_regular_pressure_kpa=scalar_regular_pressure_kpa,
                complex_impedance_ohm=complex_impedance,
                trace_variable_vibration_g=Trace.variable_step(
                    trace_x_time_s, trace_variable_vibration
                ),
                trace_fixed_harmonic_ratio=Trace.fixed_step(
                    1.0, 1.0, trace_fixed_harmonic
                ),
            )
            sleep(sleep_seconds)


def main() -> None:
    default_workspace = _load_workspace_from_env() or ".dev/ws"
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument(
        "--workspace",
        default=default_workspace,
        help=("Workspace path (default: FRICON_WORKSPACE if set, otherwise .dev/ws)"),
    )
    _ = parser.add_argument(
        "--total-points",
        type=int,
        default=600,
        help="Total number of realtime rows to write (default: 600)",
    )
    _ = parser.add_argument(
        "--sleep-seconds",
        type=float,
        default=0.05,
        help="Sleep interval between writes in seconds (default: 0.05)",
    )
    _ = parser.add_argument(
        "--scan-x-size",
        type=int,
        default=24,
        help="Scan size for x dimension (default: 24)",
    )
    _ = parser.add_argument(
        "--scan-y-size",
        type=int,
        default=16,
        help="Scan size for y dimension (default: 16)",
    )
    args = parser.parse_args()
    workspace_path = cast(str, args.workspace)
    total_points = cast(int, args.total_points)
    sleep_seconds = cast(float, args.sleep_seconds)
    scan_x_size = cast(int, args.scan_x_size)
    scan_y_size = cast(int, args.scan_y_size)
    if total_points <= 0:
        parser.error("--total-points must be > 0")
    if sleep_seconds < 0:
        parser.error("--sleep-seconds must be >= 0")
    if scan_x_size <= 0:
        parser.error("--scan-x-size must be > 0")
    if scan_y_size <= 0:
        parser.error("--scan-y-size must be > 0")

    ws = Workspace.connect(workspace_path)
    manager = ws.dataset_manager
    realtime_case_01_multitype_stream(
        manager,
        total_points=total_points,
        sleep_seconds=sleep_seconds,
        scan_x_size=scan_x_size,
        scan_y_size=scan_y_size,
    )


if __name__ == "__main__":
    main()
