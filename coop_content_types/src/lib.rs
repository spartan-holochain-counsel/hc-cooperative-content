use std::collections::BTreeMap;
use hdi::prelude::*;
use hdk::prelude::AnyLinkableHash;
use thiserror::Error;
pub use hdk_extensions::*;



//
// General Use
//
/// A map of evolution pointers
///
/// The key is the address that was evolved from and the value is the address of what it evolved to
pub type LinkPointerMap = Vec<(AnyLinkableHash, AnyLinkableHash)>;



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
/// Common fields that are expected on some entry structs
pub trait CommonFields<'a> {
    /// A timestamp that indicates when the original create entry was made
    fn published_at(&'a self) -> &'a u64;
    /// A timestamp that indicates when this entry was created
    fn last_updated(&'a self) -> &'a u64;
    /// A spot for holding data that is not relevant to integrity validation
    fn metadata(&'a self) -> &'a BTreeMap<String, rmpv::Value>;
}

/// Auto-implement the [`CommonFields`] trait
///
/// The input must be a struct with fields matching each common field method.
///
/// #### Example
/// ```
/// struct PostEntry {
///     pub message: String,
///
///     // Common fields
///     pub published_at: u64,
///     pub last_updated: u64,
///     pub metadata: BTreeMap<String, rmpv::Value>,
/// }
/// common_fields!( PostEntry );
/// ```
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
/// An entry struct for defining a group and its group authorities
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupEntry {
    /// The list of agents with admin authorities in this group
    pub admins: Vec<AgentPubKey>,
    /// The list of agents with write authorities in this group
    pub members: Vec<AgentPubKey>,
    /// An indicator of whether this group is still active
    pub deleted: Option<bool>,

    // common fields
    pub published_at: u64,
    pub last_updated: u64,
    pub metadata: BTreeMap<String, rmpv::Value>,
}
common_fields!( GroupEntry );

impl GroupEntry {
    /// Get a list of the admins and members of this group
    pub fn authorities(&self) -> Vec<AgentPubKey> {
        vec![ self.admins.clone(), self.members.clone() ]
            .into_iter()
            .flatten()
            .collect()
    }

    /// Return the differences between this group and the given group
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

/// The result of a group comparison
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthoritiesDiff {
    pub added: Vec<AgentPubKey>,
    pub removed: Vec<AgentPubKey>,
    pub intersection: Vec<AgentPubKey>,
}



//
// Group Member Anchor Entry
//
/// An entry struct (anchor) representing a group authority's personal anchor
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupAuthAnchorEntry( pub ActionHash, pub AgentPubKey );



//
// Group Member Archive Anchor Entry
//
/// An entry struct (anchor) representing a former authority of a group
#[hdk_entry_helper]
#[derive(Clone)]
pub struct GroupAuthArchiveAnchorEntry( pub ActionHash, pub AgentPubKey, String );

impl GroupAuthArchiveAnchorEntry {
    pub fn new(group_id: ActionHash, agent: AgentPubKey) -> Self {
        GroupAuthArchiveAnchorEntry(group_id, agent, "archive".to_string())
    }
}


/// An enum that represents an authority anchor (active/archived)
#[hdk_entry_helper]
#[serde(untagged)]
#[derive(Clone)]
pub enum GroupAuthAnchor {
    Active(GroupAuthAnchorEntry),
    Archive(GroupAuthArchiveAnchorEntry),
}

impl GroupAuthAnchor {
    /// Get the agent pubkey of this auth anchor
    pub fn author(&self) -> &AgentPubKey {
        match &self {
            GroupAuthAnchor::Active(anchor) => &anchor.1,
            GroupAuthAnchor::Archive(anchor) => &anchor.1,
        }
    }

    /// Get the group revision (action hash) of this auth anchor
    pub fn group(&self) -> &ActionHash {
        match &self {
            GroupAuthAnchor::Active(anchor) => &anchor.0,
            GroupAuthAnchor::Archive(anchor) => &anchor.0,
        }
    }

    /// Determine if this enum's item is [`GroupAuthAnchor::Archive`]
    pub fn is_archive(&self) -> bool {
        match &self {
            GroupAuthAnchor::Active(_) => false,
            GroupAuthAnchor::Archive(_) => true,
        }
    }
}


/// Indicates the intended group auth anchor type
///
/// Since the variable content is the same for both anchor types, this enum helps declare the
/// intended type when passing around the group/author anchor values.
#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum GroupAuthAnchorType {
    Active,
    Archive,
}

impl<'de> serde::Deserialize<'de> for GroupAuthAnchorType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let input : Option<String> = Deserialize::deserialize(deserializer)?;

        Ok(
            match input {
                Some(name) => match name.to_lowercase().as_str() {
                    "active" => GroupAuthAnchorType::Active,
                    "archive" | "inactive" => GroupAuthAnchorType::Archive,
                    lw_name => Err(serde::de::Error::custom(
                        format!("No match for '{}' in GroupAuthAnchorType enum", lw_name )
                    ))?,
                },
                None => GroupAuthAnchorType::Active,
            }
        )
    }
}



//
// CSR Input Structs
//
/// Input required for registering new content to a group
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateContentLinkInput {
    pub group_id: ActionHash,
    pub content_target: AnyLinkableHash,
}

/// Input required for registering a content update to a group
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateContentUpdateLinkInput {
    pub group_id: ActionHash,
    pub content_id: AnyLinkableHash,
    pub content_prev: AnyLinkableHash,
    pub content_next: AnyLinkableHash,
}

/// Input required for initializing a group auth anchor entry
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupAuthInput {
    pub group_id: ActionHash,
    pub author: AgentPubKey,
    pub anchor_type: GroupAuthAnchorType,
}

/// Input for following all content evolutions in a group
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetAllGroupContentInput {
    pub group_id: ActionHash,
    pub full_trace: Option<bool>,
}

/// Input for following a single content's evolution in a group
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetGroupContentInput {
    pub group_id: ActionHash,
    pub content_id: AnyLinkableHash,
    pub full_trace: Option<bool>,
}



//
// A trait for determining a group state
//
/// A trait for determining an entry's group reference
pub trait GroupRef {
    fn group_ref(&self) -> (ActionHash, ActionHash);
}

impl GroupRef for (ActionHash, ActionHash) {
    fn group_ref(&self) -> (ActionHash, ActionHash) {
        self.to_owned()
    }
}

/// Easily-implement the [`GroupRef`] trait
///
/// When using a single field, the 2 [`ActionHash`] tuple order must be `(ID, revision)`
///
/// #### Examples
///
/// ##### Example: Single Field
/// ```
/// struct PostEntry {
///     pub message: String,
///     pub group_ref: (ActionHash, ActionHash),
/// }
/// common_fields!( PostEntry, group_ref );
/// ```
///
/// ##### Example: Separate Fields
/// ```
/// struct PostEntry {
///     pub message: String,
///
///     pub group_id: ActionHash,
///     pub group_rev: ActionHash,
/// }
/// common_fields!( PostEntry, group_id, group_rev );
/// ```
///
/// ##### Example: Separate Struct
/// ```
/// struct GroupRef {
///     pub id: ActionHash,
///     pub rev: ActionHash,
/// }
///
/// struct PostEntry {
///     pub message: String,
///     pub group_ref: GroupRef,
/// }
/// common_fields!( PostEntry, group_ref.id, group_ref.rev );
/// ```
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


//
// Validation helpers
//
/// Checks that an entry's group reference and author are valid
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


/// Check that an entry's group reference is valid
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

    if group_ref.0 != trace_origin_root( &group_ref.1 )?.0 {
        return Err("Content group ID is not the initial action for the group revision".to_string())?;
    }

    Ok(())
}


/// Checks that the author of an action is an authority in the entry's group reference
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


//
// Zome call helpers
//
/// Call a local zome function
///
/// ##### Example: Basic Usage
/// ```
/// let group_id = "uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7";
/// let content_addr = "uhCkknDrZjzEgzf8iIQ6aEzbqEYrYBBg1pv_iTNUGAFJovhxOJqu0";
///
/// call_local_zome!(
///     "coop_content_csr",
///     "create_content_link",
///     coop_content_types::CreateContentLinkInput {
///         group_id: ActionHash::try_from(group_id).unwrap(),
///         content_target: ActionHash::try_from(content_addr).unwrap().into(),
///     }
/// )?
/// ```
#[macro_export]
macro_rules! call_local_zome {
    ( $zome:literal, $fn:literal, $($input:tt)+ ) => {
        match hdk::prelude::call(
            hdk::prelude::CallTargetCell::Local,
            $zome,
            $fn.into(),
            None,
            $($input)+,
        )? {
            ZomeCallResponse::Ok(extern_io) => Ok(extern_io),
            ZomeCallResponse::NetworkError(msg) => Err(hdk_extensions::guest_error!(format!("{}", msg))),
            ZomeCallResponse::CountersigningSession(msg) => Err(hdk_extensions::guest_error!(format!("{}", msg))),
            _ => Err(hdk_extensions::guest_error!(format!("Zome call response: Unauthorized"))),
        }
    };
}

/// Call a local zome function and decode the response
///
/// ##### Example: Basic Usage
/// ```
/// let group_id = "uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7";
/// let content_addr = "uhCkknDrZjzEgzf8iIQ6aEzbqEYrYBBg1pv_iTNUGAFJovhxOJqu0";
///
/// call_local_zome_decode!(
///     ActionHash,
///     "coop_content_csr",
///     "create_content_link",
///     coop_content_types::CreateContentLinkInput {
///         group_id: ActionHash::try_from(group_id).unwrap(),
///         content_target: ActionHash::try_from(content_addr).unwrap().into(),
///     }
/// )?
/// ```
#[macro_export]
macro_rules! call_local_zome_decode {
    ( $zome:literal, $fn:literal, $($input:tt)+ ) => {
        coop_content_types::call_local_zome!( $zome, $fn, $($input)+ )?
            .decode()
            .map_err(|err| hdk::prelude::wasm_error!(hdk::prelude::WasmErrorInner::from(err)) )
    };
    ( $into_type:ident, $zome:literal, $fn:literal, $($input:tt)+ ) => {
        coop_content_types::call_local_zome!( $zome, $fn, $($input)+ )?
            .decode::<$into_type>()
            .map_err(|err| hdk::prelude::wasm_error!(hdk::prelude::WasmErrorInner::from(err)) )
    };
}


/// Input required for macros `register_content_to_group` and `register_content_update_to_group`
#[derive(Clone)]
pub struct RegisterContentMacroInput<T>
where
    T: GroupRef + Clone,
{
    /// The content entry belonging to the target
    pub entry: T,
    /// An entry creation action address
    pub target: ActionHash,
}


/// Register a new content target to a group
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input template is [`RegisterContentMacroInput`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `create_content_link`
///
/// Returns [`ActionHash`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// #[hdk_entry_helper]
/// struct PostEntry {
///     pub message: String,
/// }
///
/// #[hdk_entry_defs]
/// #[unit_enum(EntryTypesUnit)]
/// pub enum EntryTypes {
///     #[entry_def]
///     Post(PostEntry),
/// }
///
/// let post = PostEntry {
///     message: "Hello world".to_string(),
/// };
/// let create_addr = create_entry( EntryTypes::Post(post) )?;
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let link_addr = register_content_to_group!({
///     entry: post,
///     target: create_addr,
/// })?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let link_addr = register_content_to_group!(
///     "coop_content_csr_renamed",
///     {
///         entry: post,
///         target: create_addr,
///     }
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let link_addr = register_content_to_group!(
///     "custom_coop_content_csr",
///     "register_content_link",
///     {
///         entry: post,
///         target: create_addr,
///     }
/// )?;
/// ```
#[macro_export]
macro_rules! register_content_to_group {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            use coop_content_types::GroupRef;
            let input = coop_content_types::RegisterContentMacroInput $($def)*;

            coop_content_types::call_local_zome_decode!(
                ActionHash,
                $zome,
                $fn_name,
                coop_content_types::CreateContentLinkInput {
                    group_id: input.entry.group_ref().0,
                    content_target: input.target.clone().into(),
                }
            )
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::register_content_to_group!( $zome, "create_content_link", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::register_content_to_group!( "coop_content_csr", $($def)* )
    };
}


/// Register a content update target to a group
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input template is [`RegisterContentMacroInput`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `create_content_update_link`
///
/// Returns [`ActionHash`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// #[hdk_entry_helper]
/// struct PostEntry {
///     pub message: String,
/// }
///
/// #[hdk_entry_defs]
/// #[unit_enum(EntryTypesUnit)]
/// pub enum EntryTypes {
///     #[entry_def]
///     Post(PostEntry),
/// }
///
/// let post = PostEntry {
///     message: "Hello world".to_string(),
/// };
/// let create_addr = create_entry( EntryTypes::Post(post) )?;
///
/// let post_updated = PostEntry {
///     message: "Hello world (updated)".to_string(),
/// };
/// let update_addr = update_entry( create_addr, EntryTypes::Post(post_updated) )?;
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let link_addr = register_content_update_to_group!({
///     entry: post_updated,
///     target: update_addr,
/// })?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let link_addr = register_content_update_to_group!(
///     "coop_content_csr_renamed",
///     {
///         entry: post_updated,
///         target: update_addr,
///     }
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let link_addr = register_content_update_to_group!(
///     "custom_coop_content_csr",
///     "register_content_update_link",
///     {
///         entry: post_updated,
///         target: update_addr,
///     }
/// )?;
/// ```
#[macro_export]
macro_rules! register_content_update_to_group {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            use coop_content_types::GroupRef;
            let input = coop_content_types::RegisterContentMacroInput $($def)*;
            let history = hdk_extensions::trace_origin( &input.target )?;

            if history.len() < 2 {
                Err(hdk_extensions::guest_error!(format!("History of target {} is empty", input.target )))?
            }

            let content_id = &history[ history.len() - 1 ].0;
            let content_prev_rev = &history[1].0;

            coop_content_types::call_local_zome_decode!(
                ActionHash,
                $zome,
                $fn_name,
                coop_content_types::CreateContentUpdateLinkInput {
                    group_id: input.entry.group_ref().0,
                    content_id: content_id.clone().into(),
                    content_prev: content_prev_rev.clone().into(),
                    content_next: input.target.clone().into(),
                }
            )
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::register_content_update_to_group!( $zome, "create_content_update_link", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::register_content_update_to_group!( "coop_content_csr", $($def)* )
    };
}


/// Input required for macro `get_group_content_latest`
#[derive(Clone)]
pub struct GetGroupContentMacroInput {
    pub group_id: ActionHash,
    pub content_id: AnyLinkableHash,
}


/// Get the latest evolution of a single content target in a group
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input template is [`GetGroupContentMacroInput`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `get_group_content_latest_shortcuts`
///
/// Returns [`ActionHash`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// let group_id = ActionHash::try_from("uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7").unwrap();
/// let content_id = ActionHash::try_from("uhCkknDrZjzEgzf8iIQ6aEzbqEYrYBBg1pv_iTNUGAFJovhxOJqu0").unwrap();
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let latest_addr = get_group_content_latest!({
///     group_id: group_id,
///     content_id: content_id.into(),
/// })?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let latest_addr = get_group_content_latest!(
///     "coop_content_csr_renamed",
///     {
///         group_id: group_id,
///         content_id: content_id.into(),
///     }
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let latest_addr = get_group_content_latest!(
///     "custom_coop_content_csr",
///     "get_single_group_content",
///     {
///         group_id: group_id,
///         content_id: content_id.into(),
///     }
/// )?;
/// ```
#[macro_export]
macro_rules! get_group_content_latest {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            let input = coop_content_types::GetGroupContentMacroInput $($def)*;
            let action_addr = hdk_extensions::resolve_action_addr( &input.content_id )?;
            let history = hdk_extensions::trace_origin( &action_addr )?;

            if history.len() < 1 {
                Err(hdk_extensions::guest_error!(format!("Unexpected state")))?
            }

            if input.content_id != history[ history.len() - 1 ].0.clone().into() {
                Err(hdk_extensions::guest_error!(format!("Given 'content_id' must be an ID (create action); not an update action")))?
            }

            coop_content_types::call_local_zome_decode!(
                ActionHash,
                $zome,
                $fn_name,
                coop_content_types::GetGroupContentInput {
                    group_id: input.group_id,
                    content_id: input.content_id,
                    full_trace: None,
                }
            )
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::get_group_content_latest!( $zome, "get_group_content_latest_shortcuts", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::get_group_content_latest!( "coop_content_csr", $($def)* )
    };
}


/// Input required for macro `get_all_group_content_latest`
#[derive(Clone)]
pub struct GetAllGroupContentMacroInput {
    pub group_id: ActionHash,
}

/// Get the latest evolution of all content targets in a group
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input template is [`GetAllGroupContentMacroInput`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `get_all_group_content_targets_shortcuts`
///
/// Returns [`LinkPointerMap`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// let group_id = ActionHash::try_from("uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7").unwrap();
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let link_map = get_all_group_content_latest!({
///     group_id: group_id,
/// })?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let link_map = get_all_group_content_latest!(
///     "coop_content_csr_renamed",
///     {
///         group_id: group_id,
///     }
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let link_map = get_all_group_content_latest!(
///     "custom_coop_content_csr",
///     "get_all_group_content",
///     {
///         group_id: group_id,
///     }
/// )?;
/// ```
#[macro_export]
macro_rules! get_all_group_content_latest {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            type Response = hdk::prelude::ExternResult<coop_content_types::LinkPointerMap>;
            let input = coop_content_types::GetAllGroupContentMacroInput $($def)*;
            let result : Response = coop_content_types::call_local_zome_decode!(
                $zome,
                $fn_name,
                input.group_id
            );
            result
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::get_all_group_content_latest!( $zome, "get_all_group_content_targets_shortcuts", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::get_all_group_content_latest!( "coop_content_csr", $($def)* )
    };
}


/// Create a new group
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input must be a [`GroupEntry`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `create_group`
///
/// Returns [`ActionHash`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// let group = GroupEntry {
///     admins: vec![ agent_info()?.agent_initial_pubkey ],
///     members: vec![],
///     deleted: None,
///     published_at: 1688078994936,
///     last_updated: 1688078994936,
///     metadata: BTreeMap::new(),
/// };
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let create_addr = create_group!( group )?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let create_addr = create_group!(
///     "coop_content_csr_renamed",
///     group
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let create_addr = create_group!(
///     "custom_coop_content_csr",
///     "new_group",
///     group
/// )?;
/// ```
#[macro_export]
macro_rules! create_group {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            let input : GroupEntry = $($def)*;
            coop_content_types::call_local_zome_decode!(
                ActionHash,
                $zome,
                $fn_name,
                input
            )
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::create_group!( $zome, "create_group", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::create_group!( "coop_content_csr", $($def)* )
    };
}


/// Get a group's latest state
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input must be a [`ActionHash`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `get_group`
///
/// Returns [`ActionHash`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// let group_id = ActionHash::try_from("uhCkkrVjqWkvcFoq2Aw4LOSe6Yx9OgQLMNG-DiXqtT0nLx8uIM2j7").unwrap();
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let group = get_group!( group )?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let group = get_group!(
///     "coop_content_csr_renamed",
///     group
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let group = get_group!(
///     "custom_coop_content_csr",
///     "new_group",
///     group
/// )?;
/// ```
#[macro_export]
macro_rules! get_group {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            let input : ActionHash = $($def)*;
            coop_content_types::call_local_zome_decode!(
                GroupEntry,
                $zome,
                $fn_name,
                input
            )
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::get_group!( $zome, "get_group", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::get_group!( "coop_content_csr", $($def)* )
    };
}


/// Update a new group
///
/// Rule patterns
/// - #1 - `<zome name>, <function name>, <template>`
/// - #2 - `<zome name>, <template>`
/// - #3 - `<template>`
///
/// The input template is [`UpdateEntryInput<GroupEntry>`].
///
/// This macro makes a local zome call using these default values:
/// - Zome name: `coop_content_csr`
/// - Function name: `update_group`
///
/// Returns [`ActionHash`]
///
/// #### Examples
/// All examples assume this setup
/// ```
/// let member_id = AgentPubKey::try_from("uhCAkP5vqve5GTqb0-zcVcPsGUFrmp27SMzEoAX1W3HlxYqYesBcN").unwrap();
/// let group_update = GroupEntry {
///     admins: vec![ agent_info()?.agent_initial_pubkey ],
///     members: vec![ member_id ],
///     deleted: None,
///     published_at: 1688078994936,
///     last_updated: 1688090053659,
///     metadata: BTreeMap::new(),
/// };
/// ```
///
/// ##### Example: Basic Usage
/// ```
/// let update_addr = update_group!({
///     base: create_addr,
///     entry: group_update,
/// })?;
/// ```
///
/// ##### Example: Custom Zome Name
/// ```
/// let update_addr = update_group!(
///     "coop_content_csr_renamed",
///     {
///         base: create_addr,
///         entry: group_update,
///     }
/// )?;
/// ```
///
/// ##### Example: Custom Zome and Function Names
/// ```
/// let update_addr = update_group!(
///     "custom_coop_content_csr",
///     "fetch_group",
///     {
///         base: create_addr,
///         entry: group_update,
///     }
/// )?;
/// ```
#[macro_export]
macro_rules! update_group {
    ( $zome:literal, $fn_name:literal, $($def:tt)* ) => {
        {
            let input = coop_content_types::UpdateEntryInput::<GroupEntry> $($def)*;
            coop_content_types::call_local_zome_decode!(
                ActionHash,
                $zome,
                $fn_name,
                input
            )
        }
    };
    ( $zome:literal, $($def:tt)* ) => {
        coop_content_types::update_group!( $zome, "update_group", $($def)* )
    };
    ( $($def:tt)* ) => {
        coop_content_types::update_group!( "coop_content_csr", $($def)* )
    };
}
