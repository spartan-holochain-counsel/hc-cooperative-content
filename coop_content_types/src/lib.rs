use std::collections::BTreeMap;
use hdi::prelude::*;
use hdk::prelude::Path;
// use hdk::hash_path::path::{ Component };
use thiserror::Error;
use hdi_extensions::{
    get_root_origin,
};


//
// Custom Errors
//
#[derive(Debug, Error)]
pub enum AppError<'a> {
    #[error("Invalid group entry: {0}")]
    InvalidGroup(&'a str),
}

impl<'a> From<AppError<'a>> for WasmError {
    fn from(error: AppError) -> Self {
        wasm_error!(WasmErrorInner::Guest( format!("{}", error ) ))
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

impl GroupEntry {
    pub fn authorities(&self) -> Vec<AgentPubKey> {
	vec![ self.admins.clone(), self.members.clone() ]
	    .into_iter()
	    .flatten()
	    .collect()
    }

    pub fn authorities_diff(&self, other: &GroupEntry) -> AuthoritiesDiff {
	let added: Vec<AgentPubKey> = other.authorities()
	    .into_iter()
	    .filter(|pubkey| !self.authorities().contains(pubkey))
	    .collect();

	let removed: Vec<AgentPubKey> = self.authorities()
	    .into_iter()
	    .filter(|pubkey| !other.authorities().contains(pubkey))
	    .collect();

	let intersection: Vec<AgentPubKey> = self.authorities()
	    .into_iter()
	    .filter(|pubkey| other.authorities().contains(pubkey))
	    .collect();

	AuthoritiesDiff {
	    added,
	    removed,
	    intersection,
	}
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthoritiesDiff {
    pub added: Vec<AgentPubKey>,
    pub removed: Vec<AgentPubKey>,
    pub intersection: Vec<AgentPubKey>,
}



//
// Path Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct PathEntry( pub Path );



//
// Group Member Anchor Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupAuthAnchorEntry( pub ActionHash, pub AgentPubKey );



//
// Group Member Archive Anchor Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupAuthArchiveAnchorEntry( pub ActionHash, pub AgentPubKey, String );

impl GroupAuthArchiveAnchorEntry {
    pub fn new(group_id: ActionHash, agent: AgentPubKey) -> Self {
	GroupAuthArchiveAnchorEntry(group_id, agent, "archive".to_string())
    }
}


#[hdk_entry_helper]
#[serde(untagged)]
#[derive(Clone)]
pub enum GroupAuthAnchor {
    Active(GroupAuthAnchorEntry),
    Archive(GroupAuthArchiveAnchorEntry),
}

impl GroupAuthAnchor {
    pub fn is_archive(&self) -> bool {
	match &self {
	    GroupAuthAnchor::Active(_) => false,
	    GroupAuthAnchor::Archive(_) => true,
	}
    }

    pub fn author(&self) -> &AgentPubKey {
	match &self {
	    GroupAuthAnchor::Active(anchor) => &anchor.1,
	    GroupAuthAnchor::Archive(anchor) => &anchor.1,
	}
    }
}



//
// CSR Input Structs
//
#[derive(Clone, Deserialize, Debug)]
pub struct UpdateInput {
    pub base: ActionHash,
    pub entry: GroupEntry,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateContentLinkInput {
    pub group_id: ActionHash,
    pub author: AgentPubKey,
    pub target: AnyDhtHash,
}



//
// A trait for determining a group state
//
pub trait GroupRef {
    fn group_ref(&self) -> (ActionHash, ActionHash);
}

impl GroupRef for (ActionHash, ActionHash) {
    fn group_ref(&self) -> (ActionHash, ActionHash) {
	self.to_owned()
    }
}

#[macro_export]
macro_rules! group_ref {
    ( $type:ident, $($ref:tt).* ) => {
	impl coop_content_types::GroupRef for $type {
	    fn group_ref(&self) -> (ActionHash, ActionHash) {
		self$(.$ref)*.to_owned()
	    }
	}
    };
    ( $type:ident, $($id:tt).*, $($rev:tt).* ) => {
	impl coop_content_types::GroupRef for $type {
	    fn group_ref(&self) -> (ActionHash, ActionHash) {
		(
		    self$(.$id)*.to_owned(),
		    self$(.$rev)*.to_owned()
		)
	    }
	}
    };
}


pub fn validate_group_auth<T>(
    entry: &T,
    action: impl Into<EntryCreationAction>
) -> Result<(), String>
where
    T: GroupRef + TryFrom<Entry, Error = WasmError> + Clone,
{
    let creation_action : EntryCreationAction = action.into();

    validate_group_ref( entry, creation_action.clone() )?;
    validate_group_member( entry, creation_action )?;

    Ok(())
}


pub fn validate_group_ref<T>(
    entry: &T,
    action: impl Into<EntryCreationAction>
) -> Result<(), String>
where
    T: GroupRef + TryFrom<Entry, Error = WasmError> + Clone,
{
    let group_ref = entry.group_ref();

    if let EntryCreationAction::Update(update) = action.into() {
	let prev_entry : T = must_get_entry( update.original_entry_address.to_owned() )?
	    .content.try_into()?;
	let prev_group_ref = prev_entry.group_ref();

	if group_ref.0 != prev_group_ref.0 {
	    return Err("Content group ID cannot be changed".to_string())?;
	}
    }

    if group_ref.0 != get_root_origin( &group_ref.1 )?.0 {
	return Err("Content group ID is not the initial action for the group revision".to_string())?;
    }

    Ok(())
}


pub fn validate_group_member<T>(
    entry: &T,
    action: impl Into<EntryCreationAction>
) -> Result<(), String>
where
    T: GroupRef + TryFrom<Entry, Error = WasmError> + Clone,
{
    let creation_action : EntryCreationAction = action.into();
    let author = creation_action.author();

    let group_ref = entry.group_ref();
    let signed_action = must_get_action( group_ref.1.to_owned() )?;
    let group : GroupEntry = match signed_action.action().entry_hash() {
	Some(entry_addr) => must_get_entry( entry_addr.to_owned() )?
	    .content.try_into()?,
	None => return Err(format!("Action ({}) does not contain an entry hash", group_ref.1 )),
    };

    // debug!("Checking authorities {:#?} for author {}", group.authorities(), author );
    if !group.authorities().contains( author ) {
	return Err(format!("Agent ({}) is not authorized to update content managed by group {}", author, group_ref.0 ))?;
    }

    Ok(())
}
