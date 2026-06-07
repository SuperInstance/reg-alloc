# reg-alloc

Register allocation with liveness analysis and graph coloring for compiler infrastructure.

## Features

- **Register file** — Physical register management with allocation and freeing
- **Liveness analysis** — Backward dataflow analysis with live ranges
- **Interference graph** — Build from liveness, query interference, degree
- **Graph coloring** — Chaitin-style allocation with simplification and spill heuristics
- **Spill management** — Slot allocation, load/store generation, frame size tracking
- **Zero dependencies** — Pure `std` implementation

## License

MIT
