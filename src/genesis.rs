// Copyright 2021 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

use crate::{Error, Result, SignerTrait};
use sn_data_types::{Credit, CreditAgreementProof, Money, PublicKey, SignedCredit};
use std::collections::BTreeMap;
use threshold_crypto::{PublicKeySet, SecretKeySet, SignatureShare};

/*
/// Produces a genesis balance for a new network.
#[allow(unused)]
pub fn get_random_genesis(balance: u64, id: PublicKey) -> Result<CreditAgreementProof> {
    let threshold = 0;
    // Nothing comes before genesis, it is a paradox
    // that it comes from somewhere. In other words, it is
    // signed over from a "ghost", the keys generated are "ghost" keys,
    // they come from nothing and can't be verified.
    // They are unimportant and will be thrown away,
    // thus the source of random is also unimportant.
    let mut rng = rand::thread_rng();
    let bls_secret_key = SecretKeySet::random(threshold, &mut rng);
    get_genesis(
        balance,
        id,
        bls_secret_key.public_keys(),
        bls_secret_key.secret_key_share(threshold),
    )
}
 */

/// Produces a genesis balance for a new network.
pub async fn get_genesis(
    balance: u64,
    id: PublicKey,
    peer_replicas: PublicKeySet,
    signer: &Box<dyn SignerTrait>,
) -> Result<CreditAgreementProof> {
    let credit = Credit {
        id: Default::default(),
        amount: Money::from_nano(balance),
        recipient: id,
        msg: "genesis".to_string(),
    };

    // actor instances' signatures over > credit <

    let serialised_credit = bincode::serialize(&credit)
        .map_err(|_| Error::Serialisation("Could not serialise credit".to_string()))?;

    let mut credit_sig_shares: BTreeMap<i32, SignatureShare> = BTreeMap::new();
    let credit_sig_share: SignatureShare = signer.sign_with_secret_key_share(&serialised_credit).await?;
    let _ = credit_sig_shares.insert(0, credit_sig_share);

    // Combine shares to produce the main signature.
    let actor_signature = sn_data_types::Signature::Bls(
        peer_replicas
            .combine_signatures(&credit_sig_shares)
            .map_err(|_| Error::CannotAggregate)?,
    );

    let signed_credit = SignedCredit {
        credit,
        actor_signature,
    };

    // replicas signatures over > signed_credit <

    let serialised_credit = bincode::serialize(&signed_credit)
        .map_err(|_| Error::Serialisation("Could not serialise signed_credit".to_string()))?;

    let mut credit_sig_shares: BTreeMap<i32, SignatureShare> = BTreeMap::new();
    let credit_sig_share: SignatureShare = signer.sign_with_secret_key_share(&serialised_credit).await?;
    let _ = credit_sig_shares.insert(0, credit_sig_share);

    let debiting_replicas_sig = sn_data_types::Signature::Bls(
        peer_replicas
            .combine_signatures(&credit_sig_shares)
            .map_err(|_| Error::CannotAggregate)?,
    );

    Ok(CreditAgreementProof {
        signed_credit,
        debiting_replicas_sig,
        debiting_replicas_keys: peer_replicas,
    })
}

/// Test only. Produces a genesis balance for a new network.
pub fn get_multi_genesis(
    balance: u64,
    id: PublicKey,
    secret_key_set: SecretKeySet,
) -> Result<CreditAgreementProof> {
    let credit = Credit {
        id: Default::default(),
        amount: Money::from_nano(balance),
        recipient: id,
        msg: "genesis".to_string(),
    };

    // actor instances' signatures over > credit <

    let serialised_credit = bincode::serialize(&credit)
        .map_err(|_| Error::Serialisation("Could not serialise credit".to_string()))?;

    let mut credit_sig_shares = BTreeMap::new();

    for i in 0..secret_key_set.threshold() + 1 {
        let secret_key = secret_key_set.secret_key_share(i);
        let credit_sig_share = secret_key.sign(serialised_credit.clone());
        let _ = credit_sig_shares.insert(0, credit_sig_share);
    }

    let peer_replicas = secret_key_set.public_keys();

    // Combine shares to produce the main signature.
    let actor_signature = sn_data_types::Signature::Bls(
        peer_replicas
            .combine_signatures(&credit_sig_shares)
            .map_err(|_| Error::CannotAggregate)?,
    );

    let signed_credit = SignedCredit {
        credit,
        actor_signature,
    };

    // replicas signatures over > signed_credit <

    let serialised_credit = bincode::serialize(&signed_credit)
        .map_err(|_| Error::Serialisation("Could not serialise signed_credit".to_string()))?;

    let mut credit_sig_shares = BTreeMap::new();

    for i in 0..secret_key_set.threshold() + 1 {
        let secret_key = secret_key_set.secret_key_share(i);
        let credit_sig_share = secret_key.sign(serialised_credit.clone());
        let _ = credit_sig_shares.insert(0, credit_sig_share);
    }

    let debiting_replicas_sig = sn_data_types::Signature::Bls(
        peer_replicas
            .combine_signatures(&credit_sig_shares)
            .map_err(|_| Error::CannotAggregate)?,
    );

    Ok(CreditAgreementProof {
        signed_credit,
        debiting_replicas_sig,
        debiting_replicas_keys: peer_replicas,
    })
}
