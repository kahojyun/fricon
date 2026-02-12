"""Generate meaningful demo datasets for analysis and chart testing.

This script seeds multiple datasets that cover common chart workflows in Fricon:
- line chart from scalar values
- heatmap from grid-like values
- scatter from scalar x/y pairs
- complex-number visualization
- trace-based line/scatter visualization

Run:
    uv run python examples/dataset/create.py
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


def seed_retail_daily_kpi(manager: DatasetManager) -> None:
    """Line/scatter friendly KPI dataset with clear business semantics."""
    with manager.create(
        "retail_daily_kpi",
        description="Daily retail KPI data for line/scatter chart tests",
        tags=["seed", "line", "scatter", "kpi"],
    ) as writer:
        for day_index in range(1, 31):
            seasonality = 1.0 + 0.08 * math.sin(day_index / 4)
            for store_id in range(1, 6):
                for channel_id in (1, 2, 3):
                    ad_spend_usd = (1500 + 40 * day_index + 120 * store_id) * seasonality
                    units_sold = (
                        180
                        + 6 * day_index
                        + 25 * store_id
                        + 18 * channel_id
                        + 20 * math.cos(day_index / 5)
                    )
                    conversion_rate_pct = 2.1 + 0.2 * channel_id + 0.04 * store_id
                    writer.write(
                        day_index=day_index,
                        store_id=store_id,
                        channel_id=channel_id,
                        ad_spend_usd=ad_spend_usd,
                        units_sold=units_sold,
                        revenue_usd=units_sold * 18.5 * seasonality,
                        conversion_rate_pct=conversion_rate_pct,
                    )


def seed_wafer_param_heatmap(manager: DatasetManager) -> None:
    """Heatmap-friendly wafer parameter grid dataset."""
    with manager.create(
        "wafer_param_heatmap",
        description="Grid data for heatmap chart tests in semiconductor scenarios",
        tags=["seed", "heatmap", "manufacturing"],
    ) as writer:
        for lot_id in range(1, 4):
            for wafer_id in range(1, 5):
                center_x = 6.0
                center_y = 6.0
                for die_x in range(0, 13):
                    for die_y in range(0, 13):
                        radius = math.sqrt((die_x - center_x) ** 2 + (die_y - center_y) ** 2)
                        threshold_shift_mv = (
                            12
                            + 0.8 * lot_id
                            + 0.4 * wafer_id
                            + 0.7 * radius
                            + 1.2 * math.sin((die_x + die_y) / 3)
                        )
                        writer.write(
                            lot_id=lot_id,
                            wafer_id=wafer_id,
                            die_x=die_x,
                            die_y=die_y,
                            threshold_shift_mv=threshold_shift_mv,
                        )


def seed_rf_impedance_sweep(manager: DatasetManager) -> None:
    """Complex-valued frequency sweep dataset for complex mode charts."""
    with manager.create(
        "rf_impedance_sweep",
        description="Complex impedance sweep data for line/scatter complex view tests",
        tags=["seed", "complex", "line", "scatter", "rf"],
    ) as writer:
        for device_id in range(1, 5):
            for temperature_c in (25, 60, 85):
                for freq_hz in np.linspace(1e6, 2.4e9, 120):
                    phase = (freq_hz / 2.4e9) * math.pi * (1 + device_id * 0.05)
                    magnitude = 0.2 + 0.03 * device_id + 0.0008 * temperature_c
                    s11_complex = magnitude * np.exp(1j * phase)
                    writer.write(
                        device_id=device_id,
                        temperature_c=temperature_c,
                        freq_hz=freq_hz,
                        s11_complex=s11_complex,
                    )


def seed_battery_cycle_traces(manager: DatasetManager) -> None:
    """Trace-based dataset for line and trace-xy scatter chart tests."""
    with manager.create(
        "battery_cycle_traces",
        description="Battery cycle traces for trace line/scatter chart tests",
        tags=["seed", "trace", "line", "scatter", "battery"],
    ) as writer:
        time_axis = np.linspace(0.0, 1800.0, 240)
        for cell_id in range(1, 5):
            for cycle_index in range(1, 10):
                # Synthetic but realistic trend: voltage decays slowly as current fluctuates.
                voltage = 4.2 - 0.0013 * time_axis - 0.005 * cycle_index
                voltage += 0.01 * np.sin(time_axis / 120)
                current = 1.4 + 0.08 * np.cos(time_axis / 90 + cell_id)
                writer.write(
                    cell_id=cell_id,
                    cycle_index=cycle_index,
                    ambient_temp_c=24 + 0.4 * cell_id,
                    voltage_trace_v=Trace.variable_step(time_axis, voltage),
                    current_trace_a=Trace.variable_step(time_axis, current),
                    soc_end_pct=max(0.0, 100 - cycle_index * 8),
                )


def seed_power_grid_state(manager: DatasetManager) -> None:
    """High-dimensional dataset useful for index filtering + chart slicing."""
    with manager.create(
        "power_grid_state_matrix",
        description="Multi-index operating states for filtering and chart slice tests",
        tags=["seed", "multi-index", "filtering"],
    ) as writer:
        for region_id in range(1, 4):
            for substation_id in range(1, 6):
                for hour_of_day in range(0, 24):
                    demand_mw = (
                        200
                        + 35 * region_id
                        + 8 * substation_id
                        + 45 * math.sin((hour_of_day - 6) / 24 * 2 * math.pi)
                    )
                    power_factor = 0.92 + 0.03 * math.cos(hour_of_day / 24 * 2 * math.pi)
                    phase_angle_complex = np.exp(1j * (hour_of_day / 24 * math.pi))
                    harmonic_trace = Trace.fixed_step(
                        50.0,
                        50.0,
                        np.array([1.0, 0.11, 0.06, 0.03, 0.015]) * demand_mw / 300,
                    )
                    writer.write(
                        region_id=region_id,
                        substation_id=substation_id,
                        hour_of_day=hour_of_day,
                        demand_mw=demand_mw,
                        power_factor=power_factor,
                        phase_angle_complex=phase_angle_complex,
                        harmonic_trace=harmonic_trace,
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

    seed_retail_daily_kpi(manager)
    seed_wafer_param_heatmap(manager)
    seed_rf_impedance_sweep(manager)
    seed_battery_cycle_traces(manager)
    seed_power_grid_state(manager)


if __name__ == "__main__":
    main()
