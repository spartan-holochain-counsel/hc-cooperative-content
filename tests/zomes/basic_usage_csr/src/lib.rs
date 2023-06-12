use hdk::prelude::*;
use hdk_extensions::{
    must_get,
    trace_evolutions,

    // HDI Extensions
    ScopedTypeConnector,
};
use basic_usage::{
    ContentEntry,
};
use coop_content_types::{
    // Macros
    attach_content_to_group,
    attach_content_update_to_group,
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
pub fn create_content(content: ContentEntry) -> ExternResult<ActionHash> {
    debug!("Creating new content entry: {:#?}", content );
    let action_hash = create_entry( content.to_input() )?;

    attach_content_to_group!( content, action_hash );

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_content(content_id: ActionHash) -> ExternResult<ContentEntry> {
    debug!("Get latest content entry: {}", content_id );
    let evolutions = trace_evolutions( &content_id )?;
    let record = must_get( evolutions.last().unwrap() )?;

    Ok( ContentEntry::try_from_record( &record )? )
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

    attach_content_update_to_group!( input.entry, action_hash );

    Ok( action_hash )
}
