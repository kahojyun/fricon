"""Generate large datasets for dataset stress testing.

This script creates scalar-heavy datasets sized for pressure-testing table,
filter, and chart flows without the extreme payload cost of per-row trace data.

Default stress cases:
- `testcase_dataset_stress_100k_points`
- `testcase_dataset_stress_250k_points`
- `testcase_dataset_stress_500k_points`
- `testcase_dataset_stress_1m_points`

Run:
    uv run python examples/dataset/create_stress_test_cases.py
    uv run python examples/dataset/create_stress_test_cases.py --counts 100k 500k 1m
    uv run python examples/dataset/create_stress_test_cases.py --counts 2000 --dataset-suffix smoke

The target workspace must already be initialized and have a running Fricon
server, matching the behavior of the other dataset example scripts.
"""

from __future__ import annotations

import argparse
import math
import os
from time import perf_counter
from typing import cast

from dotenv import find_dotenv, load_dotenv
from fricon import DatasetManager, Workspace

PRESET_POINT_COUNTS = (100_000, 250_000, 500_000, 1_000_000)


def _load_workspace_from_env() -> str | None:
    _ = load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def _parse_point_count(raw: str) -> int:
    normalized = raw.strip().lower().replace("_", "")
    multiplier = 1
    if normalized.endswith("k"):
        multiplier = 1_000
        normalized = normalized[:-1]
    elif normalized.endswith("m"):
        multiplier = 1_000_000
        normalized = normalized[:-1]

    if not normalized:
        message = f"Invalid point count: {raw!r}"
        raise argparse.ArgumentTypeError(message)

    try:
        value = int(normalized) * multiplier
    except ValueError as error:
        message = f"Invalid point count: {raw!r}"
        raise argparse.ArgumentTypeError(message) from error

    if value <= 0:
        message = f"Point count must be > 0: {raw!r}"
        raise argparse.ArgumentTypeError(message)
    return value


def _format_point_count_label(total_points: int) -> str:
    if total_points % 1_000_000 == 0:
        return f"{total_points // 1_000_000}m"
    if total_points % 1_000 == 0:
        return f"{total_points // 1_000}k"
    return str(total_points)


def _dedupe_preserve_order(values: list[int]) -> list[int]:
    seen: set[int] = set()
    ordered: list[int] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        ordered.append(value)
    return ordered


def _resolve_grid_width(total_points: int, requested_width: int | None) -> int:
    if requested_width is not None:
        return requested_width
    return max(250, min(1_000, math.isqrt(total_points)))


def _resolve_progress_every(total_points: int, requested_step: int | None) -> int:
    if requested_step is not None:
        return requested_step
    return min(100_000, max(10_000, total_points // 10))


def create_scalar_stress_case(
    manager: DatasetManager,
    *,
    total_points: int,
    dataset_prefix: str,
    dataset_suffix: str | None,
    grid_width: int | None,
    progress_every: int | None,
) -> None:
    count_label = _format_point_count_label(total_points)
    dataset_name = f"{dataset_prefix}_{count_label}_points"
    if dataset_suffix:
        dataset_name = f"{dataset_name}_{dataset_suffix}"

    resolved_grid_width = _resolve_grid_width(total_points, grid_width)
    resolved_grid_height = math.ceil(total_points / resolved_grid_width)
    resolved_progress_every = _resolve_progress_every(total_points, progress_every)

    print(
        f"Creating {dataset_name}: {total_points:,} rows, grid {resolved_grid_width}x{resolved_grid_height}"
    )

    start_time = perf_counter()
    with manager.create(
        dataset_name,
        description=f"Stress-test scalar dataset with {total_points:,} rows for dataset table, filter, and chart performance checks",
        tags=["testcase", "stress", "scalar", count_label],
    ) as writer:
        half_grid_width = max(1.0, resolved_grid_width / 2.0)
        half_grid_height = max(1.0, resolved_grid_height / 2.0)

        for row_index in range(total_points):
            idx_partition = row_index // 100_000
            idx_site = (row_index // 4_096) % 16
            idx_recipe = (row_index // 65_536) % 8
            idx_grid_x = row_index % resolved_grid_width
            idx_grid_y = row_index // resolved_grid_width

            normalized_x = idx_grid_x / max(1, resolved_grid_width - 1)
            normalized_y = idx_grid_y / max(1, resolved_grid_height - 1)
            radial_distance = math.sqrt(
                ((idx_grid_x - half_grid_width) / half_grid_width) ** 2
                + ((idx_grid_y - half_grid_height) / half_grid_height) ** 2
            )
            phase = row_index / 97.0

            writer.write(
                idx_partition=idx_partition,
                idx_recipe=idx_recipe,
                idx_site=idx_site,
                idx_grid_y=idx_grid_y,
                idx_grid_x=idx_grid_x,
                scalar_regular_time_s=row_index * 0.001,
                scalar_regular_signal_primary=(
                    math.sin(phase) + 0.2 * math.cos(phase / 11.0)
                ),
                scalar_regular_signal_secondary=(
                    math.cos(phase * 0.45 + idx_site * 0.15)
                    + normalized_y * 0.3
                    - idx_recipe * 0.02
                ),
                scalar_regular_heatmap_intensity=(
                    42.0
                    + normalized_x * 8.0
                    - normalized_y * 6.0
                    - radial_distance * 5.0
                    + 0.8 * math.sin((idx_grid_x + idx_grid_y) / 23.0)
                ),
                scalar_regular_margin_db=(
                    18.0
                    + idx_partition * 0.06
                    + idx_site * 0.03
                    - idx_recipe * 0.11
                    + normalized_x * 1.5
                ),
            )

            completed_rows = row_index + 1
            if (
                completed_rows % resolved_progress_every == 0
                or completed_rows == total_points
            ):
                elapsed_seconds = perf_counter() - start_time
                rows_per_second = completed_rows / max(elapsed_seconds, 1e-9)
                print(
                    f"  {dataset_name}: {completed_rows:,}/{total_points:,} rows written ({rows_per_second:,.0f} rows/s)"
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
        "--counts",
        nargs="+",
        type=_parse_point_count,
        default=list(PRESET_POINT_COUNTS),
        help=(
            "Point counts to generate. Accepts raw integers or shorthand such as "
            "100k and 1m. Defaults to 100k 250k 500k 1m."
        ),
    )
    _ = parser.add_argument(
        "--dataset-prefix",
        default="testcase_dataset_stress",
        help="Prefix for generated dataset names (default: testcase_dataset_stress)",
    )
    _ = parser.add_argument(
        "--dataset-suffix",
        default=None,
        help="Optional suffix appended to each dataset name",
    )
    _ = parser.add_argument(
        "--grid-width",
        type=int,
        default=None,
        help=(
            "Optional x-axis width for the synthetic grid. Defaults to an "
            "auto-derived value based on the row count."
        ),
    )
    _ = parser.add_argument(
        "--progress-every",
        type=int,
        default=None,
        help=(
            "Print progress every N written rows. Defaults to an auto-derived step "
            "per dataset size."
        ),
    )
    args = parser.parse_args()

    workspace_path = cast(str, args.workspace)
    point_counts = _dedupe_preserve_order(cast(list[int], args.counts))
    dataset_prefix = cast(str, args.dataset_prefix)
    dataset_suffix = cast(str | None, args.dataset_suffix)
    grid_width = cast(int | None, args.grid_width)
    progress_every = cast(int | None, args.progress_every)

    if not point_counts:
        parser.error("At least one point count must be provided")
    if grid_width is not None and grid_width <= 0:
        parser.error("--grid-width must be > 0")
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
    manager = ws.dataset_manager

    for total_points in point_counts:
        create_scalar_stress_case(
            manager,
            total_points=total_points,
            dataset_prefix=dataset_prefix,
            dataset_suffix=dataset_suffix,
            grid_width=grid_width,
            progress_every=progress_every,
        )


if __name__ == "__main__":
    main()
