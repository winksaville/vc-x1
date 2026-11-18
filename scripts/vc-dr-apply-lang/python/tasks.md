| bump | sets the version in `pyproject.toml` ([Suffix scheme](agent-data/versioning.md#suffix-scheme)) | first, at each commit of a cycle | edit `pyproject.toml` |
| validate | formats the code, runs the lints, and runs every test | before each review, and before every commit | `uv run ruff format`, then `uv run ruff check` and `uv run pytest` |
| validate fast | runs the tests, stopping at the first failure | while a review iterates | `uv run pytest -x -q` |
