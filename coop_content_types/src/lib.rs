use std::collections::BTreeMap;
use hdi::prelude::*;
use hdk::prelude::Path;

//
// Custom Errors
//

#[derive(Debug)]
pub enum AppError<'a> {
    RecordNotFound(&'a ActionHash),
    RecordHasNoEntry(&'a ActionHash),
}

impl<'a> From<AppError<'a>> for WasmError {
    fn from(error: AppError) -> Self {
        wasm_error!(WasmErrorInner::Guest( format!("{:?}", error ) ))
    }
}



// Trait for common fields
pub trait CommonFields<'a> {
    fn published_at(&'a self) -> &'a u64;
    fn last_updated(&'a self) -> &'a u64;
    fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value>;
}

#[macro_export]
macro_rules! common_fields {
    ( $name:ident ) => {
	impl<'a> CommonFields<'a> for $name {
	    fn published_at(&'a self) -> &'a u64 {
		&self.published_at
	    }
	    fn last_updated(&'a self) -> &'a u64 {
		&self.last_updated
	    }
	    fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value> {
		&self.metadata
	    }
	}
    };
}



//
// Path Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct PathEntry( Path );



//
// Group Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupEntry {
    pub admins: Vec<AgentPubKey>,
    pub members: Vec<AgentPubKey>,

    // common fields
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, rmpv::Value>,
}
common_fields!( GroupEntry );



//
// Group Member Anchor Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupMemberAnchorEntry( ActionHash, AgentPubKey );



//
// Group Member Archive Anchor Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupMemberArchiveAnchorEntry( ActionHash, AgentPubKey, String );



//
// Content Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct ContentEntry {
    pub group_ref: ( ActionHash, ActionHash ),
}
