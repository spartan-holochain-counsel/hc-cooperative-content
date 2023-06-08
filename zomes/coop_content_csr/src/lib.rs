use hdk::prelude::*;
use hdk_extensions::{
    must_get,
    trace_evolutions,
    agent_id,

    // HDI Extensions
    ScopedTypeConnector,
};
use coop_content::{
    GroupEntry,
    GroupAuthAnchorEntry,
    LinkTypes,
};


#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<ActionHash> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_entry( group.to_input() )?;
    let agent_id = agent_id()?;

    for pubkey in group.authorities() {
	let anchor = GroupAuthAnchorEntry( action_hash.to_owned(), pubkey );
	create_entry( anchor.to_input() )?;
	let anchor_hash = hash_entry( anchor )?;
	create_link( action_hash.to_owned(), anchor_hash, LinkTypes::GroupAuth, () )?;
    }

    create_link( agent_id, action_hash.to_owned(), LinkTypes::Group, () )?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_group(group_id: ActionHash) -> ExternResult<GroupEntry> {
    debug!("Get latest group entry: {}", group_id );
    let evolutions = trace_evolutions( &group_id )?;
    let record = must_get( &evolutions.last().unwrap() )?;

    Ok( GroupEntry::try_from_record( &record )? )
}


#[derive(Clone, Deserialize, Debug)]
pub struct UpdateInput {
    base: ActionHash,
    entry: GroupEntry,
}

#[hdk_extern]
pub fn update_group(input: UpdateInput) -> ExternResult<ActionHash> {
    debug!("Update group action: {}", input.base );
    let action_hash = update_entry( input.base, input.entry.to_input() )?;

    Ok( action_hash )
}
