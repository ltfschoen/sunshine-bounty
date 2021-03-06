use crate::traits::{
    AccessGenesis,
    AccessProfile,
    VerifyShape,
};
use codec::{
    Decode,
    Encode,
};
use frame_support::Parameter;
use sp_runtime::{
    traits::Zero,
    RuntimeDebug,
};
use sp_std::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone, Encode, Decode, RuntimeDebug)]
/// share profile reserves the total share amount every time but (might) have a limit on total reservations
pub struct ShareProfile<Shares> {
    /// The total number of shares owned by this participant
    total: Shares,
    /// The reference count for the number of votes that this is used, initialized at 0
    times_reserved: u32,
    /// Tells us if the shares can be used in another vote
    locked: bool,
}

impl<
        Shares: Copy
            + Default
            + Parameter
            + sp_std::ops::Add<Output = Shares>
            + sp_std::ops::Sub<Output = Shares>
            + Zero
            + From<u32>,
    > Default for ShareProfile<Shares>
{
    /// The default is 1 shares for convenient usage of the vote module for flat votes
    fn default() -> ShareProfile<Shares> {
        ShareProfile {
            total: Shares::zero() + 1u32.into(),
            times_reserved: 0u32,
            locked: false,
        }
    }
}

impl<
        Shares: Copy
            + Default
            + Parameter
            + sp_std::ops::Add<Output = Shares>
            + sp_std::ops::Sub<Output = Shares>
            + Zero
            + From<u32>,
    > ShareProfile<Shares>
{
    pub fn total(&self) -> Shares {
        self.total
    }

    pub fn times_reserved(&self) -> u32 {
        self.times_reserved
    }

    pub fn is_zero(&self) -> bool {
        self.total == Shares::zero()
    }

    pub fn new_shares(total: Shares) -> ShareProfile<Shares> {
        ShareProfile {
            total,
            ..Default::default()
        }
    }

    pub fn add_shares(self, amount: Shares) -> ShareProfile<Shares> {
        let total = self.total + amount;
        ShareProfile { total, ..self }
    }

    pub fn subtract_shares(self, amount: Shares) -> ShareProfile<Shares> {
        let total = self.total - amount;
        ShareProfile { total, ..self }
    }

    pub fn increment_times_reserved(self) -> ShareProfile<Shares> {
        let times_reserved = self.times_reserved + 1u32;
        ShareProfile {
            times_reserved,
            ..self
        }
    }

    pub fn decrement_times_reserved(self) -> ShareProfile<Shares> {
        let times_reserved = self.times_reserved - 1u32;
        ShareProfile {
            times_reserved,
            ..self
        }
    }

    pub fn lock(self) -> ShareProfile<Shares> {
        ShareProfile {
            locked: true,
            ..self
        }
    }

    pub fn unlock(self) -> ShareProfile<Shares> {
        ShareProfile {
            locked: false,
            ..self
        }
    }

    pub fn is_unlocked(&self) -> bool {
        !self.locked
    }
}

impl<Shares: Copy + sp_std::ops::AddAssign + Zero> AccessProfile<Shares>
    for ShareProfile<Shares>
{
    fn total(&self) -> Shares {
        self.total
    }
}

#[derive(PartialEq, Eq, Default, Clone, Encode, Decode, RuntimeDebug)]
/// The account ownership for the share genesis
pub struct SimpleShareGenesis<AccountId, Shares> {
    total: Shares,
    account_ownership: Vec<(AccountId, Shares)>,
}

impl<
        AccountId: Clone,
        Shares: Copy + sp_std::ops::AddAssign + Zero + PartialEq,
    > AccessGenesis<AccountId, Shares>
    for SimpleShareGenesis<AccountId, Shares>
{
    fn total(&self) -> Shares {
        self.total
    }
    fn account_ownership(&self) -> Vec<(AccountId, Shares)> {
        self.account_ownership.clone()
    }
}

impl<
        AccountId: Parameter,
        Shares: Copy + sp_std::ops::AddAssign + Zero + PartialEq,
    > From<Vec<(AccountId, Shares)>> for SimpleShareGenesis<AccountId, Shares>
{
    fn from(
        genesis: Vec<(AccountId, Shares)>,
    ) -> SimpleShareGenesis<AccountId, Shares> {
        let mut total: Shares = Shares::zero();
        let mut dedup_genesis = genesis;
        dedup_genesis.dedup(); // deduplicated
        for account_shares in dedup_genesis.clone() {
            total += account_shares.1;
        }
        SimpleShareGenesis {
            total,
            account_ownership: dedup_genesis,
        }
    }
}

impl<
        AccountId: Parameter,
        Shares: Copy + sp_std::ops::AddAssign + Zero + PartialEq,
    > VerifyShape for SimpleShareGenesis<AccountId, Shares>
{
    fn verify_shape(&self) -> bool {
        let mut sum: Shares = Shares::zero();
        for ac in self.account_ownership.iter() {
            sum += ac.1
        }
        sum == self.total
    }
}
