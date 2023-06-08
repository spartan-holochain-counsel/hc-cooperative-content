use hdk::prelude::*;
// use hdk_extensions::{
//     agent_id,
// };
use coop_content_types::{
    GroupAuthAnchorEntry,
};
// use basic_usage::{
//     LinkTypes,
// };
use coop_content::{
    LinkTypes,
};


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
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
    let anchor_hash = hash_entry( anchor )?;

    create_link( anchor_hash, input.target, LinkTypes::Content, () )?;

    Ok(())
}
