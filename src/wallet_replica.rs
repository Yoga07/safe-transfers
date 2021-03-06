// Copyright 2020 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use super::{
    wallet::{Wallet, WalletSnapshot},
    Outcome, TernaryResult,
};
use log::debug;
#[cfg(feature = "simulated-payouts")]
use sn_data_types::Credit;
use sn_data_types::{
    CreditAgreementProof, Debit, Error, KnownGroupAdded, Money, PublicKey, ReplicaEvent, Result,
    SignedCredit, SignedDebit, TransferAgreementProof, TransferRegistered,
};
use std::collections::HashSet;
use threshold_crypto::{PublicKeySet, PublicKeyShare};

/// The Replica is the part of an AT2 system
/// that forms validating groups, and signs
/// individual transfers between wallets.
/// Replicas validate requests to debit an wallet, and
/// apply operations that has a valid "debit agreement proof"
/// from the group, i.e. signatures from a quorum of its peers.
/// Replicas don't initiate transfers or drive the algo - only Actors do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletReplica {
    /// The public key of the Wallet.
    id: PublicKey,
    /// The public key share of this Replica.
    replica_id: PublicKeyShare,
    /// The index of this Replica key share, in the group set.
    key_index: usize,
    /// The PK set of our peer Replicas.
    peer_replicas: PublicKeySet,
    /// PK sets of other known groups of Replicas.
    other_groups: HashSet<PublicKeySet>,
    /// All wallets that this Replica validates transfers for.
    wallet: Wallet,
    /// Ensures that invidual wallet's debit
    /// initiations (ValidateTransfer cmd) are sequential.
    pending_debit: Option<u64>,
}

impl WalletReplica {
    /// A new Replica instance from a history of events.
    pub fn from_history(
        id: PublicKey,
        replica_id: PublicKeyShare,
        key_index: usize,
        peer_replicas: PublicKeySet,
        events: Vec<ReplicaEvent>,
    ) -> Result<Self> {
        let mut instance = Self::from_snapshot(
            id,
            replica_id,
            key_index,
            peer_replicas,
            Default::default(),
            Wallet::new(id),
            None,
        );

        for e in events {
            instance.apply(e)?;
        }

        Ok(instance)
    }

    /// A new Replica instance from current state.
    pub fn from_snapshot(
        id: PublicKey,
        replica_id: PublicKeyShare,
        key_index: usize,
        peer_replicas: PublicKeySet,
        other_groups: HashSet<PublicKeySet>,
        wallet: Wallet,
        pending_debit: Option<u64>,
    ) -> Self {
        Self {
            id,
            replica_id,
            key_index,
            peer_replicas,
            other_groups,
            wallet,
            pending_debit,
        }
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Queries ----------------------------------
    /// -----------------------------------------------------------------

    ///
    pub fn balance(&self) -> Money {
        self.wallet.balance()
    }

    ///
    pub fn wallet(&self) -> Option<WalletSnapshot> {
        let wallet = self.wallet.to_owned();
        Some(wallet.into())
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Cmds -------------------------------------
    /// -----------------------------------------------------------------

    /// This is the one and only infusion of money to the system. Ever.
    /// It is carried out by the first node in the network.
    pub fn genesis<F: FnOnce() -> Result<PublicKey>>(
        &self,
        credit_proof: &CreditAgreementProof,
        past_key: F,
    ) -> Outcome<()> {
        // Genesis must be the first credit.
        if self.balance() != Money::zero() || self.pending_debit.is_some() {
            return Err(Error::InvalidOperation);
        }
        self.receive_propagated(credit_proof, past_key)
    }

    /// Adds a PK set for a a new group that we learn of.
    pub fn add_known_group(&self, group: PublicKeySet) -> Outcome<KnownGroupAdded> {
        if self.other_groups.contains(&group) {
            return Err(Error::DataExists);
        }
        Outcome::success(KnownGroupAdded { group })
    }

    /// For now, with test money there is no from wallet.., money is created from thin air.
    pub fn test_validate_transfer(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Outcome<()> {
        if signed_debit.sender() == signed_credit.recipient() {
            Err(Error::from("Sender and recipient are the same."))
        } else if signed_credit.id() != &signed_debit.credit_id()? {
            Err(Error::from("The credit does not correspond to the debit."))
        } else if signed_credit.amount() != signed_debit.amount() {
            Err(Error::from("Amounts must be equal."))
        } else {
            Outcome::success(())
        }
    }

    /// Step 1. Main business logic validation of a debit.
    pub fn validate(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Outcome<()> {
        let debit = &signed_debit.debit;
        let credit = &signed_credit.credit;

        // Always verify signature first! (as to not leak any information).
        if self
            .verify_actor_signature(&signed_debit, &signed_credit)
            .is_err()
        {
            return Outcome::rejected(Error::InvalidSignature);
        } else if debit.sender() == credit.recipient() {
            return Outcome::rejected(Error::from("Sender and recipient are the same."));
        } else if credit.id() != &debit.credit_id()? {
            return Outcome::rejected(Error::from("The credit does not correspond to the debit."));
        } else if credit.amount() != debit.amount() {
            return Outcome::rejected(Error::from("Amounts must be equal."));
        } else if debit.amount() == Money::zero() {
            return Outcome::rejected(Error::Unexpected(
                "Transfer amount must be more than zero.".to_string(),
            ));
        } else if self.wallet.id() != debit.sender() {
            return Outcome::rejected(Error::NoSuchSender);
        } else if self.pending_debit.is_none() && debit.id.counter != 0 {
            return Outcome::rejected(Error::from("out of order msg, actor's counter should be 0"));
        } else if let Some(counter) = self.pending_debit {
            if debit.id.counter != (counter + 1) {
                return Outcome::rejected(Error::from(format!(
                    "out of order msg, debit counter: {:?}, current counter: {:?}",
                    debit.id.counter, counter
                )));
            }
        } else if debit.amount() > self.balance() {
            return Outcome::rejected(Error::InsufficientBalance);
        }

        Outcome::success(())
    }

    /// Step 2. Validation of agreement, and order at debit source.
    pub fn register<F: FnOnce() -> Result<PublicKey>>(
        &self,
        transfer_proof: &TransferAgreementProof,
        past_key: F,
    ) -> Outcome<TransferRegistered> {
        debug!("Checking registered transfer");

        // Always verify signature first! (as to not leak any information).
        if self
            .verify_registered_proof(transfer_proof, past_key)
            .is_err()
        {
            return Err(Error::InvalidSignature);
        }

        let debit = &transfer_proof.signed_debit.debit;
        if self.wallet.next_debit() == debit.id().counter {
            Outcome::success(TransferRegistered {
                transfer_proof: transfer_proof.clone(),
            })
        } else {
            Outcome::rejected(Error::InvalidOperation) // from this place this code won't happen, but history validates the transfer is actually debits from it's owner.
        }
    }

    /// Step 3. Validation of TransferAgreementProof, and credit idempotency at credit destination.
    /// (Since this leads to a credit, there is no requirement on order.)
    pub fn receive_propagated<F: FnOnce() -> Result<PublicKey>>(
        &self,
        credit_proof: &CreditAgreementProof,
        past_key: F,
    ) -> Outcome<()> {
        // Always verify signature first! (as to not leak any information).
        self.verify_propagated_proof(credit_proof, past_key)?;
        if self.wallet.contains(&credit_proof.id()) {
            Outcome::no_change()
        } else {
            Outcome::success(())
        }
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Mutation ---------------------------------
    /// -----------------------------------------------------------------

    /// Mutation of state.
    /// There is no validation of an event, it (the cmd) is assumed to have
    /// been properly validated before the fact is established (event raised),
    /// and thus anything that breaks here, is a bug in the validation..
    pub fn apply(&mut self, event: ReplicaEvent) -> Result<()> {
        match event {
            ReplicaEvent::KnownGroupAdded(e) => {
                let _ = self.other_groups.insert(e.group);
                Ok(())
            }
            ReplicaEvent::TransferValidated(e) => {
                let debit = e.signed_debit.debit;
                self.pending_debit = Some(debit.id.counter);
                Ok(())
            }
            ReplicaEvent::TransferRegistered(e) => {
                let debit = e.transfer_proof.signed_debit.debit;
                self.wallet.apply_debit(Debit {
                    id: debit.id(),
                    amount: debit.amount(),
                })
            }
            ReplicaEvent::TransferPropagated(e) => {
                let credit = e.credit_proof.signed_credit.credit;
                self.wallet.apply_credit(credit)
            }
        }
    }

    /// Test-helper API to simulate Client CREDIT Transfers.
    #[cfg(feature = "simulated-payouts")]
    pub fn credit_without_proof(&mut self, credit: Credit) -> Result<()> {
        self.wallet.simulated_credit(credit)
    }

    /// Test-helper API to simulate Client DEBIT Transfers.
    #[cfg(feature = "simulated-payouts")]
    pub fn debit_without_proof(&mut self, debit: Debit) -> Result<()> {
        self.wallet.simulated_debit(debit)
    }

    /// -----------------------------------------------------------------
    /// ---------------------- Private methods --------------------------
    /// -----------------------------------------------------------------

    ///
    fn verify_actor_signature(
        &self,
        signed_debit: &SignedDebit,
        signed_credit: &SignedCredit,
    ) -> Result<()> {
        println!("Actor signature verification");
        let debit = &signed_debit.debit;
        let credit = &signed_credit.credit;
        let debit_bytes = match bincode::serialize(&debit) {
            Err(_) => return Err(Error::NetworkOther("Could not serialise debit".into())),
            Ok(bytes) => bytes,
        };
        let credit_bytes = match bincode::serialize(&credit) {
            Err(_) => return Err(Error::NetworkOther("Could not serialise credit".into())),
            Ok(bytes) => bytes,
        };

        let valid_debit = signed_debit
            .sender()
            .verify(&signed_debit.actor_signature, debit_bytes)
            .is_ok();

        println!("Debit is valid?: {:?}", valid_debit);
        let valid_credit = signed_debit
            .sender()
            .verify(&signed_credit.actor_signature, credit_bytes)
            .is_ok();
        println!("Credit is valid?: {:?}", valid_debit);

        if valid_debit && valid_credit && credit.id() == &debit.credit_id()? {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    /// Verify that this is a valid _registered_
    /// TransferAgreementProof, i.e. signed by our peers.
    fn verify_registered_proof<F: FnOnce() -> Result<PublicKey>>(
        &self,
        proof: &TransferAgreementProof,
        past_key: F,
    ) -> Result<()> {
        if proof.signed_credit.id() != &proof.signed_debit.credit_id()? {
            return Err(Error::NetworkOther(
                "Credit does not correspond with the debit.".into(),
            ));
        }
        // Check that the proof corresponds to a public key set of our peers.
        let debit_bytes = match bincode::serialize(&proof.signed_debit) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::NetworkOther("Could not serialise transfer".into())),
        };
        let credit_bytes = match bincode::serialize(&proof.signed_credit) {
            Ok(bytes) => bytes,
            Err(_) => return Err(Error::NetworkOther("Could not serialise transfer".into())),
        };
        // Check if proof is signed by our peers.
        let public_key = sn_data_types::PublicKey::Bls(self.peer_replicas.public_key());
        let valid_debit = public_key.verify(&proof.debit_sig, &debit_bytes).is_ok();
        let valid_credit = public_key.verify(&proof.credit_sig, &credit_bytes).is_ok();
        if valid_debit && valid_credit {
            return Ok(());
        }
        // Check if proof is signed with an older key
        let public_key = past_key()?;
        let valid_debit = public_key.verify(&proof.debit_sig, &debit_bytes).is_ok();
        let valid_credit = public_key.verify(&proof.credit_sig, &credit_bytes).is_ok();
        if valid_debit && valid_credit {
            return Ok(());
        }

        // If it's not signed with our peers' public key, we won't consider it valid.
        Err(Error::InvalidSignature)
    }

    /// Verify that this is a valid _propagated_
    /// TransferAgreementProof, i.e. signed by a group that we know of.
    fn verify_propagated_proof<F: FnOnce() -> Result<PublicKey>>(
        &self,
        proof: &CreditAgreementProof,
        past_key: F,
    ) -> Result<()> {
        // Check that the proof corresponds to a public key set of some Replicas.
        match bincode::serialize(&proof.signed_credit) {
            Err(_) => Err(Error::NetworkOther("Could not serialise transfer".into())),
            Ok(credit_bytes) => {
                // Check if it is from our group.
                let our_key = sn_data_types::PublicKey::Bls(self.peer_replicas.public_key());
                if our_key
                    .verify(&proof.debiting_replicas_sig, &credit_bytes)
                    .is_ok()
                {
                    return Ok(());
                }

                // Check if proof is signed with an older key
                let public_key = past_key()?;
                let valid_credit = public_key
                    .verify(&proof.debiting_replicas_sig, &credit_bytes)
                    .is_ok();
                if valid_credit {
                    return Ok(());
                }

                // TODO: Check retrospectively(using SectionProofChain) for known groups also
                // Check all known groups of Replicas.
                for set in &self.other_groups {
                    let debiting_replicas = sn_data_types::PublicKey::Bls(set.public_key());
                    let result =
                        debiting_replicas.verify(&proof.debiting_replicas_sig, &credit_bytes);
                    if result.is_ok() {
                        return Ok(());
                    }
                }
                // If we don't know the public key this was signed with, we won't consider it valid.
                Err(Error::InvalidSignature)
            }
        }
    }
}
