# Fricon

Fricon is a framework for data collection automation.

## Current features

- Data storage.

## Usage

Install via PyPI:

```shell
pip install fricon
```

Initialize workspace via CLI:

```shell
fricon init path/to/workspace
```

Start the server:

```shell
fricon serve path/to/workspace
```

Create a dataset:

```python title="examples/simple/create.py"
--8<-- "examples/simple/create.py"
```

Query and open a dataset:

```python title="examples/simple/open.py"
--8<-- "examples/simple/open.py"
```
