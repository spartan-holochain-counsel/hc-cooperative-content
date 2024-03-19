pub use hdk_extensions::hdk;
pub use hdk_extensions::hdi_extensions;

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
    Ok( hdi_extensions::trace_origin(&action_address)? )
}


#[hdk_extern]
pub fn follow_evolutions(action_address: ActionHash) -> ExternResult<Vec<ActionHash>> {
    Ok( hdk_extensions::follow_evolutions(&action_address)? )
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
               self.authorities.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
    }
}

#[hdk_extern]
pub fn follow_evolutions_using_authorities(input: TraceAuthoritiesInput) -> ExternResult<Vec<ActionHash>> {
    debug!("Get latest entry: {}", input );
    Ok(
        hdk_extensions::follow_evolutions_using_authorities(
            &input.content_id,
            &input.authorities,
        )?
    )
}


#[derive(Clone, Deserialize, Debug)]
pub struct TraceAuthoritiesExceptionsInput {
    content_id: ActionHash,
    authorities: Vec<AgentPubKey>,
    exceptions: Vec<ActionHash>,
}
impl fmt::Display for TraceAuthoritiesExceptionsInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Trace {} via Authorities({:#?}) with exceptions {:#?}",
               self.content_id,
               self.authorities.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
               self.exceptions.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        )
    }
}

#[hdk_extern]
pub fn follow_evolutions_using_authorities_with_exceptions(input: TraceAuthoritiesExceptionsInput) -> ExternResult<Vec<ActionHash>> {
    debug!("Get latest entry: {}", input );
    Ok(
        hdk_extensions::follow_evolutions_using_authorities_with_exceptions(
            &input.content_id,
            &input.authorities,
            &input.exceptions,
        )?
    )
}


#[hdk_extern]
pub fn list_all_links_on_base(base: AnyLinkableHash) -> ExternResult<Vec<Link>> {
    Ok(
        get_links(
            GetLinksInputBuilder::try_new(
                base.clone(),
                ..,
            )?.build()
        )?
    )
}
