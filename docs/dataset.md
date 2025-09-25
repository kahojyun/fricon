# Dataset

`fricon` uses [Arrow IPC format] to store datasets. A basic knowledge of Arrow
data structures can be helpful to understand how `fricon` works.

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

## How are datasets stored?

A dataset is exactly one Arrow table stored in [Arrow IPC format]. When a dataset
is created, the schema of the table is automatically inferred from the first row
of data written. This allows for flexible data collection without requiring
manual schema definition.

## Type inference

`fricon` MVP currently supports a focused set of data types optimized for scientific measurements and signal processing. The following table lists the supported types:

| Python type        | Dataset data type | Description                                  |
| ------------------ | ----------------- | -------------------------------------------- |
| [`float`][]        | `Float64`         | 64-bit floating point numbers                |
| [`complex`][]      | `Complex128`      | 128-bit complex numbers (real + imaginary)   |
| [`fricon.Trace`][] | `Trace`           | Time series data with various x-axis formats |

> **Note**: The MVP version intentionally limits type support to float and complex types for simplicity. Additional types (bool, int, str) will be supported in future releases.

### Supported trace variants

Trace data supports three different formats depending on how the x-axis (independent variable) is stored:

- **SimpleList**: Only y-values are stored, x-values are implicit indices (0, 1, 2, ...)
- **FixedStep**: Regular spacing with x₀ (starting point) and step size
- **VariableStep**: Arbitrary x-values stored alongside y-values

### Future extensions

Additional data types (bool, int, str, timestamps) will be supported in future versions. The current focus on float, complex, and trace types ensures optimal performance and correctness for the most common scientific use cases.

<!-- TODO: `pyarrow` and `polars` tips -->

[Arrow IPC format]: https://arrow.apache.org/docs/format/Columnar.html#serialization-and-interprocess-communication-ipc
