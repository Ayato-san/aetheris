//! Daevanion connectivity rules confirmed by the user on 2026-11-26.
//! The acquired set is supplied by the caller; definitions never imply ownership.
use super::{binary::invalid, catalog::Result};
use rusqlite::Connection;
use std::collections::{BTreeMap, BTreeSet};

/// An undirected board with its permanent Start anchor. These predicates only
/// check connectivity, not level, currency, points or other source conditions.
pub struct Board {
    start: i64,
    neighbours: BTreeMap<i64, BTreeSet<i64>>,
}
impl Board {
    pub fn load(db: &Connection, board_id: i64) -> Result<Self> {
        let nodes: Vec<(i64, Option<String>, bool)> = db.prepare(
            "SELECT n.id,c.code,n.row_number IS NOT NULL AND n.column_number IS NOT NULL FROM daevanion_nodes n LEFT JOIN catalogue_codes c ON c.id=n.type_id WHERE n.board_id=?1"
        )?.query_map([board_id], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?.collect::<rusqlite::Result<_>>()?;
        let starts: Vec<_> = nodes
            .iter()
            .filter(|(_, kind, _)| kind.as_deref() == Some("EDaevanionNodeType::Start"))
            .map(|(id, _, _)| *id)
            .collect();
        if starts.len() != 1
            || nodes
                .iter()
                .any(|(_, kind, positioned)| kind.is_none() || !positioned)
        {
            return Err(
                invalid("Daevanion board requires one Start and explicit positions").into(),
            );
        }
        let mut neighbours: BTreeMap<_, BTreeSet<_>> = nodes
            .into_iter()
            .map(|(id, _, _)| (id, BTreeSet::new()))
            .collect();
        let edges: Vec<(i64,i64)> = db.prepare(
            "SELECT e.node_id,e.neighbour_id FROM daevanion_node_connections e JOIN daevanion_nodes n ON n.id=e.node_id WHERE n.board_id=?1"
        )?.query_map([board_id],|r|Ok((r.get(0)?,r.get(1)?)))?.collect::<rusqlite::Result<_>>()?;
        for (a, b) in edges {
            if !neighbours.contains_key(&a) || !neighbours.contains_key(&b) {
                return Err(invalid("Daevanion edge crosses boards").into());
            }
            neighbours.get_mut(&a).unwrap().insert(b);
            neighbours.get_mut(&b).unwrap().insert(a);
        }
        Ok(Self {
            start: starts[0],
            neighbours,
        })
    }
    pub fn start(&self) -> i64 {
        self.start
    }
    pub fn neighbours(&self, node: i64) -> Option<&BTreeSet<i64>> {
        self.neighbours.get(&node)
    }
    fn reachable(&self, acquired: &BTreeSet<i64>) -> BTreeSet<i64> {
        let mut visited = BTreeSet::from([self.start]);
        let mut pending = vec![self.start];
        while let Some(node) = pending.pop() {
            for neighbour in &self.neighbours[&node] {
                if acquired.contains(neighbour) && visited.insert(*neighbour) {
                    pending.push(*neighbour);
                }
            }
        }
        visited
    }
    /// Start is an anchor and may be omitted from the supplied acquired set.
    pub fn is_connected(&self, acquired: &BTreeSet<i64>) -> bool {
        acquired.is_subset(&self.reachable(acquired))
    }
    pub fn can_activate(&self, node: i64, acquired: &BTreeSet<i64>) -> bool {
        node != self.start
            && !acquired.contains(&node)
            && self.is_connected(acquired)
            && self.neighbours.get(&node).is_some_and(|adjacent| {
                adjacent
                    .iter()
                    .any(|id| *id == self.start || acquired.contains(id))
            })
    }
    pub fn can_refund(&self, node: i64, acquired: &BTreeSet<i64>) -> bool {
        if node == self.start || !acquired.contains(&node) || !self.is_connected(acquired) {
            return false;
        }
        let mut remaining = acquired.clone();
        remaining.remove(&node);
        self.is_connected(&remaining)
    }
    pub fn unreachable(&self) -> BTreeSet<i64> {
        let all = self.neighbours.keys().copied().collect();
        let visited = self.reachable(&all);
        self.neighbours
            .keys()
            .filter(|id| !visited.contains(id))
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn board(edges: &[(i64, i64)]) -> Board {
        let mut neighbours = BTreeMap::<_, BTreeSet<_>>::from([(1, BTreeSet::new())]);
        for (a, b) in edges {
            neighbours.entry(*a).or_default().insert(*b);
            neighbours.entry(*b).or_default().insert(*a);
        }
        Board {
            start: 1,
            neighbours,
        }
    }
    #[test]
    fn activation_follows_acquired_paths_and_rejects_unknown_or_disconnected_sets() {
        let board = board(&[(1, 2), (2, 3), (2, 4)]);
        assert!(board.can_activate(2, &BTreeSet::new()));
        assert!(!board.can_activate(3, &BTreeSet::new()));
        assert!(board.can_activate(3, &BTreeSet::from([2])));
        assert!(!board.can_activate(3, &BTreeSet::from([4])));
        assert!(!board.can_activate(99, &BTreeSet::from([2])));
        assert!(!board.can_activate(1, &BTreeSet::new()));
        assert!(!board.is_connected(&BTreeSet::from([99])));
    }
    #[test]
    fn refunds_preserve_all_branches_and_allow_a_remaining_path_around_a_cycle() {
        let tree = board(&[(1, 2), (2, 3), (2, 4)]);
        let acquired = BTreeSet::from([1, 2, 3, 4]);
        assert!(!tree.can_refund(2, &acquired));
        assert!(tree.can_refund(3, &acquired));
        assert!(!tree.can_refund(1, &acquired));
        assert!(!tree.can_refund(99, &acquired));
        assert!(tree.can_refund(2, &BTreeSet::from([2])));
        let cycle = board(&[(1, 2), (2, 3), (3, 4), (4, 1)]);
        assert!(cycle.can_refund(2, &acquired));
        assert!(!cycle.can_refund(2, &BTreeSet::from([2, 3])));
    }
}
