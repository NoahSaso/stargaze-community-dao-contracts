use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    Addr,
};
use dao_hooks::nft_stake::{stake_nft_hook_msgs, unstake_nft_hook_msgs};

use crate::{
    contract::execute,
    state::{DAO, HOOKS},
};

#[test]
fn test_hooks() {
    let mut deps = mock_dependencies();

    let messages = stake_nft_hook_msgs(
        HOOKS,
        &deps.storage,
        Addr::unchecked("ekez"),
        "ekez-token".to_string(),
    )
    .unwrap();
    assert_eq!(messages.len(), 0);

    let messages = unstake_nft_hook_msgs(
        HOOKS,
        &deps.storage,
        Addr::unchecked("ekez"),
        vec!["ekez-token".to_string()],
    )
    .unwrap();
    assert_eq!(messages.len(), 0);

    // Save a DAO address for the execute messages we're testing.
    DAO.save(deps.as_mut().storage, &Addr::unchecked("ekez"))
        .unwrap();

    let env = mock_env();
    let info = mock_info("ekez", &[]);

    execute(
        deps.as_mut(),
        env,
        info,
        crate::msg::ExecuteMsg::AddHook {
            addr: "ekez".to_string(),
        },
    )
    .unwrap();

    let messages = stake_nft_hook_msgs(
        HOOKS,
        &deps.storage,
        Addr::unchecked("ekez"),
        "ekez-token".to_string(),
    )
    .unwrap();
    assert_eq!(messages.len(), 1);

    let messages = unstake_nft_hook_msgs(
        HOOKS,
        &deps.storage,
        Addr::unchecked("ekez"),
        vec!["ekez-token".to_string()],
    )
    .unwrap();
    assert_eq!(messages.len(), 1);

    let env = mock_env();
    let info = mock_info("ekez", &[]);

    execute(
        deps.as_mut(),
        env,
        info,
        crate::msg::ExecuteMsg::RemoveHook {
            addr: "ekez".to_string(),
        },
    )
    .unwrap();

    let messages = stake_nft_hook_msgs(
        HOOKS,
        &deps.storage,
        Addr::unchecked("ekez"),
        "ekez-token".to_string(),
    )
    .unwrap();
    assert_eq!(messages.len(), 0);

    let messages = unstake_nft_hook_msgs(
        HOOKS,
        &deps.storage,
        Addr::unchecked("ekez"),
        vec!["ekez-token".to_string()],
    )
    .unwrap();
    assert_eq!(messages.len(), 0);
}
