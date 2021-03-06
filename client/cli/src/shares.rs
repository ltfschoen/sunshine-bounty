use clap::Clap;
use core::fmt::{
    Debug,
    Display,
};
use substrate_subxt::{
    sp_core::crypto::Ss58Codec,
    system::System,
    Runtime,
};
use sunshine_bounty_client::org::{
    AccountShare,
    Org,
    Org as Shares,
    OrgClient as SharesClient,
};
use sunshine_client_utils::{
    crypto::ss58::Ss58,
    Result,
};

#[derive(Clone, Debug, Clap)]
pub struct SharesIssueCommand {
    pub organization: u64,
    pub dest: String,
    pub shares: u64,
}

impl SharesIssueCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let account: Ss58<R> = self.dest.parse()?;
        let event = client
            .issue_shares(
                self.organization.into(),
                account.0,
                self.shares.into(),
            )
            .await?;
        println!(
            "{} shares minted for account {:?} in the context of Org {}",
            event.shares, event.who, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesBatchIssueCommand {
    pub organization: u64,
    pub new_accounts: Vec<AccountShare>,
}

impl SharesBatchIssueCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let accounts = self
            .new_accounts
            .iter()
            .map(|acc_share| -> Result<_> {
                let account: Ss58<R> = acc_share.0.parse()?;
                let amount_issued: R::Shares = (acc_share.1).into();
                Ok((account.0, amount_issued))
            })
            .collect::<Result<Vec<_>>>()?;
        let event = client
            .batch_issue_shares(self.organization.into(), accounts.as_slice())
            .await?;
        println!(
            "{} new shares minted in the context of Org {}",
            event.total_new_shares_minted, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesBatchBurnCommand {
    pub organization: u64,
    pub old_accounts: Vec<AccountShare>,
}

impl SharesBatchBurnCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let accounts = self
            .old_accounts
            .iter()
            .map(|acc_share| -> Result<_> {
                let account: Ss58<R> = acc_share.0.parse()?;
                let amount_burned: R::Shares = (acc_share.1).into();
                Ok((account.0, amount_burned))
            })
            .collect::<Result<Vec<_>>>()?;
        let event = client
            .batch_issue_shares(self.organization.into(), accounts.as_slice())
            .await?;
        println!(
            "{} new shares minted in the context of Org {}",
            event.total_new_shares_minted, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesBurnCommand {
    pub organization: u64,
    pub burner: String,
    pub shares: u64,
}

impl SharesBurnCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let account: Ss58<R> = self.burner.parse()?;
        let event = client
            .issue_shares(
                self.organization.into(),
                account.0,
                self.shares.into(),
            )
            .await?;
        println!(
            "{} shares burned from account {:?} in the context of Org {}",
            event.shares, event.who, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesReserveCommand {
    pub organization: u64,
    pub who: String,
}

impl SharesReserveCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let account: Ss58<R> = self.who.parse()?;
        let event = client
            .reserve_shares(self.organization.into(), &account.0)
            .await?;
        println!(
            "Account {} reserves {:?} shares in the context of Org {}",
            event.who, event.amount_reserved, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesUnReserveCommand {
    pub organization: u64,
    pub who: String,
}

impl SharesUnReserveCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let account: Ss58<R> = self.who.parse()?;
        let event = client
            .unreserve_shares(self.organization.into(), &account.0)
            .await?;
        println!(
            "Account {} unreserves {:?} shares in the context of Org {}",
            event.who, event.amount_unreserved, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesLockCommand {
    pub organization: u64,
    pub who: String,
}

impl SharesLockCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let account: Ss58<R> = self.who.parse()?;
        let event = client
            .lock_shares(self.organization.into(), &account.0)
            .await?;
        println!(
            "Locked shares for Account {} in the context of Org {}",
            event.who, event.organization
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Clap)]
pub struct SharesUnLockCommand {
    pub organization: u64,
    pub who: String,
}

impl SharesUnLockCommand {
    pub async fn exec<R: Runtime + Shares, C: SharesClient<R>>(
        &self,
        client: &C,
    ) -> Result<()>
    where
        <R as System>::AccountId: Ss58Codec,
        <R as Org>::OrgId: From<u64> + Display,
        <R as Org>::Shares: From<u64> + Display,
    {
        let account: Ss58<R> = self.who.parse()?;
        let event = client
            .unlock_shares(self.organization.into(), &account.0)
            .await?;
        println!(
            "Unlocked shares for Account {} in the context of Org {}",
            event.who, event.organization
        );
        Ok(())
    }
}
