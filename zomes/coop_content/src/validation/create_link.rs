use hdi::prelude::*;
use hdk::prelude::debug;
use hdi_extensions::{
    get_root_origin,
    // Macros
    valid, invalid, unwrap_validation,
};
use crate::{
    // EntryTypes,
    LinkTypes,
    GroupEntry,
    GroupAuthAnchorEntry,
    GroupAuthAnchor,
};


fn validate_content_link_base(
    base: &AnyLinkableHash,
    create: &CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    let anchor_entry_hash = match base.to_owned().into_entry_hash() {
	Some(hash) => hash,
	None => invalid!(format!("Content link base address must be an entry hash; not '{}'", base )),
    };
    let anchor : GroupAuthAnchor = match must_get_entry( anchor_entry_hash )?.content.try_into() {
	Ok(entry) => entry,
	Err(_) => invalid!(format!("A content link base address must be a group auth anchor entry")),
    };

    if anchor.is_archive() {
	let group : GroupEntry = must_get_valid_record( anchor.group().to_owned() )?.try_into()?;
	if !group.admins.contains( &create.author ) {
	    invalid!(format!("Creating a link based on an auth archive anchor can only be made by group admins"))
	}
    } else if anchor.author() != &create.author {
	invalid!(format!("Creating a link based on an auth anchor can only be made by the matching agent ({})", anchor.author() ))
    }

    valid!()
}


pub fn validation(
    base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    link_type: LinkTypes,
    tag: LinkTag,
    create: CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
	LinkTypes::Content => {
	    debug!("Checking LinkTypes::Content base address: {}", base_address );

	    unwrap_validation!( validate_content_link_base( &base_address, &create ) );

	    valid!()
	},
	LinkTypes::ContentUpdate => {
	    debug!("Checking LinkTypes::ContentUpdate");

	    unwrap_validation!( validate_content_link_base( &base_address, &create ) );

	    let tag_str = match String::from_utf8( tag.into_inner() ) {
		Ok(text) => text,
		Err(err) => invalid!(format!("Content update link tag must be a UTF8 string: {}", err )),
	    };

	    if !tag_str.contains(":") {
		invalid!(format!("Content update link has malformed tag: {}", tag_str ))
	    }

	    let (tag_id, tag_rev) = match tag_str.split_once(":") {
		Some(parts) => parts,
		None => invalid!(format!("Content update link has malformed tag: {}", tag_str )),
	    };

	    let content_id = match ActionHash::try_from( tag_id.to_string() ) {
		Ok(addr) => addr,
		Err(err) => invalid!(format!("Invalid tag part 1: {}", err )),
	    };
	    let content_rev = match ActionHash::try_from( tag_rev.to_string() ) {
		Ok(addr) => addr,
		Err(err) => invalid!(format!("Invalid tag part 2: {}", err )),
	    };

	    if content_id != get_root_origin( &content_rev )?.0 {
		invalid!(format!("Tag parts do not match; Content update link tag ID is not the root of the tag revision: {}", tag_str ))
	    }

	    valid!()
	},
	LinkTypes::Group => {
	    debug!("Checking LinkTypes::Group");
	    // Group base should be an AgentPubKey
	    let agent_pubkey = match base_address.clone().into_agent_pub_key() {
		Some(hash) => hash,
		None => invalid!(format!("Group link base address must be an agent pubkey; not '{}'", base_address )),
	    };

	    if agent_pubkey != create.author {
		invalid!(format!("Creating a link based on an agent pubkey can only be made by the matching agent ({})", agent_pubkey ))
	    }

	    // Group target should be a GroupEntry
	    let group_action_hash = match target_address.to_owned().into_action_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Group link target address must be the action hash of a group entry creation; not '{}'", target_address )),
	    };
	    let _group : GroupEntry = match must_get_valid_record( group_action_hash )?.try_into() {
		Ok(entry) => entry,
		Err(_) => invalid!(format!("A group link target address must be a group entry")),
	    };

	    valid!()
	},
	LinkTypes::GroupAuth => {
	    debug!("Checking LinkTypes::GroupAuth base address: {}", base_address );
	    let group_action_hash = match base_address.to_owned().into_action_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Group auth link base address must be the action hash of a group entry creation; not '{}'", base_address )),
	    };
	    let group : GroupEntry = match must_get_valid_record( group_action_hash )?.try_into() {
		Ok(entry) => entry,
		Err(_) => invalid!(format!("A group auth link base address must be a group entry")),
	    };

	    if !group.admins.contains( &create.author ) {
		invalid!("The author of a group auth link must be an admin of the base group".to_string())
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
	LinkTypes::GroupAuthArchive => {
	    debug!("Checking LinkTypes::GroupAuthArchive");

	    let group_action_hash = match base_address.to_owned().into_action_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Group auth archive link base address must be the action hash of a group entry creation; not '{}'", base_address )),
	    };
	    let _group : GroupEntry = match must_get_valid_record( group_action_hash )?.try_into() {
		Ok(entry) => entry,
		Err(_) => invalid!(format!("A group auth archive link base address must be a group entry")),
	    };

	    valid!()
	},
    }
}
