# Grand Pattern WASM

**WebAssembly port of the Grand Pattern mono-vibe system.** Runs in any browser.

## Build

```bash
wasm-pack build --target web --out-dir pkg
```

## Demo

```bash
# After building, serve the directory with any HTTP server
python3 -m http.server 8080
# Open http://localhost:8080/demo.html
```

## Usage from JavaScript

```javascript
import { WasmCellGraph, topology_ring } from './pkg/grand_pattern_wasm.js';

const graph = new WasmCellGraph();

// Add rooms with initial vibes
for (let i = 0; i < 20; i++) graph.add_room(Math.random());

// Connect with ring topology
const edges = topology_ring(20);
for (let i = 0; i < edges.length; i += 2) {
    graph.add_edge(edges[i], edges[i + 1], 1.0);
}

// Run simulation loop
function step() {
    graph.diffuse(0.1);   // spread vibes
    graph.tick();          // advance clock
    graph.learn();         // update surprise
    console.log('fleet vibe:', graph.fleet_vibe());
    requestAnimationFrame(step);
}
step();
```

## API

### `WasmCellGraph`

| Method | Description |
|--------|-------------|
| `new()` | Create empty graph |
| `add_room(vibe)` | Add room, returns id |
| `add_edge(from, to, weight)` | Add directed weighted edge |
| `tick()` | Advance simulation clock |
| `diffuse(rate)` | Diffuse vibes across edges |
| `learn()` | Update surprise values |
| `total_vibe()` | Sum of all room vibes |
| `fleet_vibe()` | Average vibe |
| `fleet_surprise()` | Average surprise |
| `room_vibe(id)` | Get room's vibe |
| `room_surprise(id)` | Get room's surprise |
| `room_count()` | Number of rooms |
| `tick_count()` | Current tick |
| `reset()` | Clear everything |

### Topology Generators

All return flat `[src, dst, src, dst, ...]` arrays.

| Function | Description |
|----------|-------------|
| `topology_chain(n)` | Linear chain |
| `topology_ring(n)` | Circular ring |
| `topology_star(n)` | Star from center (0) |
| `topology_small_world(n, p)` | Watts-Strogatz with rewiring prob `p` |

## License

MIT
