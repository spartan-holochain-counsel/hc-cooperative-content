use hdi::prelude::*;
use hdk::prelude::debug;
use hdi_extensions::{
    // Macros
    valid, invalid,
};
use crate::{
    // EntryTypes,
    LinkTypes,
    GroupEntry,
    GroupAuthAnchorEntry
};

pub fn validation(
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
    _tag: LinkTag,
    create: CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
	LinkTypes::Content => {
	    debug!("Checking LinkTypes::Content base address: {}", base_address );
	    // TODO: support any linkable hash, not just entry hash
	    let anchor_entry_hash = match base_address.to_owned().into_entry_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Content link base address must be an entry hash; not '{}'", base_address )),
	    };
	    let anchor : GroupAuthAnchorEntry = must_get_entry( anchor_entry_hash )?.content.try_into()?;

	    if anchor.1 != create.author {
		invalid!(format!("Creating a link based on an auth anchor can only be made by the matching agent ({})", anchor.1 ))
	    }

	    valid!()
	},
	LinkTypes::Group => {
	    valid!()
	},
	LinkTypes::GroupAuth => {
	    debug!("Checking LinkTypes::GroupAuth base address: {}", base_address );
	    let group_action_hash = match base_address.to_owned().into_action_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Group auth link base address must be the action hash of a group entry creation; not '{}'", base_address )),
	    };
	    let group : GroupEntry = must_get_valid_record( group_action_hash )?.try_into()?;

	    if !group.admins.contains( &create.author ) {
		invalid!("The author of a group auth link must be an admin of the group".to_string())
	    }

	    let anchor_entry_hash = match target_address.to_owned().into_entry_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Group auth link target address must be the entry hash of a group auth anchor entry creation; not '{}'", target_address )),
	    };
	    let anchor : GroupAuthAnchorEntry = must_get_entry( anchor_entry_hash )?.content.try_into()?;

	    if !group.authorities().contains( &anchor.1 ) {
		invalid!(format!("Links to group auth anchors must match an authority in the group revision they are based off of"))
	    }

	    valid!()
	},
	_ => invalid!(format!("Create validation not implemented for link type: {:#?}", link_type )),
    }
}
