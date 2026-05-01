from __future__ import annotations

import fricon


def test_import() -> None:
    assert fricon.__name__ == "fricon"
    assert hasattr(fricon, "Column")
    assert hasattr(fricon, "FriconDatasetError")


def test_column_constructor() -> None:
    column = fricon.Column(
        float,
        unit="V",
        label="Voltage",
        hidden_by_default=True,
        chart_axis=True,
    )

    assert column.dtype is float
    assert column.unit == "V"
    assert column.label == "Voltage"
    assert column.hidden_by_default
    assert column.chart_axis
