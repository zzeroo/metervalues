## Evolution

- Next TDD case for business rule around readings
  - whether readings must not be chronological
  - duplicate reading dates should be allowed for the same meter instance. E.g. reading before huge water consumption and after to track the used water

## Refactoring

- refactor src/api/readings.rs 
  - move structs to src/models.rs
