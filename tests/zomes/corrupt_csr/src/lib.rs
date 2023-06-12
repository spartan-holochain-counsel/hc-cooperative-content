use hdk::prelude::*;
use hdk_extensions::{
    // HDI Extensions
    ScopedTypeConnector,
};
use coop_content::{
    LinkTypes,
    GroupAuthAnchorEntry,
};


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("'{}' init", zome_info()?.name );
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<AgentInfo> {
    Ok( agent_info()? )
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidAuthAnchorInput {
    group_id: ActionHash,
    anchor_agent: AgentPubKey,
    target: ActionHash,
}

#[hdk_extern]
pub fn invalid_auth_anchor_link(input: InvalidAuthAnchorInput) -> ExternResult<()> {
    debug!("InvalidAuthAnchorInput: {:#?}", input );
    let anchor = GroupAuthAnchorEntry( input.group_id.to_owned(), input.anchor_agent.to_owned() );
    let anchor_hash = hash_entry( &anchor )?;

    create_link( anchor_hash, input.target, LinkTypes::Content, () )?;

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidGroupAuthInput {
    group_id: ActionHash,
    group_rev: ActionHash,
    anchor_agent: AgentPubKey,
}

#[hdk_extern]
pub fn invalid_group_auth_link(input: InvalidGroupAuthInput) -> ExternResult<()> {
    debug!("InvalidGroupAuthInput: {:#?}", input );
    let anchor = GroupAuthAnchorEntry( input.group_id, input.anchor_agent );
    create_entry( anchor.to_input() )?;
    let anchor_hash = hash_entry( &anchor )?;

    create_link( input.group_rev, anchor_hash, LinkTypes::GroupAuth, () )?;

    Ok(())
}
