use hdi::prelude::*;
use hdi_extensions::{
    // Macros
    valid, invalid,
};
use crate::{
    EntryTypes,
};

pub fn validation(
    app_entry: EntryTypes,
    create: Create
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
	EntryTypes::Group(group) => {
	    if !group.admins.contains( &create.author ) {
		invalid!("The author of a group entry must be an admin of the group".to_string())
	    }

	    valid!()
	},
	EntryTypes::GroupAuthAnchor(_anchor) => {
	    valid!()
	},
	_ => invalid!(format!("Create validation not implemented for entry type: {:#?}", create.entry_type )),
    }
}
