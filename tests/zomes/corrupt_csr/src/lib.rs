pub use coop_content::hdi_extensions;
pub use coop_content_sdk::hdk;

use hdk::prelude::*;
use hdi_extensions::{
    ScopedTypeConnector,
};
use coop_content::{
    LinkTypes,
};
use coop_content_sdk::{
    ContributionsAnchorEntry,
    ArchivedContributionsAnchorEntry,
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
    let anchor = ContributionsAnchorEntry( input.group_id.to_owned(), input.anchor_agent.to_owned() );
    let anchor_hash = hash_entry( &anchor )?;

    create_link( anchor_hash, input.target, LinkTypes::Contribution, () )?;

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidContributionsInput {
    group_id: ActionHash,
    group_rev: ActionHash,
    anchor_agent: AgentPubKey,
}

#[hdk_extern]
pub fn invalid_group_auth_link(input: InvalidContributionsInput) -> ExternResult<()> {
    debug!("InvalidContributionsInput: {:#?}", input );
    let anchor = ContributionsAnchorEntry( input.group_id, input.anchor_agent );
    create_entry( anchor.to_input() )?;
    let anchor_hash = hash_entry( &anchor )?;

    create_link( input.group_rev, anchor_hash, LinkTypes::GroupAuth, () )?;

    Ok(())
}


#[hdk_extern]
pub fn delete_group_auth_link(input: InvalidContributionsInput) -> ExternResult<()> {
    let anchor = ContributionsAnchorEntry( input.group_id, input.anchor_agent );
    let anchor_hash = hash_entry( &anchor )?;
    let links = get_links(
        GetLinksInputBuilder::try_new(
            input.group_rev.clone(),
            LinkTypes::GroupAuth,
        )?.build()
    )?;

    for link in links {
        if link.target == anchor_hash.clone().into() {
            debug!("Deleting link create: {}", link.create_link_hash );
            delete_link( link.create_link_hash.clone() )?;
        }
    }

    Ok(())
}


#[derive(Clone, Deserialize, Debug)]
pub struct InvalidArchivedContributionsLinkInput {
    group_rev: ActionHash,
    anchor_agent: AgentPubKey,
}

#[hdk_extern]
pub fn invalid_group_auth_archive_link(input: InvalidArchivedContributionsLinkInput) -> ExternResult<()> {
    debug!("InvalidArchivedContributionsLinkInput: {:#?}", input );
    let anchor = ArchivedContributionsAnchorEntry::new( input.group_rev.to_owned(), input.anchor_agent );
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
    let anchor = ArchivedContributionsAnchorEntry::new( input.group_rev, input.archived_agent );
    create_entry( anchor.to_input() )?;
    let archive_anchor_hash = hash_entry( &anchor )?;
    create_link( archive_anchor_hash.to_owned(), input.target.to_owned(), LinkTypes::Contribution, () )?;

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
    create_link( input.base.to_owned(), input.target.to_owned(), LinkTypes::Contribution, () )?;

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
