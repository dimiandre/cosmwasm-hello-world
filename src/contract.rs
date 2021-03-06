#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{GreetResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:hello-world";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        greeting: msg.greeting,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("greeting", state.greeting))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetGreeting { greeting } => try_set_greeting(deps, info, greeting),
    }
}

pub fn try_set_greeting(deps: DepsMut, info: MessageInfo, greeting: String) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.greeting = greeting;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Greet {} => to_binary(&query_greeting(deps)?),
    }
}

fn query_greeting(deps: Deps) -> StdResult<GreetResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GreetResponse { greeting: state.greeting })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg { greeting: String::from("Ciao Mondo!") };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Greet {}).unwrap();
        let value: GreetResponse = from_binary(&res).unwrap();
        assert_eq!("Ciao Mondo!", value.greeting);
    }

    #[test]
    fn change_greet() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { greeting: String::from("Ciao Mondo!") };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SetGreeting { greeting: String::from("Hello World!")};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Greet {}).unwrap();
        let value: GreetResponse = from_binary(&res).unwrap();
        assert_eq!("Hello World!", value.greeting);
    }

    #[test]
    fn not_change_greet() {
        let mut deps = mock_dependencies(&coins(2, "token"));

        let msg = InstantiateMsg { greeting: String::from("Ciao Mondo!") };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Only owner can change greet
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SetGreeting { greeting: String::from("Hello World!") };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // greeting didn't change
        let res = query(deps.as_ref(), mock_env(), QueryMsg::Greet {}).unwrap();
        let value: GreetResponse = from_binary(&res).unwrap();
        assert_eq!("Ciao Mondo!", value.greeting);
    }
}
