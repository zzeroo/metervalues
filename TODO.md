## Evolution

## Refactoring

- test isolation and automatic cleanup
  - consider wrapping each test in a transaction and rolling it back automatically
- refinement: the duplicate meter_number currently produces a 500, although semantically it should probably become 409 Conflict.
