use hdi::prelude::*;
use hdk::prelude::debug;
use hdi_extensions::{
    summon_app_entry,
    // Macros
    valid, invalid,
};
use crate::{
    LinkTypes,
    GroupEntry,
    GroupAuthAnchor,
};


pub fn validation(
    original_action_hash: ActionHash,
    base_address: AnyLinkableHash,
    delete: DeleteLink,
) -> ExternResult<ValidateCallbackResult> {
    let record = must_get_valid_record( original_action_hash )?;
    let create_link = match record.action() {
        Action::CreateLink(action) => action,
        _ => invalid!(format!("Original action hash does not belong to create link action")),
    };
    let link_type = match LinkTypes::from_type( create_link.zome_index, create_link.link_type )? {
        Some(lt) => lt,
        None => invalid!(format!("No match for LinkTypes")),
    };

    match link_type {
        LinkTypes::Content | LinkTypes::ContentUpdate => {
            debug!("Checking LinkTypes::Content[Update] delete");
            // Deletion is valid when
            // - the base is an archive anchor, if the author is an admin
            // - the base is an auth anchor, if the author is the matching anchor agent
            let anchor : GroupAuthAnchor = summon_app_entry( &base_address )?;

            debug!("Base address anchor: {:#?}", anchor );
            match anchor {
                GroupAuthAnchor::Active(entry) => {
                    if entry.1 != delete.author {
                        invalid!(format!("A content [update] link based on a group auth anchor can only be deleted by the matching anchor agent ({})", entry.1 ))
                    }
                },
                GroupAuthAnchor::Archive(entry) => {
                    let group : GroupEntry = must_get_valid_record( entry.0.clone() )?.try_into()?;

                    debug!("Archive anchor group revision: {:#?}", group );
                    if !group.authorities().contains( &delete.author )  {
                        invalid!(format!("A content [update] link based on a group auth archive anchor can only be deleted by an admin in the anchor's group revision ({})", entry.0 ))
                    }
                },
            };

            valid!()
        },
        LinkTypes::Group => {
            debug!("Checking LinkTypes::Group delete");
            // These can be deleted by the original author of the link
            if create_link.author != delete.author {
                invalid!(format!("A group link can only be deleted by the author who created it ({})", create_link.author ))
            }

            valid!()
        },
        LinkTypes::GroupAuth | LinkTypes::GroupAuthArchive => {
            debug!("Checking LinkTypes::GroupAuth[Archive] delete");
            // Never allowed because the way to remove members is by updating the group.  Once a
            // GroupAuth link is successfully made, it must be valid forever.
            invalid!(format!("Once created, group auth anchor links cannot be deleted"))
        },
    }
}
