"""Generate one extreme single-line dataset for stress testing.

By default this creates one dataset representing a single series with
`1,000,000` scalar points:
- `testcase_dataset_stress_line_1m_points`

Run:
    uv run python examples/dataset/create_line_stress_test_case.py
    uv run python examples/dataset/create_line_stress_test_case.py --points 2000000

The target workspace must already be initialized and have a running Fricon
server, matching the behavior of the other dataset example scripts.
"""

from __future__ import annotations

import argparse
import os
from time import perf_counter
from typing import cast

import numpy as np
import numpy.typing as npt
from dotenv import find_dotenv, load_dotenv
from fricon import DatasetManager, Workspace

DEFAULT_POINTS = 1_000_000
DEFAULT_SEED = 11


def _load_workspace_from_env() -> str | None:
    _ = load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def _format_point_label(points: int) -> str:
    if points % 1_000_000 == 0:
        return f"{points // 1_000_000}m"
    if points % 1_000 == 0:
        return f"{points // 1_000}k"
    return str(points)


def create_line_stress_case(
    manager: DatasetManager,
    *,
    points: int,
    seed: int,
    dataset_suffix: str | None,
    progress_every: int | None,
) -> None:
    dataset_name = f"testcase_dataset_stress_line_{_format_point_label(points)}_points"
    if dataset_suffix:
        dataset_name = f"{dataset_name}_{dataset_suffix}"

    resolved_progress_every = progress_every or min(100_000, max(10_000, points // 10))

    print(f"Creating {dataset_name}: single series with {points:,} points")

    rng = np.random.default_rng(seed)
    step_noise: npt.NDArray[np.float64] = rng.normal(loc=0.0, scale=0.015, size=points)
    line_values: npt.NDArray[np.float64] = np.cumsum(step_noise)
    line_values = line_values - float(line_values[0])  # pyright: ignore[reportAny]

    start_time = perf_counter()
    with manager.create(
        dataset_name,
        description=f"Single-series line stress-test dataset with {points:,} scalar points generated from a random walk",
        tags=["testcase", "stress", "line", _format_point_label(points)],
    ) as writer:
        for sample_index in range(points):
            x_value = sample_index * 0.001

            writer.write(
                idx_series_id=0,
                scalar_regular_sample_index=sample_index,
                scalar_regular_x=x_value,
                scalar_regular_y=float(line_values[sample_index]),  # pyright: ignore[reportAny]
            )

            completed_rows = sample_index + 1
            if (
                completed_rows % resolved_progress_every == 0
                or completed_rows == points
            ):
                elapsed_seconds = perf_counter() - start_time
                rows_per_second = completed_rows / max(elapsed_seconds, 1e-9)
                print(
                    f"  {dataset_name}: {completed_rows:,}/{points:,} points written ({rows_per_second:,.0f} rows/s)"
                )

    elapsed_seconds = perf_counter() - start_time
    rows_per_second = points / max(elapsed_seconds, 1e-9)
    print(
        f"Finished {dataset_name} in {elapsed_seconds:.1f}s ({rows_per_second:,.0f} rows/s)"
    )


def main() -> None:
    default_workspace = _load_workspace_from_env() or ".dev/ws"
    parser = argparse.ArgumentParser(description=__doc__)
    _ = parser.add_argument(
        "--workspace",
        default=default_workspace,
        help=("Workspace path (default: FRICON_WORKSPACE if set, otherwise .dev/ws)"),
    )
    _ = parser.add_argument(
        "--points",
        type=int,
        default=DEFAULT_POINTS,
        help=f"Number of points in the single line series (default: {DEFAULT_POINTS})",
    )
    _ = parser.add_argument(
        "--seed",
        type=int,
        default=DEFAULT_SEED,
        help=f"Random seed for the line walk (default: {DEFAULT_SEED})",
    )
    _ = parser.add_argument(
        "--dataset-suffix",
        default=None,
        help="Optional suffix appended to the dataset name",
    )
    _ = parser.add_argument(
        "--progress-every",
        type=int,
        default=None,
        help="Print progress every N written rows",
    )
    args = parser.parse_args()

    workspace_path = cast(str, args.workspace)
    points = cast(int, args.points)
    seed = cast(int, args.seed)
    dataset_suffix = cast(str | None, args.dataset_suffix)
    progress_every = cast(int | None, args.progress_every)

    if points <= 0:
        parser.error("--points must be > 0")
    if progress_every is not None and progress_every <= 0:
        parser.error("--progress-every must be > 0")

    try:
        ws = Workspace.connect(workspace_path)
    except RuntimeError as error:
        parser.error(
            "Failed to connect to the workspace. Initialize the workspace and "
            + "ensure a Fricon server is running before using this script. "
            + f"Original error: {error}"
        )

    create_line_stress_case(
        ws.dataset_manager,
        points=points,
        seed=seed,
        dataset_suffix=dataset_suffix,
        progress_every=progress_every,
    )


if __name__ == "__main__":
    main()
