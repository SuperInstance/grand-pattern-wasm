use wasm_bindgen::prelude::*;

/// A mono-vibe cell graph that runs entirely in WASM.
#[wasm_bindgen]
pub struct WasmCellGraph {
    rooms: Vec<f64>,
    surprises: Vec<f64>,
    edges_src: Vec<usize>,
    edges_dst: Vec<usize>,
    edges_weight: Vec<f64>,
    tick_count: u32,
}

#[wasm_bindgen]
impl WasmCellGraph {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        WasmCellGraph {
            rooms: Vec::new(),
            surprises: Vec::new(),
            edges_src: Vec::new(),
            edges_dst: Vec::new(),
            edges_weight: Vec::new(),
            tick_count: 0,
        }
    }

    /// Add a room with an initial vibe. Returns the room id.
    pub fn add_room(&mut self, vibe: f64) -> usize {
        let id = self.rooms.len();
        self.rooms.push(vibe);
        self.surprises.push(0.0);
        id
    }

    /// Add a directed weighted edge between two rooms.
    pub fn add_edge(&mut self, from: usize, to: usize, weight: f64) {
        self.edges_src.push(from);
        self.edges_dst.push(to);
        self.edges_weight.push(weight);
    }

    /// Advance the simulation clock by one tick.
    pub fn tick(&mut self) {
        self.tick_count += 1;
    }

    /// Diffuse vibes across edges at the given rate.
    /// Each edge transfers `rate * weight * (src_vibe - dst_vibe)` from src to dst.
    pub fn diffuse(&mut self, rate: f64) {
        let n = self.edges_src.len();
        let mut deltas = vec![0.0f64; self.rooms.len()];

        for i in 0..n {
            let src = self.edges_src[i];
            let dst = self.edges_dst[i];
            let w = self.edges_weight[i];
            if src < self.rooms.len() && dst < self.rooms.len() {
                let flow = rate * w * (self.rooms[src] - self.rooms[dst]);
                deltas[src] -= flow;
                deltas[dst] += flow;
            }
        }

        for i in 0..self.rooms.len() {
            self.rooms[i] += deltas[i];
        }
    }

    /// Learn: update surprise values based on how much vibes changed.
    /// Surprise tracks absolute change magnitude per room.
    pub fn learn(&mut self) {
        // In this simplified model, surprise is the absolute difference
        // between current vibe and what diffusion would predict.
        // We approximate: surprise = |room_vibe - fleet_vibe|
        let avg = self.fleet_vibe();
        for i in 0..self.rooms.len() {
            let old = self.surprises[i];
            let new_surprise = (self.rooms[i] - avg).abs();
            // Exponential moving average
            self.surprises[i] = 0.8 * old + 0.2 * new_surprise;
        }
    }

    /// Sum of all room vibes.
    pub fn total_vibe(&self) -> f64 {
        self.rooms.iter().sum()
    }

    /// Average vibe across all rooms (fleet vibe).
    pub fn fleet_vibe(&self) -> f64 {
        if self.rooms.is_empty() {
            return 0.0;
        }
        self.total_vibe() / self.rooms.len() as f64
    }

    /// Average surprise across all rooms.
    pub fn fleet_surprise(&self) -> f64 {
        if self.surprises.is_empty() {
            return 0.0;
        }
        self.surprises.iter().sum::<f64>() / self.surprises.len() as f64
    }

    /// Get vibe of a specific room.
    pub fn room_vibe(&self, id: usize) -> f64 {
        if id < self.rooms.len() {
            self.rooms[id]
        } else {
            0.0
        }
    }

    /// Get surprise of a specific room.
    pub fn room_surprise(&self, id: usize) -> f64 {
        if id < self.surprises.len() {
            self.surprises[id]
        } else {
            0.0
        }
    }

    /// Number of rooms in the graph.
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Current tick count.
    pub fn tick_count(&self) -> u32 {
        self.tick_count
    }

    /// Reset the graph to empty state.
    pub fn reset(&mut self) {
        self.rooms.clear();
        self.surprises.clear();
        self.edges_src.clear();
        self.edges_dst.clear();
        self.edges_weight.clear();
        self.tick_count = 0;
    }
}

impl Default for WasmCellGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ── Topology generators ──────────────────────────────────────────────
// All return a flat Vec<usize> of [src, dst, src, dst, ...] edge pairs.

/// Chain topology: 0→1→2→...→(n-1)
#[wasm_bindgen]
pub fn topology_chain(n: usize) -> Vec<usize> {
    let mut edges = Vec::new();
    for i in 0..n.saturating_sub(1) {
        edges.push(i);
        edges.push(i + 1);
    }
    edges
}

/// Ring topology: 0→1→2→...→(n-1)→0
#[wasm_bindgen]
pub fn topology_ring(n: usize) -> Vec<usize> {
    let mut edges = Vec::new();
    if n == 0 {
        return edges;
    }
    for i in 0..n {
        edges.push(i);
        edges.push((i + 1) % n);
    }
    edges
}

/// Star topology: center (0) connects to all others.
#[wasm_bindgen]
pub fn topology_star(n: usize) -> Vec<usize> {
    let mut edges = Vec::new();
    for i in 1..n {
        edges.push(0);
        edges.push(i);
        edges.push(i);
        edges.push(0);
    }
    edges
}

/// Watts-Strogatz small-world topology with rewiring probability p.
/// Uses a simple deterministic approach for WASM compatibility.
#[wasm_bindgen]
pub fn topology_small_world(n: usize, p: f64) -> Vec<usize> {
    let mut edges = Vec::new();
    if n < 4 {
        return topology_ring(n);
    }

    // Start with a ring lattice (each node connects to 2 nearest neighbors)
    for i in 0..n {
        for offset in 1..=2 {
            let j = (i + offset) % n;
            edges.push(i);
            edges.push(j);
        }
    }

    // Rewire edges with probability p using a simple hash-based approach
    // (no external RNG needed for WASM)
    let mut edges_src: Vec<usize> = Vec::new();
    let mut edges_dst: Vec<usize> = Vec::new();

    let mut seed: u64 = 42;
    for chunk in edges.chunks(2) {
        let src = chunk[0];
        let dst = chunk[1];

        // Simple LCG pseudo-random
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let rand_val = ((seed >> 33) as f64) / (1u64 << 31) as f64;

        if rand_val < p {
            // Rewire to a random different node
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let new_dst = ((seed >> 33) as usize) % n;
            if new_dst != src {
                edges_src.push(src);
                edges_dst.push(new_dst);
            } else {
                edges_src.push(src);
                edges_dst.push(dst);
            }
        } else {
            edges_src.push(src);
            edges_dst.push(dst);
        }
    }

    let mut result = Vec::new();
    for i in 0..edges_src.len() {
        result.push(edges_src[i]);
        result.push(edges_dst[i]);
    }
    result
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructor_empty() {
        let g = WasmCellGraph::new();
        assert_eq!(g.room_count(), 0);
        assert_eq!(g.tick_count(), 0);
    }

    #[test]
    fn test_add_room_incrementing_ids() {
        let mut g = WasmCellGraph::new();
        assert_eq!(g.add_room(0.5), 0);
        assert_eq!(g.add_room(0.8), 1);
        assert_eq!(g.add_room(0.3), 2);
        assert_eq!(g.room_count(), 3);
    }

    #[test]
    fn test_add_edge_stores() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_room(2.0);
        g.add_edge(0, 1, 0.5);
        // Verify edge has effect via diffusion: flow = rate * weight * (src - dst) = 1.0 * 0.5 * (1.0 - 2.0) = -0.5
        // So room 0 gains 0.5, room 1 loses 0.5
        g.diffuse(1.0);
        assert!((g.room_vibe(0) - 1.5).abs() < 1e-10);
        assert!((g.room_vibe(1) - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_diffuse_changes_vibes() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_room(0.0);
        g.add_edge(0, 1, 1.0);
        let v0_before = g.room_vibe(0);
        g.diffuse(0.5);
        assert_ne!(g.room_vibe(0), v0_before);
    }

    #[test]
    fn test_conservation() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_room(2.0);
        g.add_room(3.0);
        g.add_edge(0, 1, 1.0);
        g.add_edge(1, 2, 1.0);
        g.add_edge(2, 0, 1.0);
        let total_before = g.total_vibe();
        g.diffuse(0.5);
        let total_after = g.total_vibe();
        assert!((total_before - total_after).abs() < 1e-10);
    }

    #[test]
    fn test_tick_increments() {
        let mut g = WasmCellGraph::new();
        assert_eq!(g.tick_count(), 0);
        g.tick();
        assert_eq!(g.tick_count(), 1);
        g.tick();
        assert_eq!(g.tick_count(), 2);
    }

    #[test]
    fn test_learn_updates_surprise() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_room(0.0);
        g.add_edge(0, 1, 1.0);
        g.learn();
        // Surprises should be non-zero since vibes differ from average
        assert!(g.room_surprise(0) > 0.0 || g.room_surprise(1) > 0.0);
    }

    #[test]
    fn test_total_vibe() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_room(2.0);
        g.add_room(3.0);
        assert!((g.total_vibe() - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_fleet_vibe() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_room(3.0);
        assert!((g.fleet_vibe() - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_fleet_surprise() {
        let mut g = WasmCellGraph::new();
        g.add_room(0.5);
        g.add_room(0.5);
        // With uniform vibes, surprise should be low
        g.learn();
        assert!(g.fleet_surprise() < 1.0);
    }

    #[test]
    fn test_room_vibe_correct() {
        let mut g = WasmCellGraph::new();
        g.add_room(0.42);
        assert!((g.room_vibe(0) - 0.42).abs() < 1e-10);
        assert_eq!(g.room_vibe(999), 0.0); // out of bounds
    }

    #[test]
    fn test_room_surprise_correct() {
        let mut g = WasmCellGraph::new();
        g.add_room(0.5);
        assert_eq!(g.room_surprise(0), 0.0); // initial
    }

    #[test]
    fn test_room_count() {
        let mut g = WasmCellGraph::new();
        assert_eq!(g.room_count(), 0);
        g.add_room(1.0);
        assert_eq!(g.room_count(), 1);
    }

    #[test]
    fn test_reset() {
        let mut g = WasmCellGraph::new();
        g.add_room(1.0);
        g.add_edge(0, 0, 1.0);
        g.tick();
        g.reset();
        assert_eq!(g.room_count(), 0);
        assert_eq!(g.tick_count(), 0);
    }

    #[test]
    fn test_topology_chain() {
        let edges = topology_chain(4);
        assert_eq!(edges, vec![0, 1, 1, 2, 2, 3]);
    }

    #[test]
    fn test_topology_ring() {
        let edges = topology_ring(4);
        assert_eq!(edges, vec![0, 1, 1, 2, 2, 3, 3, 0]);
    }

    #[test]
    fn test_topology_ring_zero() {
        let edges = topology_ring(0);
        assert!(edges.is_empty());
    }

    #[test]
    fn test_topology_star() {
        let edges = topology_star(3);
        // 0↔1 and 0↔2
        assert_eq!(edges, vec![0, 1, 1, 0, 0, 2, 2, 0]);
    }

    #[test]
    fn test_topology_small_world() {
        let edges = topology_small_world(10, 0.0);
        // With p=0, should be same as ring lattice with k=2
        // 10 nodes × 2 forward edges = 20 edges = 40 values
        assert_eq!(edges.len(), 40);
    }
}
