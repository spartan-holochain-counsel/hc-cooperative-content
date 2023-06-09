use std::fmt;
use hdk::prelude::*;


#[hdk_extern]
fn init(_: ()) -> ExternResult<InitCallbackResult> {
    debug!("'{}' init", zome_info()?.name );
    Ok(InitCallbackResult::Pass)
}


//
// Exposed for debugging purposes
//
#[hdk_extern]
fn whoami(_: ()) -> ExternResult<AgentInfo> {
    Ok( agent_info()? )
}


#[hdk_extern]
pub fn trace_origin(action_address: ActionHash) -> ExternResult<Vec<(ActionHash, Action)>> {
    Ok( hdk_extensions::trace_origin(&action_address)? )
}


#[hdk_extern]
pub fn trace_evolutions(action_address: ActionHash) -> ExternResult<Vec<ActionHash>> {
    Ok( hdk_extensions::trace_evolutions(&action_address)? )
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
