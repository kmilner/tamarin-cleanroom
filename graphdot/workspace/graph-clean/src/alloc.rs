//! Global `n<K>` node-id / port allocation (BEHAVIOR.md §3e).
//!
//! Observed rule, byte-consistent over **all 12 022** corpus graphs
//! (`mine_ids.py`, 12022/12022): the graph uses one monotonic counter starting
//! at 0, and identifiers are handed out in **emission order** so that id order ==
//! file order. For each node, in order:
//!   * a **record** first consumes one id per record *cell* (its `<port>`s), in
//!     cell order (premises, then the info cell, then conclusions), and then one
//!     more id for the node itself — so a record with `k` cells occupies ids
//!     `p, p+1, …, p+k-1` (ports) and `p+k` (node);
//!   * every other node kind (**ellipse**, **plain** legend, **invtrapezium**, …)
//!     consumes exactly one id, the node id.
//!
//! Example (`n7` from a real graph): 4 premise + 1 info + 3 conclusion cells take
//! ports `n0…n6`, then the node is `n7`; the next ellipse is `n8`. The scheme is
//! independent of the term contents; it depends only on the emission order and
//! each node's cell count.

/// A single monotonic `n<K>` id source implementing the observed allocation.
#[derive(Debug, Default, Clone)]
pub struct NodeIdAllocator {
    next: usize,
}

impl NodeIdAllocator {
    pub fn new() -> Self {
        Self { next: 0 }
    }

    /// The next raw id value that would be handed out (for assertions/tests).
    pub fn peek(&self) -> usize {
        self.next
    }

    /// Take one id, formatted `n<K>`.
    fn take(&mut self) -> String {
        let s = format!("n{}", self.next);
        self.next += 1;
        s
    }

    /// Allocate ids for a **record** with `n_cells` cells: returns the `n_cells`
    /// port ids (in cell order) followed by the node id. Ports precede the node,
    /// matching the observed scheme.
    pub fn record(&mut self, n_cells: usize) -> RecordIds {
        let ports = (0..n_cells).map(|_| self.take()).collect();
        let node = self.take();
        RecordIds { ports, node }
    }

    /// Allocate the single id of a non-record node (ellipse / plain / shaped).
    pub fn node(&mut self) -> String {
        self.take()
    }
}

/// The ids a record occupies: one `port` per cell, then the `node` id.
#[derive(Debug, Clone)]
pub struct RecordIds {
    pub ports: Vec<String>,
    pub node: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_takes_ports_then_node() {
        let mut a = NodeIdAllocator::new();
        // A record with 8 cells (4 prem + 1 info + 3 concl): ports n0..n7, node n8.
        let r = a.record(8);
        assert_eq!(r.ports, ["n0", "n1", "n2", "n3", "n4", "n5", "n6", "n7"]);
        assert_eq!(r.node, "n8");
        // next ellipse is n9.
        assert_eq!(a.node(), "n9");
    }

    #[test]
    fn interleaved_matches_observed_sequence() {
        // Reproduce the id sequence of the real graph in BEHAVIOR.md §3e:
        // record(8)->n8, ellipse->n9, ellipse->n?, record(5)->n15, ...
        let mut a = NodeIdAllocator::new();
        assert_eq!(a.record(8).node, "n8");
        assert_eq!(a.node(), "n9"); // ellipse
        assert_eq!(a.node(), "n10"); // ellipse
        let r = a.record(5); // ports n11..n15? no: n11..n15 is 5 -> node n16
        assert_eq!(r.ports, ["n11", "n12", "n13", "n14", "n15"]);
        assert_eq!(r.node, "n16");
    }
}
