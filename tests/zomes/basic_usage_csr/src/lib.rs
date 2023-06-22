use hdk::prelude::*;
use hdk_extensions::{
    must_get,

    // HDI Extensions
    ScopedTypeConnector,
};
use basic_usage::{
    ContentEntry,
};
use coop_content_types::{
    GetGroupContentInput,
    GetAllGroupContentInput,
    // Macros
    get_group_content_latest,
    get_all_group_content_latest,
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

    attach_content_to_group!({
	entry: content,
	target: action_hash.clone(),
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
pub fn get_group_content(input: GetAllGroupContentInput) -> ExternResult<Vec<(ActionHash, ContentEntry)>> {
    debug!("Get all latest content entry: {:#?}", input );
    let contents = get_all_group_content_latest!({
	group_id: input.group_id,
    })?.into_iter()
	.filter_map(|(_, latest)| {
	    let latest_addr = latest.into_action_hash()?;
	    let record = must_get( &latest_addr ).ok()?;
	    Some((
		latest_addr,
		ContentEntry::try_from_record( &record ).ok()?
	    ))
	})
	.collect();

    Ok( contents )
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

    attach_content_update_to_group!({
	entry: input.entry,
	target: action_hash.clone(),
    })?;

    Ok( action_hash )
}
