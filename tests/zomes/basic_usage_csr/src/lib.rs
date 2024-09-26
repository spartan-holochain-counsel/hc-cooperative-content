pub use basic_usage::test_types;
pub use basic_usage::coop_content_sdk;

pub use coop_content_sdk::hdk;
pub use coop_content_sdk::hdi_extensions;
pub use coop_content_sdk::hdk_extensions;

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
use test_types::{
    ContentEntry,
    CommentEntry,
};
use coop_content_sdk::{
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


#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum ContentTypes {
    Content(ContentEntry),
    Comment(CommentEntry),
    Unknown(Record),
}

impl TryInto<ContentTypes> for Record {
    type Error = WasmError;

    fn try_into(self) -> ExternResult<ContentTypes> {
        ContentEntry::try_from_record(&self).map(ContentTypes::Content)
            .or_else(|_| CommentEntry::try_from_record(&self).map(ContentTypes::Comment) )
            .or_else(|_| Ok(ContentTypes::Unknown(self)) )
    }
}

#[hdk_extern]
pub fn get_group_content(input: GetAllGroupContentInput) -> ExternResult<Vec<Entity<ContentTypes>>> {
    debug!("Get all latest content entry: {:#?}", input );
    let contents = get_all_group_content_latest!({
        group_id: input.group_id,
        content_type: input.content_type,
        content_base: input.content_base,
    })?.into_iter()
        .filter_map(|(origin, latest)| {
            let origin_addr = origin.into_action_hash()?;
            let latest_addr = latest.into_action_hash()?;
            let record = must_get( &latest_addr ).ok()?;

            Some(Entity(
                MorphAddr(origin_addr, latest_addr),
                record.try_into().ok()?
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
        content_type: String::from("content"),
        content_base: None,
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


#[hdk_extern]
pub fn create_comment(comment: CommentEntry) -> ExternResult<ActionHash> {
    debug!("Creating new comment entry: {:#?}", comment );
    let action_hash = create_entry( comment.to_input() )?;
    let content_base = comment.parent_comment.clone()
        .map( |base| format!("{}", base ) );

    register_content_to_group!({
        entry: comment,
        target: action_hash.clone(),
        content_type: String::from("comment"),
        content_base,
    })?;

    Ok( action_hash )
}
