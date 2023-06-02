use lazy_static::lazy_static;
use hdk::prelude::*;
use coop_content_types::{
    AppError,
    GroupEntry,
};
use coop_content::{
    ScopedTypeConnector,
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
    let action_hash = create_entry( group.to_input() )?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_group(group_id: ActionHash) -> ExternResult<GroupEntry> {
    debug!("Get latest group entry: {}", group_id );
    let record = get( group_id.clone(), GetOptions::latest() )?
	.ok_or(AppError::RecordNotFound(&group_id))?;

    // We always expect the Record to be a
    //
    // - Create action
    // - With an EntryType::App
    // - That has an AppEntryDef matching the ScopedEntryDefIndex

    Ok( GroupEntry::try_from_record( &record )? )
}
