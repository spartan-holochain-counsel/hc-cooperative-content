use crate::hdk::prelude::{
    hdi, holo_hash,
    debug,
};
use crate::hdi::prelude::*;
use holo_hash::AnyLinkableHashPrimitive;
use crate::hdi_extensions::{
    trace_origin_root,
    summon_app_entry,
    verify_app_entry_struct,
    AnyLinkableHashTransformer,
    // Macros
    valid, invalid, guest_error,
};
use crate::{
    // EntryTypes,
    LinkTypes,
    GroupEntry,
    ContributionAnchors,
};


fn validate_content_link_base(
    base: &AnyLinkableHash,
    create: &CreateLink,
) -> ExternResult<()> {
    let anchor : ContributionAnchors = summon_app_entry( &base )?;

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

fn validate_anchor_link_base(
    base: &AnyLinkableHash,
    target: &AnyLinkableHash,
    create: &CreateLink,
) -> ExternResult<()> {
    let group : GroupEntry = summon_app_entry( base )?;

    if !group.is_admin( &create.author ) {
        Err(guest_error!("The author of a group auth link must be an admin of the base group".to_string()))?;
    }

    let anchor : ContributionAnchors = summon_app_entry( target )?;

    if !anchor.is_archive() && !group.is_contributor( anchor.author() ) {
        Err(guest_error!(format!("Links to a contributions anchor must match a contributor in the group base")))?;
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
        LinkTypes::Contribution => {
            debug!("Checking LinkTypes::Contribution base address: {}", base_address );

            validate_content_link_base( &base_address, &create )?;

            valid!()
        },
        LinkTypes::ContributionUpdate => {
            debug!("Checking LinkTypes::ContributionUpdate");

            validate_content_link_base( &base_address, &create )?;

            let tag_str = match String::from_utf8( tag.into_inner() ) {
                Ok(text) => text,
                Err(err) => invalid!(format!("Contribution update link tag must be a UTF8 string: {}", err )),
            };

            if !tag_str.contains(":") {
                invalid!(format!("Contribution update link has malformed tag: {}", tag_str ))
            }

            let (tag_id, tag_rev) = match tag_str.split_once(":") {
                Some(parts) => parts,
                None => invalid!(format!("Contribution update link has malformed tag: {}", tag_str )),
            };

            let content_id = match AnyLinkableHash::try_from_string( tag_id ) {
                Ok(addr) => addr,
                Err(err) => invalid!(format!("Invalid tag part 1: {}", err )),
            };
            let content_rev = match AnyLinkableHash::try_from_string( tag_rev ) {
                Ok(addr) => addr,
                Err(err) => invalid!(format!("Invalid tag part 2: {}", err )),
            };

            // Is this check necessary?  Can't we just let group contributors define any pointers
            // that they want?
            if let (
                AnyLinkableHashPrimitive::Action(id_addr),
                AnyLinkableHashPrimitive::Action(rev_addr)
            ) = (content_id.into_primitive(), content_rev.into_primitive()) {
                if id_addr != trace_origin_root( &rev_addr )?.0 {
                    invalid!(format!("Tag parts do not match; Contribution update link tag ID is not the root of the tag revision: {}", tag_str ))
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
            verify_app_entry_struct::<GroupEntry>( &target_address )?;

            valid!()
        },
        LinkTypes::GroupAuth => {
            debug!("Checking LinkTypes::GroupAuth base address: {}", base_address );

            validate_anchor_link_base( &base_address, &target_address, &create )?;

            valid!()
        },
        LinkTypes::GroupAuthArchive => {
            debug!("Checking LinkTypes::GroupAuthArchive");

            validate_anchor_link_base( &base_address, &target_address, &create )?;

            valid!()
        },
    }
}
