use hdi::prelude::*;
// use hdi_extensions::{
//     trace_origin,
// };
use crate::{
    // Macros
    valid, invalid,

    // CommonFields,
    EntryTypes,
    // LinkTypes,
    GroupEntry,
};

pub fn validation(
    app_entry: EntryTypes,
    update: Update,
    _original_action_hash: ActionHash,
    original_entry_hash: EntryHash
) -> ExternResult<ValidateCallbackResult> {
    match app_entry {
	EntryTypes::Group(group) => {
	    let prev_group : GroupEntry = must_get_entry( original_entry_hash )?.content.try_into()?;

	    if !prev_group.admins.contains( &update.author ) {
		invalid!("Updating a group can only be done by an admin".to_string())
	    }

	    if group.admins != prev_group.admins {
		invalid!("Changing a group's admin list requires counter-signing".to_string())
	    }

	    valid!()
	},
	_ => invalid!(format!("Update validation not implemented for entry type: {:#?}", update.entry_type )),
    }
}
