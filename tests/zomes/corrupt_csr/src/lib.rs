pub use coop_content::hdk;
pub use coop_content::hdi_extensions;
pub use coop_content::coop_content_types;

use hdk::prelude::*;
use hdi_extensions::{
    ScopedTypeConnector,
};
use coop_content::{
    LinkTypes,
};
use coop_content_types::{
    GroupAuthAnchorEntry,
    GroupAuthArchiveAnchorEntry,
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


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidAuthAnchorInput {
    group_id: ActionHash,
    anchor_agent: AgentPubKey,
    target: ActionHash,
}

#[hdk_extern]
pub fn invalid_auth_anchor_link(input: InvalidAuthAnchorInput) -> ExternResult<()> {
    debug!("InvalidAuthAnchorInput: {:#?}", input );
    let anchor = GroupAuthAnchorEntry( input.group_id.to_owned(), input.anchor_agent.to_owned() );
    let anchor_hash = hash_entry( &anchor )?;

    create_link( anchor_hash, input.target, LinkTypes::Content, () )?;

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidGroupAuthInput {
    group_id: ActionHash,
    group_rev: ActionHash,
    anchor_agent: AgentPubKey,
}

#[hdk_extern]
pub fn invalid_group_auth_link(input: InvalidGroupAuthInput) -> ExternResult<()> {
    debug!("InvalidGroupAuthInput: {:#?}", input );
    let anchor = GroupAuthAnchorEntry( input.group_id, input.anchor_agent );
    create_entry( anchor.to_input() )?;
    let anchor_hash = hash_entry( &anchor )?;

    create_link( input.group_rev, anchor_hash, LinkTypes::GroupAuth, () )?;

    Ok(())
}


#[hdk_extern]
pub fn delete_group_auth_link(input: InvalidGroupAuthInput) -> ExternResult<()> {
    let anchor = GroupAuthAnchorEntry( input.group_id, input.anchor_agent );
    let anchor_hash = hash_entry( &anchor )?;
    let links = get_links( input.group_rev.clone(), LinkTypes::GroupAuth, None )?;

    for link in links {
        if link.target == anchor_hash.clone().into() {
            debug!("Deleting link create: {}", link.create_link_hash );
            delete_link( link.create_link_hash.clone() )?;
        }
    }

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidGroupAuthArchiveLinkInput {
    group_rev: ActionHash,
    anchor_agent: AgentPubKey,
}

#[hdk_extern]
pub fn invalid_group_auth_archive_link(input: InvalidGroupAuthArchiveLinkInput) -> ExternResult<()> {
    debug!("InvalidGroupAuthArchiveLinkInput: {:#?}", input );
    let anchor = GroupAuthArchiveAnchorEntry::new( input.group_rev.to_owned(), input.anchor_agent );
    create_entry( anchor.to_input() )?;
    let anchor_hash = hash_entry( &anchor )?;

    create_link( input.group_rev, anchor_hash, LinkTypes::GroupAuthArchive, () )?;

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidArchiveLinkInput {
    group_rev: ActionHash,
    archived_agent: AgentPubKey,
    target: ActionHash,
}

#[hdk_extern]
pub fn invalid_archive_link(input: InvalidArchiveLinkInput) -> ExternResult<()> {
    debug!("InvalidArchiveLinkInput: {:#?}", input );
    let anchor = GroupAuthArchiveAnchorEntry::new( input.group_rev, input.archived_agent );
    create_entry( anchor.to_input() )?;
    let archive_anchor_hash = hash_entry( &anchor )?;
    create_link( archive_anchor_hash.to_owned(), input.target.to_owned(), LinkTypes::Content, () )?;

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidLinkBaseInput {
    base: AnyLinkableHash,
    target: ActionHash,
}

#[hdk_extern]
pub fn invalid_content_link_base(input: InvalidLinkBaseInput) -> ExternResult<()> {
    debug!("InvalidLinkBaseInput: {:#?}", input );
    create_link( input.base.to_owned(), input.target.to_owned(), LinkTypes::Content, () )?;

    Ok(())
}


#[hdk_extern]
pub fn invalid_group_auth_link_base(input: InvalidLinkBaseInput) -> ExternResult<()> {
    debug!("InvalidLinkBaseInput: {:#?}", input );
    create_link( input.base.to_owned(), input.target.to_owned(), LinkTypes::GroupAuth, () )?;

    Ok(())
}


#[hdk_extern]
pub fn delete_group(addr: ActionHash) -> ExternResult<()> {
    debug!("Delete group: {}", addr );
    delete_entry( addr )?;

    Ok(())
}
