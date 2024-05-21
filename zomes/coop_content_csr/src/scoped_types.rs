pub mod entry_traits;

use crate::hdk::prelude::*;
use crate::hdi_extensions::{
    guest_error,
    AnyLinkableHashTransformer,
};
use coop_content::{
    LinkTypes,
};
use coop_content_sdk::{
    create_link_input,

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
        let links = get_links(
            create_link_input(
                base,
                &LinkTypes::GroupAuth,
                &None::<()>,
            )?
        )?;

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
        let links = get_links(
            create_link_input(
                base,
                &LinkTypes::GroupAuthArchive,
                &None::<()>,
            )?
        )?;

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

    fn create_targets(
        &self,
        content_type: Option<String>,
        content_base: Option<String>
    ) -> ExternResult<Vec<AnyLinkableHash>> {
        if content_type.is_none() && content_base.is_some() {
            Err(guest_error!(format!(
                "'content_type' cannot be None if 'content_base' is Some(..); type={:?} base={:?}",
                content_type, content_base,
            )))?
        }

        let base = hash_entry( self )?;
        let content_type = content_type.unwrap_or("".to_string());
        let tag = match content_base {
            Some(base) => format!("{}:{}", content_type, base ),
            None => content_type,
        };
        debug!("Get links {}<{:?}> =[{}]=> *", base, LinkTypes::Contribution, tag );

        Ok(
            get_links(
                create_link_input(
                    &base,
                    &LinkTypes::Contribution,
                    &Some(tag.as_str().as_bytes().to_vec()),
                )?
            )?
                .into_iter()
                .map(|link| link.target )
                .collect()
        )
    }

    fn update_links(&self) -> ExternResult<Vec<Link>> {
        get_links(
            create_link_input(
                &self.base_hash()?,
                &LinkTypes::ContributionUpdate,
                &None::<()>,
            )?
        )
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

    fn create_targets(
        &self,
        content_type: Option<String>,
        content_base: Option<String>
    ) -> ExternResult<Vec<AnyLinkableHash>> {
        if content_type.is_none() && content_base.is_some() {
            Err(guest_error!(format!(
                "'content_type' cannot be None if 'content_base' is Some(..); type={:?} base={:?}",
                content_type, content_base,
            )))?
        }

        let base = self.base_hash()?;
        let content_type = content_type.unwrap_or("".to_string());
        let tag = match content_base {
            Some(base) => format!("{}:{}", content_type, base ),
            None => content_type,
        };
        debug!("Get links {}<{:?}> =[{}]=> *", base, LinkTypes::Contribution, tag );

        Ok(
            get_links(
                create_link_input(
                    &base,
                    &LinkTypes::Contribution,
                    &Some(tag.as_str().as_bytes().to_vec()),
                )?
            )?
                .into_iter()
                .map(|link| link.target )
                .collect()
        )
    }

    fn update_links(&self) -> ExternResult<Vec<Link>> {
        get_links(
            create_link_input(
                &self.base_hash()?,
                &LinkTypes::ContributionUpdate,
                &None::<()>,
            )?
        )
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
