use lazy_static::lazy_static;
use hdk::prelude::*;
use coop_content_types::{
    GroupEntry,
};
use coop_content::{
    EntryTypes,
};

lazy_static! {
    static ref ZOME_NAME: String = match zome_info() {
	Ok(info) => format!("{}", info.name ),
	Err(_) => String::from("?"),
    };
}


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("'{}' init", *ZOME_NAME );
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
