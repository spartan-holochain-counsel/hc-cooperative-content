use hdk::prelude::*;
// use hdk::hash_path::path::{ Component };
pub use hdi_extensions::*;
use thiserror::Error;


//
// Custom Errors
//
#[derive(Debug, Error)]
pub enum HdkExtError<'a> {
    #[error("Record not found @ address {0}")]
    RecordNotFound(&'a ActionHash),
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
// Unwrapped get methods
//
pub fn must_get(action: &ActionHash) -> ExternResult<Record> {
    Ok(
	get( action.to_owned(), GetOptions::latest() )?
	    .ok_or(HdkExtError::RecordNotFound(action))?
    )
}


pub fn must_get_details(action: &ActionHash) -> ExternResult<RecordDetails> {
    let details = get_details( action.to_owned(), GetOptions::latest() )?
	.ok_or(HdkExtError::RecordNotFound(action))?;

    match details {
	Details::Record(record_details) => Ok( record_details ),
	Details::Entry(_) => Err(HdkExtError::ExpectedRecordNotEntry(action))?,
    }
}


//
// Tracing Actions
//
pub fn trace_evolutions (action_address: &ActionHash) -> ExternResult<Vec<ActionHash>> {
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


pub fn trace_evolutions_using_authorities (action_address: &ActionHash, authors: &Vec<AgentPubKey>) -> ExternResult<Vec<ActionHash>> {
    let mut evolutions = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
	let details = must_get_details( &addr )?;
	let valid_updates: Vec<SignedHashed<Action>> = details.updates.iter()
	    .filter_map(|sa| {
		debug!("Checking authorities for author '{}': {:?}", sa.action().author(), authors );
		if authors.contains(sa.action().author()) {
		    Some(sa.to_owned())
		} else {
		    None
		}
	    })
	    .collect();
	debug!("Filtered {}/{} updates", details.updates.len() - valid_updates.len(), details.updates.len() );
	let maybe_next_update = valid_updates.iter()
	    .min_by_key(|sa| sa.action().timestamp() );

	next_addr = match maybe_next_update {
	    Some(signed_action) => Some(signed_action.hashed.hash.to_owned()),
	    None => None,
	};

	evolutions.push( addr );
    }

    Ok( evolutions )
}
