# Dataset

`fricon` datasets are table-shaped data collections. A basic knowledge of Arrow
data structures can be helpful because the Python API accepts Arrow-compatible
values and exposes Arrow-style tables.

## Public dataset contract

Each dataset contains one table. The first written row defines the table schema.
Later rows must use the same column names and compatible value types as that
first row. Missing columns, extra columns, and different value kinds are not
part of the supported write contract.

Supported public write values are currently `float`, `int` values converted to
`float`, `complex`, and trace values. `None` and nullable columns are not part
of the current write contract.

After writing at least one row, call `finish()` or `close()` to complete a
dataset successfully. A writer used as a context manager calls `close()` when
the block exits normally. Calling `abort()`, raising from the context manager
block, or dropping a writer before successful completion marks the dataset as
aborted. Finishing an empty writer does not create a completed dataset.

Users do not currently declare dataset semantics, scan axes, or logical indices
through the public API. The exact workspace file layout, internal metadata
files, chunk file names, and write-buffer thresholds are implementation details
rather than public storage contracts.

## [Apache Arrow](https://arrow.apache.org/docs/index.html)

You may be familiar with [pandas](https://pandas.pydata.org/), which is a
widely-used data manipulation library in Python. Arrow is a similar library
but with much stricter data types requirements. Each Arrow table comes with a
schema that specifies the data types of each column. Following are some key
classes in the python binding of Arrow:

- [`pyarrow.RecordBatch`][]: A record batch is a collection of arrays with the
  same length. Each record batch is associated with a schema.
- [`pyarrow.Array`][]: An array is a sequence of values with the same data
  type.
- [`pyarrow.Scalar`][]: A scalar is a single value with a data type.
- [`pyarrow.Schema`][]: A schema is a collection of fields. Each field
  corresponds to a column in a table.
- [`pyarrow.Field`][]: A field is a data type with a name.
- [`pyarrow.DataType`][]
- [`pyarrow.Table`][]: A helper type to unify representations of single and
  collection of record batches with the same schema.

## How datasets work

When a dataset is created, the table schema is automatically inferred from the
first row of data written. This allows for flexible data collection without
requiring manual schema definition.

## Write batching

Dataset writes are buffered automatically. This keeps the write API row-oriented
while reducing transport overhead for larger ingests. The exact buffering
thresholds are implementation details.

## Type inference

`fricon` currently supports a focused set of data types optimized for scientific measurements and signal processing. The following table lists the supported types:

| Python type              | Dataset data type | Description                                  |
| ------------------------ | ----------------- | -------------------------------------------- |
| [`float`][]              | `Float64`         | 64-bit floating point numbers                |
| [`int`][]                | `Float64`         | Converted to 64-bit floating point numbers   |
| [`complex`][]            | `Complex128`      | 128-bit complex numbers (real + imaginary)   |
| [`fricon.Trace`][]       | `Trace`           | Time series data with explicit x-axis values |
| list, NumPy, Arrow array | `Trace`           | Simple trace with implicit integer x indices |

> **Note**: The current release intentionally stores scalar columns as float or complex values.

### Supported trace variants

Trace data supports three different formats depending on how the x-axis (independent variable) is stored:

- **SimpleList**: pass a list, NumPy array, or Arrow array as the column value;
  only y-values are stored, and x-values are implicit indices (0, 1, 2, ...).
- **FixedStep**: use `fricon.Trace.fixed_step(x0, step, y)` for regular
  spacing with x0 and step size.
- **VariableStep**: use `fricon.Trace.variable_step(x, y)` for arbitrary
  x-values stored alongside y-values.
