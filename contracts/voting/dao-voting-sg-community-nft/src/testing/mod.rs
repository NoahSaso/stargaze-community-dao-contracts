mod adversarial;
mod execute;
mod hooks;
mod instantiate;
mod queries;
mod tests;

use cosmwasm_std::{Addr, Empty};
use cw_multi_test::{App, Contract, ContractWrapper, Executor};

use crate::msg::InstantiateMsg;

use self::instantiate::instantiate_cw721_base;

/// Address used as the owner, instantiator, and minter.
pub(crate) const CREATOR_ADDR: &str = "creator";

pub(crate) const OWNER: &str = "owner";

pub(crate) struct CommonTest {
    app: App,
    module: Addr,
    nft: Addr,
}

pub(crate) fn voting_sg_community_nft_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        super::contract::execute,
        super::contract::instantiate,
        super::contract::query,
    );
    Box::new(contract)
}

pub(crate) fn setup_test() -> CommonTest {
    let mut app = App::default();
    let module_id = app.store_code(voting_sg_community_nft_contract());

    let nft = instantiate_cw721_base(&mut app, CREATOR_ADDR, CREATOR_ADDR);
    let module = app
        .instantiate_contract(
            module_id,
            Addr::unchecked(CREATOR_ADDR),
            &InstantiateMsg {
                nft_contract: nft.to_string(),
                owner: Some(OWNER.to_string()),
            },
            &[],
            "sg-community-nft_voting",
            None,
        )
        .unwrap();
    CommonTest { app, module, nft }
}

// Advantage to using a macro for this is that the error trace links
// to the exact line that the error occured, instead of inside of a
// function where the assertion would otherwise happen.
macro_rules! is_error {
    ($x:expr => $e:tt) => {
        assert!(format!("{:#}", $x.unwrap_err()).contains($e))
    };
}

pub(crate) use is_error;
