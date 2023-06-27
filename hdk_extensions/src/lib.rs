use hdk::prelude::{
    get, get_details, agent_info,
    debug, wasm_error,
    ExternResult, WasmError, WasmErrorInner, GetOptions,
    AgentPubKey, ActionHash, AnyDhtHash, AnyLinkableHash,
    Record, Action, Details, RecordDetails, SignedHashed,
};
use hdk::prelude::holo_hash::{
    AnyDhtHashPrimitive,
    AnyLinkableHashPrimitive,
};
use thiserror::Error;

pub use hdi_extensions;
pub use hdi_extensions::*;



//
// Custom Errors
//
#[derive(Debug, Error)]
pub enum HdkExtError<'a> {
    #[error("Record not found @ address {0}")]
    RecordNotFound(&'a AnyDhtHash),
    #[error("No entry in record ({0})")]
    RecordHasNoEntry(&'a ActionHash),
    #[error("Expected an action hash, not an entry hash: {0}")]
    ExpectedRecordNotEntry(&'a ActionHash),
}

impl<'a> From<HdkExtError<'a>> for WasmError {
    fn from(error: HdkExtError) -> Self {
        wasm_error!(WasmErrorInner::Guest( format!("{}", error ) ))
    }
}



//
// Agent
//
pub fn agent_id() -> ExternResult<AgentPubKey> {
    Ok( agent_info()?.agent_initial_pubkey )
}



//
// Get Helpers
//
pub fn must_get<T>(addr: &T) -> ExternResult<Record>
where
    T: Into<AnyDhtHash> + Clone + std::fmt::Debug,
{
    let addr : AnyDhtHash = addr.to_owned().into();
    Ok(
        match addr.clone().into_primitive() {
            AnyDhtHashPrimitive::Entry(entry_hash) => {
                get( entry_hash.clone(), GetOptions::latest() )?
                    .ok_or(HdkExtError::RecordNotFound(&addr))?
            },
            AnyDhtHashPrimitive::Action(action_hash) => {
                get( action_hash.clone(), GetOptions::latest() )?
                    .ok_or(HdkExtError::RecordNotFound(&addr))?
            },
        }
    )
}


pub fn must_get_details(action: &ActionHash) -> ExternResult<RecordDetails> {
    let details = get_details( action.to_owned(), GetOptions::latest() )?
        .ok_or(HdkExtError::RecordNotFound(&action.to_owned().into()))?;

    match details {
        Details::Record(record_details) => Ok( record_details ),
        Details::Entry(_) => Err(HdkExtError::ExpectedRecordNotEntry(action))?,
    }
}


pub fn exists<T>(addr: &T) -> ExternResult<bool>
where
    T: Clone + std::fmt::Debug,
    AnyDhtHash: From<T>,
{
    debug!("Checking if entry {:?} exists", addr );
    Ok( get( addr.to_owned(), GetOptions::content() )?.is_some() )
}



//
// Tracing Actions
//
pub fn resolve_action_addr<T>(addr: &T) -> ExternResult<ActionHash>
where
    T: Into<AnyLinkableHash> + Clone,
{
    let addr : AnyLinkableHash = addr.to_owned().into();
    match addr.into_primitive() {
        AnyLinkableHashPrimitive::Entry(entry_hash) => {
            Ok(
                get( entry_hash.to_owned(), GetOptions::latest() )?
                    .ok_or(HdkExtError::RecordNotFound(&entry_hash.into()))?
                    .action_address().to_owned()
            )
        },
        AnyLinkableHashPrimitive::Action(action_hash) => Ok( action_hash ),
        AnyLinkableHashPrimitive::External(external_hash) => Err(guest_error!(
            format!("External hash ({}) will not have a corresponding action", external_hash )
        )),
    }
}


pub fn trace_evolutions(action_address: &ActionHash) -> ExternResult<Vec<ActionHash>> {
    let mut evolutions = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
        let details = must_get_details( &addr )?;
        let maybe_next_update = details.updates.iter()
            .min_by_key(|sa| sa.action().timestamp() );

        next_addr = match maybe_next_update {
            Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
            None => None,
        };

        evolutions.push( addr );
    }

    Ok( evolutions )
}


pub fn latest_evolution(action_address: &ActionHash) -> ExternResult<ActionHash> {
    let evolutions = trace_evolutions( action_address )?;

    Ok( evolutions.last().unwrap().to_owned() )
}


pub fn trace_evolutions_selector<F>(
    action_address: &ActionHash,
    selector: F
) -> ExternResult<Vec<ActionHash>>
where
    F: Fn(Vec<SignedHashed<Action>>) -> ExternResult<Option<ActionHash>>,
{
    let mut evolutions = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
        let details = must_get_details( &addr )?;
        next_addr = selector( details.updates )?;

        evolutions.push( addr );
    }

    Ok( evolutions )
}


pub fn trace_evolutions_using_authorities(
    action_address: &ActionHash,
    authors: &Vec<AgentPubKey>
) -> ExternResult<Vec<ActionHash>> {
    let evolutions = trace_evolutions_selector( action_address, |updates| {
        let updates_count = updates.len();
        let valid_updates : Vec<SignedHashed<Action>> = updates
            .into_iter()
            .filter(|sa| {
                debug!(
                    "Checking authorities for author '{}': {:?}",
                    sa.action().author(),
                    authors
                );
                authors.contains( sa.action().author() )
            })
            .collect();

        debug!(
            "Filtered {}/{} updates",
            updates_count - valid_updates.len(),
            updates_count
        );
        let maybe_next_update = valid_updates.iter()
            .min_by_key(|sa| sa.action().timestamp() );

        Ok(
            match maybe_next_update {
                Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
                None => None,
            }
        )
    })?;

    Ok( evolutions )
}


pub fn trace_evolutions_using_authorities_with_exceptions(
    action_address: &ActionHash,
    authors: &Vec<AgentPubKey>,
    exceptions: &Vec<ActionHash>
) -> ExternResult<Vec<ActionHash>> {
    let evolutions = trace_evolutions_selector( action_address, |updates| {
        let updates_count = updates.len();
        let valid_updates : Vec<SignedHashed<Action>> = updates
            .into_iter()
            .filter(|sa| {
                debug!(
                    "Checking authorities for author '{}' or an action exception '{}'",
                    sa.action().author(),
                    sa.action_address()
                );
                authors.contains( sa.action().author() ) || exceptions.contains( sa.action_address() )
            })
            .collect();

        debug!(
            "Filtered {}/{} updates",
            updates_count - valid_updates.len(),
            updates_count
        );
        let maybe_next_update = valid_updates.iter()
            .min_by_key(|sa| sa.action().timestamp() );

        Ok(
            match maybe_next_update {
                Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
                None => None,
            }
        )
    })?;

    Ok( evolutions )
}
