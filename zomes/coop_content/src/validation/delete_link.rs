use crate::{
    hdi,
    hdi_extensions,
    LinkTypes,
    GroupEntry,
    ContributionAnchors,
};
use hdi::prelude::*;
use hdi_extensions::{
    summon_app_entry,
    // AnyLinkableHashTransformer,
    // Macros
    valid, invalid,
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
        LinkTypes::Contribution | LinkTypes::ContributionUpdate => {
            // Deletion is valid when
            // - the base is an archive anchor, if the author is an admin
            // - the base is an auth anchor, if the author is the matching anchor agent
            let anchor : ContributionAnchors = summon_app_entry( &base_address )?;

            match anchor {
                ContributionAnchors::Active(entry) => {
                    if *entry.author() != delete.author {
                        invalid!(format!("A content [update] link based on a contributions anchor can only be deleted by the matching anchor agent ({})", entry.author() ))
                    }
                },
                ContributionAnchors::Archive(entry) => {
                    let group : GroupEntry = must_get_valid_record( entry.group().to_owned() )?.try_into()?;

                    if !group.contributors().contains( &delete.author )  {
                        invalid!(format!("A content [update] link based on an archived contributions anchor can only be deleted by an admin in the anchor's group revision ({})", entry.group() ))
                    }
                },
            };

            valid!()
        },
        LinkTypes::GroupInvite => {
            valid!()
        },
        LinkTypes::Group => {
            // These can be deleted by the original author of the link
            if create_link.author != delete.author {
                invalid!(format!("A group link can only be deleted by the author who created it ({})", create_link.author ))
            }

            valid!()
        },
        LinkTypes::GroupAuth | LinkTypes::GroupAuthArchive => {
            // Never allowed because the way to remove members is by updating the group.  Once a
            // GroupAuth link is successfully made, it must be valid forever.
            invalid!(format!("Once created, group auth links cannot be deleted"))
        },
    }
}
