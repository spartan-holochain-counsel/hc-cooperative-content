pub use coop_content_types::hdk;
pub use coop_content_types::hdi_extensions;
pub use coop_content_types::hdk_extensions;

use hdk::prelude::*;
use hdi_extensions::{
    ScopedTypeConnector,
};
use hdk_extensions::{
    must_get,
    Entity, MorphAddr,
    // Inputs
    UpdateEntryInput,
};
use basic_usage::{
    ContentEntry,
};
use coop_content_types::{
    GroupEntry,
    GetGroupContentInput,
    GetAllGroupContentInput,
    // Macros
    create_group, get_group, update_group,
    get_group_content_latest,
    get_all_group_content_latest,
    register_content_to_group,
    register_content_update_to_group,
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


#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<ActionHash> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_group!( group )?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_group(id: ActionHash) -> ExternResult<GroupEntry> {
    debug!("Creating new group entry: {:#?}", id );
    let group = get_group!( id )?;

    Ok( group )
}


#[hdk_extern]
pub fn update_group(input: UpdateEntryInput<GroupEntry>) -> ExternResult<ActionHash> {
    debug!("Update group: {:#?}", input );
    let action_hash = update_group!({
        base: input.base,
        entry: input.entry,
    })?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_content(input: GetGroupContentInput) -> ExternResult<ContentEntry> {
    debug!("Get latest content entry: {:#?}", input );
    let latest_addr = get_group_content_latest!({
        group_id: input.group_id,
        content_id: input.content_id,
    })?;
    let record = must_get( &latest_addr )?;

    Ok( ContentEntry::try_from_record( &record )? )
}


#[hdk_extern]
pub fn get_group_content(input: GetAllGroupContentInput) -> ExternResult<Vec<Entity<ContentEntry>>> {
    debug!("Get all latest content entry: {:#?}", input );
    let contents = get_all_group_content_latest!({
        group_id: input.group_id,
    })?.into_iter()
        .filter_map(|(origin, latest)| {
            let origin_addr = origin.into_action_hash()?;
            let latest_addr = latest.into_action_hash()?;
            let record = must_get( &latest_addr ).ok()?;
            Some(Entity(
                MorphAddr(origin_addr, latest_addr),
                ContentEntry::try_from_record( &record ).ok()?
            ))
        })
        .collect();

    Ok( contents )
}


#[hdk_extern]
pub fn create_content(content: ContentEntry) -> ExternResult<ActionHash> {
    debug!("Creating new content entry: {:#?}", content );
    let action_hash = create_entry( content.to_input() )?;

    register_content_to_group!({
        entry: content,
        target: action_hash.clone(),
    })?;

    Ok( action_hash )
}


#[derive(Clone, Deserialize, Debug)]
pub struct UpdateInput {
    base: ActionHash,
    entry: ContentEntry,
}

#[hdk_extern]
pub fn update_content(input: UpdateInput) -> ExternResult<ActionHash> {
    debug!("Update content action: {}", input.base );
    // let prev_content : ContentEntry = must_get( &input.base )?.try_into();
    let action_hash = update_entry( input.base, input.entry.to_input() )?;

    register_content_update_to_group!({
        entry: input.entry,
        target: action_hash.clone(),
    })?;

    Ok( action_hash )
}
