# Dataset

`fricon` uses [Arrow IPC format]
to store datasets. A basic knowledge of Arrow data structures can be helpful to
understand how `fricon` works.

## [Apache Arrow](https://arrow.apache.org/docs/index.html)

You may be familiar with [pandas](https://pandas.pydata.org/), which is a
widely-used data manipulation library in Python. Arrow is a similar library
but with much stricter data types requirements. Each Arrow table comes with a
schema that specifies the data types of each column. Following are some key
classes in the python binding of Arrow:

* [`pyarrow.RecordBatch`][]: A record batch is a collection of arrays with the
same length. Each record batch is associated with a schema.
* [`pyarrow.Array`][]: An array is a sequence of values with the same data
type.
* [`pyarrow.Scalar`][]: A scalar is a single value with a data type.
* [`pyarrow.Schema`][]: A schema is a collection of fields. Each field
corresponds to a column in a table.
* [`pyarrow.Field`][]: A field is a data type with a name.
* [`pyarrow.DataType`][]
* [`pyarrow.Table`][]: A helper type to unify representations of single and
collection of record batches with the same schema.

## How are datasets stored?

A dataset is exactly one Arrow table stored in [Arrow IPC format]. When a dataset
is created, the schema of the table must be determined first. In `fricon`,
users can specify a partial schema in
[`DatasetManager.create`][fricon.DatasetManager.create], and unspecified
columns will be inferred from the first row of the dataset.

## Type inference

`fricon` only tries to infer a subset of Arrow data types. The following table
lists the mapping between Python types and Arrow data types:

| Python type                                | Arrow data type          |
|--------------------------------------------|--------------------------|
| [`bool`][]                                 | [`pyarrow.bool_`][]      |
| [`int`][]                                  | [`pyarrow.int64`][]      |
| [`float`][]                                | [`pyarrow.float64`][]    |
| [`complex`][]                              | [`fricon.complex128`][]  |
| [`str`][]                                  | [`pyarrow.string`][]     |
| [`Sequence`][collections.abc.Sequence]     | [`pyarrow.list_`][]      |
| [`fricon.Trace`][]                         | [`fricon.trace_`][]      |

Notice that `fricon` defines custom data types for complex numbers and traces.
Users can use utility functions to convert these custom data types back to
Python types, or process them directly with `pyarrow` or `polars`.

If users want to store other data types, they need to construct [`pyarrow.Scalar`][] values
by themselves. `fricon` will store these values as is.

<!-- TODO: `pyarrow` and `polars` tips -->

[Arrow IPC format]: https://arrow.apache.org/docs/format/Columnar.html#serialization-and-interprocess-communication-ipc
