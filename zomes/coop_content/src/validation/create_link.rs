use hdi::prelude::*;
use hdi_extensions::{
    // Macros
    valid, invalid,
};
use crate::{
    // EntryTypes,
    LinkTypes,
    GroupAuthAnchorEntry
};

pub fn validation(
    base_address: AnyLinkableHash,
    _target_address: AnyLinkableHash,
    link_type: LinkTypes,
    _tag: LinkTag,
    create: CreateLink,
) -> ExternResult<ValidateCallbackResult> {
    match link_type {
	LinkTypes::Content => {
	    let entry_hash = match base_address.to_owned().into_entry_hash() {
		Some(hash) => hash,
		None => invalid!(format!("Content link base address must be an entry hash; not '{}'", base_address )),
	    };

	    let anchor : GroupAuthAnchorEntry = must_get_entry( entry_hash )?.content.try_into()?;
	    if anchor.1 != create.author {
		invalid!(format!("Creating a link based on an auth anchor can only be made by the matching agent ({})", anchor.1 ))
	    }

	    valid!()
	},
	LinkTypes::Group => {
	    valid!()
	},
	LinkTypes::GroupAuth => {
	    valid!()
	},
	_ => invalid!(format!("Create validation not implemented for link type: {:#?}", link_type )),
    }
}
