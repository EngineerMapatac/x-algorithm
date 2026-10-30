use super::Store;
use crate::hydration::batch::{HydrationBatch, RawHydrationBatch};
use crate::hydration::fetcher::{AnyFetcher, Fetcher, Owner};
use crate::hydration::hydrator::{Edge, UserPair};
use crate::hydration::inventory::Answer;
use crate::hydration::Hydrator;

impl Store {
    pub(super) fn claim_viewer_edge(&mut self, node: Hydrator, keys: Vec<u64>) -> Vec<u64> {
        let pairs: Vec<UserPair> = keys
            .into_iter()
            .filter_map(|key| self.viewer_pair(key))
            .collect();
        match self.edge_fetcher_mut(node) {
            Some(fetcher) => fetcher
                .claim(pairs)
                .into_iter()
                .map(|pair| pair.destination)
                .collect(),
            None => Vec::new(),
        }
    }

    fn viewer_pair(&self, destination: u64) -> Option<UserPair> {
        Some(UserPair {
            source: self.viewer_id?,
            destination,
        })
    }

    pub(super) fn edge_fetcher(&self, node: Hydrator) -> Option<&Fetcher<bool, UserPair>> {
        let edge = node.edge()?;
        Some(
            self.edges
                .get(usize::from(edge as u8))?
                .of(self.owner(node)),
        )
    }

    pub(super) fn edge_fetcher_mut(
        &mut self,
        node: Hydrator,
    ) -> Option<&mut Fetcher<bool, UserPair>> {
        let (edge, owner) = (node.edge()?, self.owner(node));
        Some(self.edges.get_mut(usize::from(edge as u8))?.of_mut(owner))
    }

    pub(super) fn answer(&self, node: Hydrator, key: u64) -> Option<Answer> {
        if node.is_edge() {
            return Some(self.edge_fetcher(node)?.answer(self.viewer_pair(key)?));
        }
        Some(self.fetcher(node)?.answer(key))
    }

    pub(super) fn edge_holds(&self, node: Hydrator, key: u64) -> Option<bool> {
        self.edge_fetcher(node)?
            .get(self.viewer_pair(key)?)
            .copied()
    }

    pub(super) fn land_edge(
        &mut self,
        owner: Owner,
        edge: Edge,
        source: u64,
        destinations: &[u64],
        answers: RawHydrationBatch<bool>,
    ) {
        let pair = |destination| UserPair {
            source,
            destination,
        };
        let claimed: Vec<UserPair> = destinations.iter().copied().map(pair).collect();
        let answers = answers
            .into_hydrated()
            .into_iter()
            .map(|(destination, holds)| (pair(destination), holds))
            .collect();
        if let Some(fetchers) = self.edges.get_mut(usize::from(edge as u8)) {
            fetchers
                .of_mut(owner)
                .land(&claimed, HydrationBatch::from_hydrated(answers));
        }
    }
}
