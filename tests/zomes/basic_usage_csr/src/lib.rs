use lazy_static::lazy_static;
use hdk::prelude::*;
use hdk_extensions::{
    must_get,
    trace_evolutions,
    // trace_evolutions_using_authorities,

    // HDI Extensions
    ScopedTypeConnector,
};
use basic_usage::{
    ContentEntry,
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
pub fn create_content(content: ContentEntry) -> ExternResult<ActionHash> {
    debug!("Creating new content entry: {:#?}", content );
    let action_hash = create_entry( content.to_input() )?;

    Ok( action_hash )
}


#[hdk_extern]
pub fn get_content(content_id: ActionHash) -> ExternResult<ContentEntry> {
    debug!("Get latest content entry: {}", content_id );
    let evolutions = trace_evolutions( &content_id )?;
    let record = must_get( &evolutions.last().unwrap() )?;

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
    let action_hash = update_entry( input.base, input.entry.to_input() )?;

    Ok( action_hash )
}
