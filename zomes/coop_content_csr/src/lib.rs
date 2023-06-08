use hdk::prelude::*;
use hdk_extensions::{
    must_get,
    trace_evolutions,

    // HDI Extensions
    ScopedTypeConnector,
};
use coop_content::{
    GroupEntry,
};



#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<ActionHash> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_entry( group.to_input() )?;

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
