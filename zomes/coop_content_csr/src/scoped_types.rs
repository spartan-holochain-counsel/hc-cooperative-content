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
    fn content_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
	let base = hash_entry( self )?;
	Ok(
	    get_links( base, LinkTypes::Content, None )?
		.into_iter()
		.map(|link| link.target )
		.collect()
	)
    }
}

impl GroupAuthArchiveLinks for GroupAuthArchiveAnchorEntry {
    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
	let base = hash_entry( self )?;
	Ok(
	    get_links( base, LinkTypes::Content, None )?
		.into_iter()
		.map(|link| link.target )
		.collect()
	)
    }

    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>> {
	let base = hash_entry( self )?;
	Ok(
	    get_links( base, LinkTypes::ContentUpdate, None )?
		.into_iter()
		.map(|link| link.target )
		.collect()
	)
    }
}
