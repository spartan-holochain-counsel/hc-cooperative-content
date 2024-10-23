mod scoped_types;

pub use coop_content::hdi;
pub use coop_content::hdi_extensions;
pub use coop_content_sdk::hdk;
pub use coop_content_sdk::hdk_extensions;

use std::collections::HashMap;
use lazy_static::lazy_static;
use hdk::prelude::*;
use hdk_extensions::{
    agent_id,
    must_get,
    exists,
    resolve_action_addr,
    follow_evolutions,
    follow_evolutions_using_authorities_with_exceptions,
    // Input Structs
    UpdateEntryInput,
    GetLinksInput,
};
pub use scoped_types::entry_traits::*;
use hdi_extensions::{
    trace_origin_root,
    ScopedTypeConnector,
    // Macros
    guest_error,
};
use coop_content::{
    EntryTypes,
    EntryTypesUnit,
    LinkTypes,
    // Entry Structs
    GroupEntry,
    ContributionsAnchorEntry,
    ArchivedContributionsAnchorEntry,
    ContributionAnchorTypes,
};
use coop_content_sdk::{
    create_link_input,

    // Input Structs
    GroupAuthInput,
    GetAllGroupContentInput,
    GetGroupContentInput,
    CreateContributionLinkInput,
    CreateContributionUpdateLinkInput,
};


lazy_static! {
    static ref ZOME_NAME: String = match zome_info() {
        Ok(info) => format!("{}", info.name ),
        Err(_) => String::from("?"),
    };
}

type LinkPointerMap = HashMap<AnyLinkableHash, AnyLinkableHash>;
type EvolutionMap = HashMap<AnyLinkableHash, Vec<AnyLinkableHash>>;


fn create_if_not_exists<'a, T, E, E2>(entry: &'a T) -> ExternResult<Option<ActionHash>>
where
    T: ScopedTypeConnector<EntryTypes, EntryTypesUnit>,
    Entry: TryFrom<&'a T, Error = E> + TryFrom<T, Error = E2>,
    WasmError: From<E> + From<E2>,
{
    Ok(
        match exists( &hash_entry( entry )? )? {
            true => None,
            false => Some( create_entry( entry.to_input() )? ),
        }
    )
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



/// The context and content of a specific entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity<T> {
    /// The address of the original create action
    pub id: ActionHash,

    /// The create/update action of the current entry
    pub action: ActionHash,

    /// The address of the current entry
    pub address: EntryHash,

    #[serde(rename = "type")]
    /// An identifier for the content's type and structure
    pub ctype: String,

    /// The entity's current value
    pub content: T,
}


/// Create a new group entity
#[hdk_extern]
pub fn create_group(group: GroupEntry) -> ExternResult<Entity<GroupEntry>> {
    debug!("Creating new group entry: {:#?}", group );
    let action_hash = create_entry( group.to_input() )?;
    let entry_hash = hash_entry( &group )?;
    let agent_id = agent_id()?;

    for pubkey in group.contributors() {
        let anchor = ContributionsAnchorEntry( action_hash.to_owned(), pubkey );
        let anchor_hash = hash_entry( &anchor )?;
        debug!("Creating contributions anchor ({}): {:#?}", anchor_hash, anchor );
        create_entry( anchor.to_input() )?;
        create_link( action_hash.to_owned(), anchor_hash, LinkTypes::GroupAuth, () )?;
    }

    create_link( agent_id, action_hash.to_owned(), LinkTypes::Group, () )?;

    Ok(Entity {
        id: action_hash.clone(),
        action: action_hash,
        address: entry_hash,
        ctype: "group".to_string(),
        content: group,
    })
}


/// Update a group
#[hdk_extern]
pub fn update_group(input: UpdateEntryInput<GroupEntry>) -> ExternResult<Entity<GroupEntry>> {
    debug!("Update group action: {}", input.base );
    let group_id = trace_origin_root( &input.base )?.0;
    let prev_group : GroupEntry = must_get( &input.base )?.try_into()?;
    let contributors_diff = prev_group.contributors_diff( &input.entry );

    let action_hash = update_entry( input.base.to_owned(), input.entry.to_input() )?;
    let entry_hash = hash_entry( &input.entry )?;

    let archive_links = get_links(
        create_link_input(
            &input.base,
            &LinkTypes::GroupAuthArchive,
            &None::<()>,
        )?
    )?;
    for link in archive_links {
        create_link( action_hash.to_owned(), link.target, LinkTypes::GroupAuthArchive, link.tag )?;
    }

    for pubkey in contributors_diff.removed {
        debug!("Removed Agent: {}", pubkey );
        let anchor = ContributionsAnchorEntry( group_id.to_owned(), pubkey.to_owned() );
        let anchor_hash = hash_entry( &anchor )?;
        let archive_anchor = ArchivedContributionsAnchorEntry::new( action_hash.to_owned(), pubkey.to_owned() );
        let archive_anchor_hash = hash_entry( &archive_anchor )?;

        create_if_not_exists( &archive_anchor )?;
        create_link( action_hash.to_owned(), archive_anchor_hash.to_owned(), LinkTypes::GroupAuthArchive, () )?;

        let creates = get_links(
            create_link_input(
                &anchor_hash,
                &LinkTypes::Contribution,
                &None::<()>,
            )?
        )?;
        let updates = get_links(
            create_link_input(
                &anchor_hash,
                &LinkTypes::ContributionUpdate,
                &None::<()>,
            )?
        )?;

        debug!("Copying {} creates for auth archive: {}", creates.len(), pubkey );
        for link in creates {
            create_link( archive_anchor_hash.to_owned(), link.target, LinkTypes::Contribution, link.tag )?;
        }

        debug!("Copying {} updates for auth archive: {}", updates.len(), pubkey );
        for link in updates {
            create_link( archive_anchor_hash.to_owned(), link.target, LinkTypes::ContributionUpdate, link.tag )?;
        }
    }

    for pubkey in contributors_diff.added {
        debug!("Added Agent: {}", pubkey );
        let anchor = ContributionsAnchorEntry( group_id.to_owned(), pubkey.to_owned() );
        let anchor_hash = hash_entry( &anchor )?;
        create_if_not_exists( &anchor )?;
        create_link( action_hash.to_owned(), anchor_hash, LinkTypes::GroupAuth, () )?;
    }

    for pubkey in contributors_diff.intersection {
        debug!("Unchanged Agent: {}", pubkey );
        let anchor = ContributionsAnchorEntry( group_id.to_owned(), pubkey.to_owned() );
        let anchor_hash = hash_entry( &anchor )?;
        create_link( action_hash.to_owned(), anchor_hash, LinkTypes::GroupAuth, () )?;
    }

    Ok(Entity {
        id: group_id,
        action: action_hash,
        address: entry_hash,
        ctype: "group".to_string(),
        content: input.entry,
    })
}


/// Get the current state for the given group ID
#[hdk_extern]
pub fn get_group(group_id: ActionHash) -> ExternResult<Entity<GroupEntry>> {
    if group_id != trace_origin_root( &group_id )?.0 {
        Err(guest_error!(format!(
            "Action hash '{}' is not an ID",
            group_id,
        )))?
    }
    debug!("Get latest group entry: {}", group_id );
    let latest_addr = follow_evolutions( &group_id )?.last().unwrap().to_owned();
    let record = must_get( &latest_addr )?;

    let group = GroupEntry::try_from_record( &record )?;

    Ok(Entity {
        id: group_id,
        action: record.action_address().to_owned(),
        address: record.action().entry_hash()
            .ok_or(guest_error!("Record does not have an entry".to_string()))?
            .to_owned(),
        ctype: "group".to_string(),
        content: group,
    })
}


/// Get all group content with the optional 'content type' and 'full trace' filters
#[hdk_extern]
pub fn get_all_group_content_targets(input: GetAllGroupContentInput) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash)>> {
    match input.full_trace {
        None | Some(false) => get_all_group_content_targets_shortcuts( (input.group_id, input.content_type, input.content_base) ),
        Some(true) => get_all_group_content_targets_full_trace( (input.group_id, input.content_type, input.content_base) ),
    }
}


/// Get all group content using full trace with the optional 'content type' filter
#[hdk_extern]
pub fn get_all_group_content_targets_full_trace(
    (group_id, content_type, content_base): (ActionHash, Option<String>, Option<String>)
) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash)>> {
    debug!("Get latest group content: {}", group_id );
    let latest_addr = follow_evolutions( &group_id )?.last().unwrap().to_owned();
    let record = must_get( &latest_addr )?;
    let group_rev = record.action_address().to_owned();
    let group : GroupEntry = record.try_into()?;

    let mut content_creates = vec![];
    let mut archived_updates : Vec<ActionHash> = vec![];

    let auth_archive_anchors = GroupEntry::group_auth_archive_anchor_hashes( &group_rev )?;

    debug!("Found {} auth archives for group rev '{}'", auth_archive_anchors.len(), group_rev );
    for auth_archive_addr in auth_archive_anchors.iter() {
        let anchor : ArchivedContributionsAnchorEntry = must_get( auth_archive_addr )?.try_into()?;
        content_creates.extend( anchor.create_targets( content_type.clone(), content_base.clone() )? );

        let archive_updates = anchor.update_targets()?;
        let update_actions : Vec<ActionHash> = archive_updates.iter()
            .cloned()
            .filter_map(|target| target.into_action_hash() )
            .collect();
        debug!("Removed {}/{} archive updates because they were not ActionHash targets", archive_updates.len() - update_actions.len(), archive_updates.len() );
        archived_updates.extend( update_actions );
    }

    let group_auth_anchors = GroupEntry::group_auth_anchor_hashes( &group_rev )?;

    debug!("Found {} current contributors for group rev '{}'", group_auth_anchors.len(), group_rev );
    for auth_anchor_addr in group_auth_anchors.iter() {
        let anchor : ContributionsAnchorEntry = must_get( auth_anchor_addr )?.try_into()?;
        let content_targets = anchor.create_targets( content_type.clone(), content_base.clone() )?;
        debug!("Found {} content links for group contributor '{}'", content_targets.len(), anchor.1 );
        content_creates.extend( content_targets );
    }

    let mut targets = vec![];

    for content_addr in content_creates {
        match content_addr.clone().into_action_hash() {
            Some(addr) => {
                let evolutions = follow_evolutions_using_authorities_with_exceptions( &addr, &group.contributors(), &archived_updates )?;
                targets.push((
                    content_addr,
                    evolutions.last().unwrap().to_owned().into()
                ));
            },
            None => continue,
        }
    }

    Ok( targets )
}


fn follow_update_map(
    start: &AnyLinkableHash,
    updates: &LinkPointerMap
) -> Vec<AnyLinkableHash> {
    let mut link_map = updates.clone();
    let mut evolutions = vec![ start.to_owned() ];
    let mut base = start.to_owned();

    while let Some(next_addr) = link_map.remove( &base ) {
        evolutions.push( next_addr.to_owned() );
        base = next_addr;
    }

    evolutions
}

/// Get all revisions of group content using shortcuts with the optional 'content type' filter
#[hdk_extern]
pub fn follow_all_group_content_evolutions_shortcuts(
    (group_id, content_type, content_base): (ActionHash, Option<String>, Option<String>)
) -> ExternResult<Vec<(AnyLinkableHash, Vec<AnyLinkableHash>)>> {
    debug!("Get latest group content: {}", group_id );
    let latest_addr = follow_evolutions( &group_id )?.last().unwrap().to_owned();
    let record = must_get( &latest_addr )?;
    let group_rev = record.action_address().to_owned();

    let mut targets = vec![];
    let mut updates = HashMap::new();

    let auth_archive_anchors = GroupEntry::group_auth_archive_anchor_hashes( &group_rev )?;

    debug!("Found {} auth archives for group rev '{}'", auth_archive_anchors.len(), group_rev );
    for auth_archive_addr in auth_archive_anchors.iter() {
        let anchor : ArchivedContributionsAnchorEntry = must_get( auth_archive_addr )?.try_into()?;
        debug!("Auth archive anchor: {:#?}", anchor );

        let content_ids = anchor.create_targets( content_type.clone(), content_base.clone() )?;
        debug!("Found {} content IDs: {:#?}", content_ids.len(), content_ids );
        targets.extend( content_ids );

        let shortcuts = anchor.shortcuts()?;
        debug!("Found {} content update shortcuts: {:#?}", shortcuts.len(), shortcuts );
        for (_,base,target) in shortcuts {
            updates.insert( base, target );
        }
    }

    let group_auth_anchors = GroupEntry::group_auth_anchor_hashes( &group_rev )?;

    debug!("Found {} current authorities for group rev '{}'", group_auth_anchors.len(), group_rev );
    for auth_anchor_addr in group_auth_anchors.iter() {
        let anchor : ContributionsAnchorEntry = must_get( auth_anchor_addr )?.try_into()?;
        debug!("Auth anchor: {:#?}", anchor );

        let content_ids = anchor.create_targets( content_type.clone(), content_base.clone() )?;
        debug!("Found {} content IDs: {:#?}", content_ids.len(), content_ids );
        targets.extend( content_ids );

        let shortcuts = anchor.shortcuts()?;
        debug!("Found {} content update shortcuts: {:#?}", shortcuts.len(), shortcuts );
        for (_,base,target) in shortcuts {
            updates.insert( base, target );
        }
    }

    let mut content_evolutions = vec![];

    for addr in targets {
        content_evolutions.push((
            addr.clone(),
            follow_update_map( &addr, &updates )
        ));
    }

    Ok( content_evolutions )
}


/// Get all group content using shortcuts with the optional 'content type' filter
#[hdk_extern]
pub fn get_all_group_content_targets_shortcuts(
    (group_id, content_type, content_base): (ActionHash, Option<String>, Option<String>)
) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash)>> {
    Ok(
        follow_all_group_content_evolutions_shortcuts( (group_id, content_type, content_base) )?.into_iter()
            .filter_map( |(key, evolutions)| {
                let latest_addr = evolutions.last()?.to_owned();
                Some( (key, latest_addr) )
            })
            .collect()
    )
}


/// Calculate the [`EntryHash`] for a [`ContributionsAnchorEntry`]
#[hdk_extern]
pub fn group_auth_anchor_hash(input: GroupAuthInput) -> ExternResult<EntryHash> {
    Ok( hash_entry( ContributionsAnchorEntry( input.group_id, input.author ) )? )
}


/// Calculate the [`EntryHash`] for a [`ArchivedContributionsAnchorEntry`]
#[hdk_extern]
pub fn group_auth_archive_anchor_hash(input: GroupAuthInput) -> ExternResult<EntryHash> {
    Ok( hash_entry( ArchivedContributionsAnchorEntry::new( input.group_id, input.author ) )? )
}


/// Attach some new content to the given group
#[hdk_extern]
pub fn create_content_link(input: CreateContributionLinkInput) -> ExternResult<ActionHash> {
    let author = agent_id()?;
    let tag = format!("#{}#:{}", input.content_type, input.content_base.unwrap_or("".to_string()) );
    debug!("Creating content link from ContributionsAnchorEntry( {}, {} ) =[{}]=> {}", input.group_id, author, tag, input.content_target );
    let anchor = ContributionsAnchorEntry( input.group_id, author );
    let anchor_hash = hash_entry( &anchor )?;

    create_if_not_exists( &anchor )?;

    Ok(
        create_link(
            anchor_hash,
            input.content_target,
            LinkTypes::Contribution,
            tag.as_str().as_bytes().to_vec()
        )?
    )
}


/// Register an update to some content for the given group
#[hdk_extern]
pub fn create_content_update_link(input: CreateContributionUpdateLinkInput) -> ExternResult<ActionHash> {
    let author = agent_id()?;
    let tag = format!("{}:{}", input.content_id, input.content_prev );
    let anchor = ContributionsAnchorEntry( input.group_id, author );
    let anchor_hash = hash_entry( &anchor )?;
    debug!("Auth anchor: {:#?}", anchor );

    create_if_not_exists( &anchor )?;

    debug!("Creating content update link from {} --'{}'--> {}", anchor_hash, tag, input.content_next );
    Ok(
        create_link(
            anchor_hash,
            input.content_next,
            LinkTypes::ContributionUpdate,
            tag.into_bytes()
        )?
    )
}


/// Delete any links to the given contribution from the given author
#[hdk_extern]
pub fn delete_group_auth_anchor_content_links(input: (GroupAuthInput, AnyLinkableHash)) -> ExternResult<Vec<ActionHash>> {
    debug!("Input: {:#?}", input );
    let base = match input.0.anchor_type {
        ContributionAnchorTypes::Active => {
            let anchor = ContributionsAnchorEntry( input.0.group_id, input.0.author );
            debug!("Delete input anchor: {:#?}", anchor );
            hash_entry( anchor )?
        },
        ContributionAnchorTypes::Archive => {
            let anchor = ArchivedContributionsAnchorEntry::new( input.0.group_id, input.0.author );
            debug!("Delete input anchor: {:#?}", anchor );
            hash_entry( anchor )?
        },
    };

    let link_types = vec![
        LinkTypes::Contribution,
        LinkTypes::ContributionUpdate,
    ];
    let links = get_links(
        create_link_input(
            &base,
            &link_types,
            &None::<()>,
        )?
    )?;
    let mut deleted = vec![];

    for link in links {
        if link.target == input.1 {
            delete_link( link.create_link_hash.clone() )?;
            deleted.push( link.create_link_hash );
        }
    }

    Ok( deleted )
}


/// Get all evolutions for the given content from the perspective of the given group with the
/// optional 'full trace' filter
#[hdk_extern]
pub fn get_group_content_evolutions(
    input: GetGroupContentInput
) -> ExternResult<Vec<AnyLinkableHash>> {
    debug!("Get group content evolutions: {:?}", input );
    match input.full_trace {
        None | Some(false) => get_group_content_evolutions_shortcuts( input ),
        Some(true) => get_group_content_evolutions_full_trace( input ),
    }
}


/// Get all evolutions for the given content from the perspective of the given group using full trace
#[hdk_extern]
pub fn get_group_content_evolutions_full_trace(
    input: GetGroupContentInput
) -> ExternResult<Vec<AnyLinkableHash>> {
    debug!("Get group ({}) content evolutions (full-trace): {}", input.group_id, input.content_id );
    let base_addr = resolve_action_addr( &input.content_id )?;
    let latest_addr = follow_evolutions( &input.group_id )?.last().unwrap().to_owned();
    let record = must_get( &latest_addr )?;
    let group_rev = record.action_address().to_owned();
    let group : GroupEntry = record.try_into()?;

    let mut archived_updates : Vec<ActionHash> = vec![];
    let auth_archive_anchors = GroupEntry::group_auth_archive_anchor_hashes( &group_rev )?;

    debug!("Found {} auth archives for group rev '{}'", auth_archive_anchors.len(), group_rev );
    for auth_archive_addr in auth_archive_anchors.iter() {
        let anchor : ArchivedContributionsAnchorEntry = must_get( auth_archive_addr )?.try_into()?;

        let archive_updates = anchor.update_targets()?;
        let update_actions : Vec<ActionHash> = archive_updates.iter()
            .cloned()
            .filter_map(|target| target.into_action_hash() )
            .collect();
        debug!("Removed {}/{} archive updates because they were not ActionHash targets", archive_updates.len() - update_actions.len(), archive_updates.len() );
        archived_updates.extend( update_actions );
    }

    Ok(
        follow_evolutions_using_authorities_with_exceptions(
            &base_addr,
            &group.contributors(),
            &archived_updates
        )?.into_iter().map( |hash| hash.into() ).collect()
    )
}


/// Get all evolutions for the given content from the perspective of the given group using shortcuts
#[hdk_extern]
pub fn get_group_content_evolutions_shortcuts(
    input: GetGroupContentInput
) -> ExternResult<Vec<AnyLinkableHash>> {
    debug!("Get group ({}) content evolutions (shortcuts): {}", input.group_id, input.content_id );
    let all_content_evolutions : EvolutionMap = follow_all_group_content_evolutions_shortcuts( (input.group_id, None, None) )?
        .into_iter().collect();

    debug!("Looking for {} in: {:#?}", input.content_id, all_content_evolutions );
    let evolutions = all_content_evolutions.get( &input.content_id.clone().into() )
        .ok_or(guest_error!(format!("Content ID ({}) is not in group content: {:?}", input.content_id, all_content_evolutions.keys() )))?
        .to_owned();

    Ok( evolutions )
}


/// Get the current state for the given content from the perspective of the given group with the
/// optional 'full trace' filter
#[hdk_extern]
pub fn get_group_content_latest(
    input: GetGroupContentInput
) -> ExternResult<AnyLinkableHash> {
    debug!("Get group content latest: {:?}", input );
    match input.full_trace {
        None | Some(false) => get_group_content_latest_shortcuts( input ),
        Some(true) => get_group_content_latest_full_trace( input ),
    }
}


/// Get the current state for the given content from the perspective of the given group using full trace
#[hdk_extern]
pub fn get_group_content_latest_full_trace(
    input: GetGroupContentInput
) -> ExternResult<AnyLinkableHash> {
    debug!("Get group ({}) content latest (full-trace): {}", input.group_id, input.content_id );
    Ok(
        get_group_content_evolutions_full_trace( input )?
            .last().unwrap().to_owned()
    )
}


/// Get the current state for the given content from the perspective of the given group using shortcuts
#[hdk_extern]
pub fn get_group_content_latest_shortcuts(
    input: GetGroupContentInput
) -> ExternResult<AnyLinkableHash> {
    debug!("Get group ({}) content latest (shortcuts): {}", input.group_id, input.content_id );
    Ok(
        get_group_content_evolutions_shortcuts( input )?
            .last().unwrap().to_owned()
    )
}



//
// Generic
//
/// Delete all links that match the link input
#[hdk_extern]
pub fn delete_matching_links(input: GetLinksInput<LinkTypes>) -> ExternResult<Vec<ActionHash>> {
    debug!("GetLinksInput: {:#?}", input );
    let links = get_links(
        create_link_input(
            &input.base,
            &input.link_type_filter,
            &input.tag,
        )?
    )?;
    let mut deleted = vec![];

    for link in links {
        if link.target == input.target {
            delete_link( link.create_link_hash.clone() )?;
            deleted.push( link.create_link_hash );
        }
    }

    Ok( deleted )
}
