pub mod entry_traits;

use crate::hdk::prelude::*;
use crate::hdi_extensions::{
    AnyLinkableHashTransformer,
};
use coop_content::{
    LinkTypes,
};
use coop_content_sdk::{
    // Entry Structs
    GroupEntry,
    ContributionsAnchorEntry,
    ArchivedContributionsAnchorEntry,
};
pub use entry_traits::{
    GroupLinks,
    ContributionsLinks,
    ArchivedContributionsLinks,
};



impl GroupLinks for GroupEntry {
    fn group_auth_anchor_hashes(base: &ActionHash) -> ExternResult<Vec<EntryHash>> {
        let links = get_links( base.to_owned(), LinkTypes::GroupAuth, None )?;

        Ok(
            links.into_iter()
                .filter_map(|link| {
                    link.target.into_entry_hash().or_else(|| {
                        debug!("WARNING: Should be unreachable because LinkTypes::GroupAuth validation only allows EntryHash target");
                        None
                    })
                })
                .collect()
        )
    }

    fn group_auth_archive_anchor_hashes(base: &ActionHash) -> ExternResult<Vec<EntryHash>> {
        let links = get_links( base.to_owned(), LinkTypes::GroupAuthArchive, None )?;

        Ok(
            links.into_iter()
                .filter_map(|link| {
                    link.target.into_entry_hash().or_else(|| {
                        debug!("WARNING: Should be unreachable because LinkTypes::GroupAuthArchive validation only allows EntryHash target");
                        None
                    })
                })
                .collect()
        )
    }
}

impl ContributionsLinks for ContributionsAnchorEntry {
    fn base_hash(&self) -> ExternResult<EntryHash> {
        hash_entry( self )
    }

    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
        let base = hash_entry( self )?;
        Ok(
            get_links( base, LinkTypes::Contribution, None )?
                .into_iter()
                .map(|link| link.target )
                .collect()
        )
    }

    fn update_links(&self) -> ExternResult<Vec<Link>> {
        get_links( self.base_hash()?, LinkTypes::ContributionUpdate, None )
    }

    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
        Ok(
            self.update_links()?
                .into_iter()
                .map(|link| link.target )
                .collect()
        )
    }

    fn shortcuts(&self) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash, AnyLinkableHash)>> {
        Ok(self.update_links()?.into_iter()
            .filter_map(|link| {
                let tag_str = String::from_utf8( link.tag.into_inner() ).ok()?;
                let (tag_id, tag_rev) = tag_str.split_once(":")
                    .or_else(|| {
                        debug!("Contribution update link has malformed tag: {}", tag_str );
                        None
                    })?;

                Some((
                    AnyLinkableHash::try_from_string( tag_id ).ok()?,
                    AnyLinkableHash::try_from_string( tag_rev ).ok()?,
                    link.target
                ))
            })
            .collect())
    }
}

impl ArchivedContributionsLinks for ArchivedContributionsAnchorEntry {
    fn base_hash(&self) -> ExternResult<EntryHash> {
        hash_entry( self )
    }

    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
        let base = self.base_hash()?;
        Ok(
            get_links( base, LinkTypes::Contribution, None )?
                .into_iter()
                .map(|link| link.target )
                .collect()
        )
    }

    fn update_links(&self) -> ExternResult<Vec<Link>> {
        get_links( self.base_hash()?, LinkTypes::ContributionUpdate, None )
    }

    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
        Ok(
            self.update_links()?
                .into_iter()
                .map(|link| link.target )
                .collect()
        )
    }

    fn shortcuts(&self) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash, AnyLinkableHash)>> {
        Ok(self.update_links()?.into_iter()
            .filter_map(|link| {
                let tag_str = String::from_utf8( link.tag.into_inner() ).ok()?;
                let (tag_id, tag_rev) = tag_str.split_once(":")
                    .or_else(|| {
                        debug!("Contribution update link has malformed tag: {}", tag_str );
                        None
                    })?;

                Some((
                    AnyLinkableHash::try_from_string( tag_id ).ok()?,
                    AnyLinkableHash::try_from_string( tag_rev ).ok()?,
                    link.target
                ))
            })
           .collect())
    }
}
