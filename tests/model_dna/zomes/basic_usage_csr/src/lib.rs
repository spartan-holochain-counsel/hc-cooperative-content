use std::fmt;
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


//
// Exposed for debugging purposes
//
#[hdk_extern]
pub fn trace_origin(action_address: ActionHash) -> ExternResult<Vec<(ActionHash, Action)>> {
    Ok( hdk_extensions::trace_origin(&action_address)? )
}


#[derive(Clone, Deserialize, Debug)]
pub struct TraceAuthoritiesInput {
    content_id: ActionHash,
    authorities: Vec<AgentPubKey>,
}
impl fmt::Display for TraceAuthoritiesInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	write!(f, "Trace {} via Authorities({:#?})",
	       self.content_id,
	       self.authorities.iter().map(|s| s.to_string()).collect::<Vec<_>>()
	)
    }
}

#[hdk_extern]
pub fn trace_evolutions_using_authorities(input: TraceAuthoritiesInput) -> ExternResult<Vec<ActionHash>> {
    debug!("Get latest entry: {}", input );
    Ok( hdk_extensions::trace_evolutions_using_authorities( &input.content_id, &input.authorities )? )
}
