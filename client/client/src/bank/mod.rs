mod subxt;

pub use subxt::*;

use crate::{
    error::Error,
    org::Org,
};
use substrate_subxt::{
    system::System,
    Runtime,
    SignedExtension,
    SignedExtra,
};
use sunshine_client_utils::{
    async_trait,
    Client,
    Result,
};

#[async_trait]
pub trait BankClient<T: Runtime + Bank>: Client<T> {
    async fn open_org_bank_account(
        &self,
        seed: BalanceOf<T>,
        hosting_org: <T as Org>::OrgId,
        bank_operator: Option<<T as System>::AccountId>,
    ) -> Result<OrgBankAccountOpenedEvent<T>>;
}

#[async_trait]
impl<T, C> BankClient<T> for C
where
    T: Runtime + Bank,
    <<T::Extra as SignedExtra<T>>::Extra as SignedExtension>::AdditionalSigned:
        Send + Sync,
    C: Client<T>,
{
    async fn open_org_bank_account(
        &self,
        seed: BalanceOf<T>,
        hosting_org: <T as Org>::OrgId,
        bank_operator: Option<<T as System>::AccountId>,
    ) -> Result<OrgBankAccountOpenedEvent<T>> {
        let signer = self.chain_signer()?;
        self.chain_client()
            .open_org_bank_account_and_watch(
                &signer,
                seed,
                hosting_org,
                bank_operator,
            )
            .await?
            .org_bank_account_opened()?
            .ok_or_else(|| Error::EventNotFound.into())
    }
}
