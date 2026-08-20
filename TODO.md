## Evolution

## Refactoring

- refactor src/api/readings.rs 
  - move structs to src/models.rs
- test isolation and automatic cleanup
  - consider wrapping each test in a transaction and rolling it back automatically
