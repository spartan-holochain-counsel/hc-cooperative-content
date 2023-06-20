pub mod entry_traits;

use hdk::prelude::*;
use coop_content::{
    LinkTypes,

    // Entry Structs
    GroupEntry,
    GroupAuthAnchorEntry,
    GroupAuthArchiveAnchorEntry,
};
pub use entry_traits::{
    GroupLinks,
    GroupAuthLinks,
    GroupAuthArchiveLinks,
};



impl GroupLinks for GroupEntry {
    fn group_auth_addrs(base: &ActionHash) -> ExternResult<Vec<EntryHash>> {
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

    fn group_auth_archive_addrs(base: &ActionHash) -> ExternResult<Vec<EntryHash>> {
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

impl GroupAuthLinks for GroupAuthAnchorEntry {
    fn base_address(&self) -> ExternResult<EntryHash> {
	hash_entry( self )
    }

    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
	let base = hash_entry( self )?;
	Ok(
	    get_links( base, LinkTypes::Content, None )?
		.into_iter()
		.map(|link| link.target )
		.collect()
	)
    }

    fn update_links(&self) -> ExternResult<Vec<Link>> {
	get_links( self.base_address()?, LinkTypes::ContentUpdate, None )
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
			debug!("Content update link has malformed tag: {}", tag_str );
			None
		    })?;

		Some((
		    AnyLinkableHash::from( ActionHash::try_from( tag_id.to_string()).ok()? ),
		    AnyLinkableHash::from( ActionHash::try_from( tag_rev.to_string()).ok()? ),
		    link.target
		))
	    })
	    .collect())
    }
}

impl GroupAuthArchiveLinks for GroupAuthArchiveAnchorEntry {
    fn base_address(&self) -> ExternResult<EntryHash> {
	hash_entry( self )
    }

    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
	let base = self.base_address()?;
	Ok(
	    get_links( base, LinkTypes::Content, None )?
		.into_iter()
		.map(|link| link.target )
		.collect()
	)
    }

    fn update_links(&self) -> ExternResult<Vec<Link>> {
	get_links( self.base_address()?, LinkTypes::ContentUpdate, None )
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
			debug!("Content update link has malformed tag: {}", tag_str );
			None
		    })?;

		Some((
		    AnyLinkableHash::from( ActionHash::try_from( tag_id.to_string()).ok()? ),
		    AnyLinkableHash::from( ActionHash::try_from( tag_rev.to_string()).ok()? ),
		    link.target
		))
	    })
	   .collect())
    }
}
