use hdk::prelude::*;
use coop_content_types::{
    GroupEntry,
};
use coop_content::{
    EntryTypes,
};



#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("'basic_usage_csr' init");
    Ok(InitCallbackResult::Pass)
}


#[hdk_extern]
fn whoami(_: ()) -> ExternResult<AgentInfo> {
    Ok( agent_info()? )
}


#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<ActionHash> {
    debug!("Creating new group entry: {:?}", group );
    let action_hash = create_entry( EntryTypes::Group(group) )?;

    Ok( action_hash )
}
