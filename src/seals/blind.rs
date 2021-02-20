// LNP/BP Core Library implementing LNPBP specifications & standards
// Written in 2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use bitcoin::hashes::{sha256d, Hash, HashEngine};
use bitcoin::secp256k1::rand::{thread_rng, RngCore};
use bitcoin::{OutPoint, Txid};

use crate::client_side_validation::{
    commit_strategy, CommitConceal, CommitEncodeWithStrategy,
};
use crate::commit_verify::CommitVerify;

/// Data required to generate or reveal the information about blinded
/// transaction outpoint
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    Display,
    Default,
    StrictEncode,
    StrictDecode,
)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate")
)]
#[display("{txid}:{vout}!{blinding}")]
pub struct OutpointReveal {
    /// Blinding factor preventing rainbow table bruteforce attack based on
    /// the existing blockchain txid set
    pub blinding: u64,

    /// Txid that should be blinded
    pub txid: Txid,

    /// Tx output number that should be blinded
    pub vout: u32,
}

impl From<OutpointReveal> for OutPoint {
    #[inline]
    fn from(reveal: OutpointReveal) -> Self {
        OutPoint::new(reveal.txid, reveal.vout as u32)
    }
}

impl From<OutPoint> for OutpointReveal {
    fn from(outpoint: OutPoint) -> Self {
        Self {
            blinding: thread_rng().next_u64(),
            txid: outpoint.txid,
            vout: outpoint.vout as u32,
        }
    }
}

impl From<OutPoint> for OutpointHash {
    fn from(outpoint: OutPoint) -> Self {
        OutpointReveal::from(outpoint).commit_conceal()
    }
}

impl CommitConceal for OutpointReveal {
    type ConcealedCommitment = OutpointHash;

    #[inline]
    fn commit_conceal(&self) -> Self::ConcealedCommitment {
        self.outpoint_hash()
    }
}

impl CommitVerify<OutpointReveal> for OutpointHash {
    fn commit(reveal: &OutpointReveal) -> Self {
        let mut engine = OutpointHash::engine();
        engine.input(&reveal.blinding.to_be_bytes()[..]);
        engine.input(&reveal.txid[..]);
        engine.input(&reveal.vout.to_be_bytes()[..]);
        OutpointHash::from_engine(engine)
    }
}

impl OutpointReveal {
    #[inline]
    pub fn outpoint_hash(&self) -> OutpointHash {
        OutpointHash::commit(self)
    }
}

hash_newtype!(
    OutpointHash,
    sha256d::Hash,
    32,
    doc = "Blind version of transaction outpoint"
);

impl strict_encoding::Strategy for OutpointHash {
    type Strategy = strict_encoding::strategies::HashFixedBytes;
}

impl CommitEncodeWithStrategy for OutpointHash {
    type Strategy = commit_strategy::UsingStrict;
}
