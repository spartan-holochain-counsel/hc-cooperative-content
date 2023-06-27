use hdi::prelude::*;
use holo_hash::AnyLinkableHashPrimitive;
use hdk::prelude::debug;
use hdi_extensions::{
    get_root_origin,
    must_get_any_linkable_entry,
    any_linkable_deserialize_check,
    AnyLinkableHashTransformer,
    // Macros
    valid, invalid, guest_error,
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
) -> ExternResult<()> {
    let anchor : GroupAuthAnchor = must_get_any_linkable_entry( &base )?;

    if anchor.is_archive() {
        let group : GroupEntry = must_get_valid_record( anchor.group().to_owned() )?.try_into()?;
        if !group.admins.contains( &create.author ) {
            Err(guest_error!(format!("Creating a link based on an auth archive anchor can only be made by group admins")))?
        }
    } else if anchor.author() != &create.author {
        Err(guest_error!(format!("Creating a link based on an auth anchor can only be made by the matching agent ({})", anchor.author() )))?
    }

    Ok(())
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

            validate_content_link_base( &base_address, &create )?;

            valid!()
        },
        LinkTypes::ContentUpdate => {
            debug!("Checking LinkTypes::ContentUpdate");

            validate_content_link_base( &base_address, &create )?;

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

            let content_id = match AnyLinkableHash::try_from_string( tag_id ) {
                Ok(addr) => addr,
                Err(err) => invalid!(format!("Invalid tag part 1: {}", err )),
            };
            let content_rev = match AnyLinkableHash::try_from_string( tag_rev ) {
                Ok(addr) => addr,
                Err(err) => invalid!(format!("Invalid tag part 2: {}", err )),
            };

            // Is this check necessary?  Can't we just let group authorities define any pointers
            // that they want?
            if let (
                AnyLinkableHashPrimitive::Action(id_addr),
                AnyLinkableHashPrimitive::Action(rev_addr)
            ) = (content_id.into_primitive(), content_rev.into_primitive()) {
                if id_addr != get_root_origin( &rev_addr )?.0 {
                    invalid!(format!("Tag parts do not match; Content update link tag ID is not the root of the tag revision: {}", tag_str ))
                }
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
            any_linkable_deserialize_check::<GroupEntry>( &target_address )?;

            valid!()
        },
        LinkTypes::GroupAuth => {
            debug!("Checking LinkTypes::GroupAuth base address: {}", base_address );
            let group : GroupEntry = must_get_any_linkable_entry( &base_address )?;

            if !group.admins.contains( &create.author ) {
                invalid!("The author of a group auth link must be an admin of the base group".to_string())
            }

            let anchor : GroupAuthAnchorEntry = must_get_any_linkable_entry( &target_address )?;

            if !group.authorities().contains( &anchor.1 ) {
                invalid!(format!("Links to group auth anchors must match an authority in the group revision they are based off of"))
            }

            valid!()
        },
        LinkTypes::GroupAuthArchive => {
            debug!("Checking LinkTypes::GroupAuthArchive");

            any_linkable_deserialize_check::<GroupEntry>( &base_address )?;

            valid!()
        },
    }
}
