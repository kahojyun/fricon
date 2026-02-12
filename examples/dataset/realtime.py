"""Write a realtime-like monitoring dataset with semantic column names."""

from __future__ import annotations

import math
import os
from time import sleep

from dotenv import find_dotenv, load_dotenv
from fricon import DatasetManager, Workspace


def _load_workspace_from_env() -> str | None:
    load_dotenv(find_dotenv(usecwd=True), override=False)
    return os.getenv("FRICON_WORKSPACE")


def realtime_sensor_monitor(manager: DatasetManager) -> None:
    with manager.create(
        "factory_realtime_sensor_monitor",
        description="Realtime stream-style sensor data for UI write-status testing",
        tags=["seed", "realtime", "monitoring"],
    ) as writer:
        for minute_index in range(120):
            for production_line_id in range(1, 5):
                sleep(0.005)
                vibration_rms = 0.7 + 0.15 * math.sin(minute_index / 9 + production_line_id)
                motor_temp_c = 58 + 0.08 * minute_index + 1.3 * production_line_id
                defect_risk_score = 0.2 * vibration_rms + 0.02 * (motor_temp_c - 55)
                writer.write(
                    minute_index=minute_index,
                    production_line_id=production_line_id,
                    vibration_rms=vibration_rms,
                    motor_temp_c=motor_temp_c,
                    defect_risk_score=defect_risk_score,
                )


def main() -> None:
    workspace_path = _load_workspace_from_env() or ".dev/ws"
    ws = Workspace.connect(workspace_path)
    manager = ws.dataset_manager
    realtime_sensor_monitor(manager)


if __name__ == "__main__":
    main()
