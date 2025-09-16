from __future__ import annotations

import numpy as np
from fricon import Trace


def test_simple_list_float_sequence() -> None:
    t = Trace.simple_list([0.5, 1.5, 2.5])
    assert t is not None


def test_simple_list_complex_sequence() -> None:
    seq = [complex(1.0, -0.5), complex(0.0, 2.0), complex(-3.25, 4.75)]
    t = Trace.simple_list(seq)
    assert t is not None


def test_simple_list_numpy_float_array() -> None:
    arr = np.array([1.0, 2.0, 3.0], dtype=np.float64)
    t = Trace.simple_list(arr)
    assert t is not None


def test_simple_list_numpy_complex_array() -> None:
    arr = np.array([1 + 2j, -3.5 + 0.25j], dtype=np.complex128)
    t = Trace.simple_list(arr)
    assert t is not None


def test_variable_step_float() -> None:
    xs = [0.0, 0.5, 1.25]
    ys = [10.0, 11.0, 12.5]
    t = Trace.variable_step(xs, ys)
    assert t is not None


def test_variable_step_complex() -> None:
    xs = np.array([0.0, 1.0, 2.0], dtype=np.float64)
    ys = [complex(0.0, 1.0), complex(1.0, -1.0), complex(2.5, 3.5)]
    t = Trace.variable_step(xs, ys)
    assert t is not None


def test_variable_step_numpy_complex() -> None:
    xs = np.array([0.0, 1.0, 2.0, 3.0], dtype=np.float64)
    ys = np.array([1 + 0j, -1 + 2j, 0.5 - 0.25j, 3.0 + 4.0j], dtype=np.complex128)
    t = Trace.variable_step(xs, ys)
    assert t is not None


def test_fixed_step_float() -> None:
    t = Trace.fixed_step(0.0, 0.1, [1.0, 2.0, 3.0])
    assert t is not None


def test_fixed_step_complex_numpy() -> None:
    ys = np.array([1 + 0.5j, -2.25 + 3.75j, 0.0 + 1.0j], dtype=np.complex128)
    t = Trace.fixed_step(1.0, 0.5, ys)
    assert t is not None
