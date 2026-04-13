"""Generate one extreme heatmap dataset for stress testing.

By default this creates exactly one `1000 x 1000` heatmap-style dataset:
- `testcase_dataset_stress_heatmap_1000x1000`

Run:
    uv run python examples/dataset/create_heatmap_stress_test_case.py
    uv run python examples/dataset/create_heatmap_stress_test_case.py --width 2000 --height 500

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

DEFAULT_WIDTH = 1_000
DEFAULT_HEIGHT = 1_000
DEFAULT_SEED = 7


def _load_workspace_from_env() -> str | None:
    _ = load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def create_heatmap_stress_case(
    manager: DatasetManager,
    *,
    width: int,
    height: int,
    seed: int,
    dataset_suffix: str | None,
    progress_every: int | None,
) -> None:
    total_points = width * height
    dataset_name = f"testcase_dataset_stress_heatmap_{width}x{height}"
    if dataset_suffix:
        dataset_name = f"{dataset_name}_{dataset_suffix}"

    resolved_progress_every = progress_every or min(
        100_000, max(10_000, total_points // 10)
    )

    print(f"Creating {dataset_name}: {width} x {height} = {total_points:,} points")

    rng = np.random.default_rng(seed)
    # Build a smooth 2D random-walk field so neighboring cells remain correlated
    # while the global surface still has large-scale drift.
    x_steps: npt.NDArray[np.float64] = rng.normal(
        loc=0.0, scale=0.08, size=(height, width)
    )
    y_steps: npt.NDArray[np.float64] = rng.normal(
        loc=0.0, scale=0.08, size=(height, width)
    )
    field: npt.NDArray[np.float64] = np.cumsum(x_steps, axis=1) + np.cumsum(
        y_steps, axis=0
    )
    field = field - float(field.mean())  # pyright: ignore[reportAny]
    field = field / max(float(field.std()), 1e-9)  # pyright: ignore[reportAny]

    start_time = perf_counter()
    with manager.create(
        dataset_name,
        description=(
            f"Heatmap stress-test dataset with a {width} x {height} regular grid "
            f"({total_points:,} points)"
        ),
        tags=["testcase", "stress", "heatmap", f"{width}x{height}"],
    ) as writer:
        completed_rows = 0
        for idx_grid_y in range(height):
            for idx_grid_x in range(width):
                writer.write(
                    idx_grid_y=idx_grid_y,
                    idx_grid_x=idx_grid_x,
                    scalar_regular_heatmap_intensity=(
                        50.0 + float(field[idx_grid_y, idx_grid_x]) * 9.0  # pyright: ignore[reportAny]
                    ),
                )

                completed_rows += 1
                if (
                    completed_rows % resolved_progress_every == 0
                    or completed_rows == total_points
                ):
                    elapsed_seconds = perf_counter() - start_time
                    rows_per_second = completed_rows / max(elapsed_seconds, 1e-9)
                    print(
                        f"  {dataset_name}: {completed_rows:,}/{total_points:,} points written ({rows_per_second:,.0f} rows/s)"
                    )

    elapsed_seconds = perf_counter() - start_time
    rows_per_second = total_points / max(elapsed_seconds, 1e-9)
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
        "--width",
        type=int,
        default=DEFAULT_WIDTH,
        help=f"Heatmap width in points (default: {DEFAULT_WIDTH})",
    )
    _ = parser.add_argument(
        "--height",
        type=int,
        default=DEFAULT_HEIGHT,
        help=f"Heatmap height in points (default: {DEFAULT_HEIGHT})",
    )
    _ = parser.add_argument(
        "--seed",
        type=int,
        default=DEFAULT_SEED,
        help=f"Random seed for the heatmap walk (default: {DEFAULT_SEED})",
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
    width = cast(int, args.width)
    height = cast(int, args.height)
    seed = cast(int, args.seed)
    dataset_suffix = cast(str | None, args.dataset_suffix)
    progress_every = cast(int | None, args.progress_every)

    if width <= 0:
        parser.error("--width must be > 0")
    if height <= 0:
        parser.error("--height must be > 0")
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

    create_heatmap_stress_case(
        ws.dataset_manager,
        width=width,
        height=height,
        seed=seed,
        dataset_suffix=dataset_suffix,
        progress_every=progress_every,
    )


if __name__ == "__main__":
    main()
