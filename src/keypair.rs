// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use safe_nd::{ClientFullId as Keypair, PublicId, PublicKey, Signature};
use std::sync::Arc;

/// TODO: Mapping from upper layer node/client "FullId"
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActorKeypair {
    /// Represents a network client.
    Client(Arc<Keypair>),
    /// Represents a network node.
    Node(Arc<Keypair>),
}

impl ActorKeypair {
    /// Creates a client full ID.
    pub fn client(keypair: Keypair) -> Self {
        Self::Client(Arc::new(keypair))
    }

    /// Creates a node full ID.
    pub fn node(keypair: Keypair) -> Self {
        Self::Node(Arc::new(keypair))
    }

    /// Signs a given message using the App / Client full id as required.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        match self {
            Self::Client(client_keypair) => client_keypair.sign(msg),
            Self::Node(node_keypair) => node_keypair.sign(msg),
        }
    }

    /// Returns a corresponding public key.
    pub fn public_key(&self) -> PublicKey {
        match self {
            Self::Client(client_keypair) => *client_keypair.public_id().public_key(),
            Self::Node(node_keypair) => *node_keypair.public_id().public_key(),
        }
    }
}
