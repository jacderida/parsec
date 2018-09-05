// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use error::Error;
use gossip::Event;
use hash::Hash;
use id::SecretId;
use network_event::NetworkEvent;
use serialise;
use std::collections::btree_map::{self, BTreeMap, Entry};

pub(crate) struct PeerList<S: SecretId> {
    our_id: S,
    peers: BTreeMap<S::PublicId, BTreeMap<u64, Hash>>,
    // Map of Hash(peer_id) => peer_id
    peer_id_hashes: Vec<(Hash, S::PublicId)>,
}

impl<S: SecretId> PeerList<S> {
    /// Constructor of `PeerList`.
    pub fn new(our_id: S) -> Self {
        PeerList {
            our_id,
            peers: BTreeMap::new(),
            peer_id_hashes: vec![],
        }
    }

    /// Returns `our_id`.
    pub fn our_id(&self) -> &S {
        &self.our_id
    }

    /// Returns all sorted peer_ids.
    pub fn all_ids(&self) -> Vec<&S::PublicId> {
        self.peers.keys().collect()
    }

    /// Returns an unsorted map of Hash(peer_id) => peer_id
    pub fn peer_id_hashes(&self) -> &Vec<(Hash, S::PublicId)> {
        &self.peer_id_hashes
    }

    /// Returns an iterator of peers.
    pub fn iter(&self) -> btree_map::Iter<S::PublicId, BTreeMap<u64, Hash>> {
        self.peers.iter()
    }

    /// Returns `true` if the given peer is known.
    pub fn has_peer(&self, peer_id: &S::PublicId) -> bool {
        self.peers.contains_key(peer_id)
    }

    /// Adds a peer into the map.
    pub fn add_peer(&mut self, peer_id: S::PublicId) {
        let _ = self
            .peers
            .entry(peer_id.clone())
            .or_insert_with(BTreeMap::new);
        self.peer_id_hashes
            .push((Hash::from(serialise(&peer_id).as_slice()), peer_id));
    }

    /// Checks whether the input count becomes the super majority of the network.
    pub fn is_super_majority(&self, count: usize) -> bool {
        3 * count > 2 * self.peers.len()
    }

    /// Returns the hash of the last event created by this peer. Returns `None` if cannot find.
    pub fn last_event_hash(&self, peer_id: &S::PublicId) -> Option<&Hash> {
        self.peers
            .get(peer_id)
            .and_then(|events| events.values().rev().next())
    }

    /// Returns the hash of the indexed event.
    pub fn event_by_index(&self, peer_id: &S::PublicId, index: u64) -> Option<&Hash> {
        self.peers
            .get(peer_id)
            .and_then(|events| events.get(&index))
    }

    /// Adds event created by the peer. Returns an error if the creator is not known, or if we
    /// already held an event from this peer with this index, but that event's hash is different to
    /// the one being added (in which case `peers` is left unmodified).
    pub fn add_event<T: NetworkEvent>(
        &mut self,
        event: &Event<T, S::PublicId>,
    ) -> Result<(), Error> {
        if let Some(peer) = self.peers.get_mut(event.creator()) {
            match peer.entry(event.index()) {
                Entry::Occupied(entry) => {
                    if entry.get() != event.hash() {
                        return Err(Error::InvalidEvent);
                    }
                }
                Entry::Vacant(entry) => {
                    let _ = entry.insert(*event.hash());
                }
            }
            Ok(())
        } else {
            Err(Error::UnknownPeer)
        }
    }
}