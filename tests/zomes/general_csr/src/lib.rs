use hdk::prelude::*;
use holo_hash::AnyDhtHashPrimitive;
use hdk_extensions::{
    guest_error,
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
fn fetch_action(addr: ActionHash) -> ExternResult<Action> {
    Ok( must_get_action(addr)?.hashed.content )
}


#[hdk_extern]
fn fetch_entry(addr: AnyDhtHash) -> ExternResult<Entry> {
    let entry_hash = match addr.into_primitive() {
        AnyDhtHashPrimitive::Entry(addr) => addr,
        AnyDhtHashPrimitive::Action(addr) => {
            fetch_action( addr.clone() )?.entry_hash()
                .ok_or(guest_error!(format!("Action ({}) does not have an entry", addr )))?
                .to_owned()
        },
    };

    Ok( must_get_entry(entry_hash)?.content )
}


#[hdk_extern]
pub fn trace_origin(action_address: ActionHash) -> ExternResult<Vec<(ActionHash, Action)>> {
    Ok( hdk_extensions::trace_origin( &action_address )? )
}


#[hdk_extern]
pub fn follow_evolutions(action_address: ActionHash) -> ExternResult<Vec<ActionHash>> {
    Ok( hdk_extensions::follow_evolutions( &action_address )? )
}
