use crate::{
    hdi,
    hdi_extensions,
    EntryTypes,
    GroupEntry,
};
use hdi::prelude::*;
use hdi_extensions::{
    // Macros
    valid, invalid,
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

            if !prev_group.is_admin( &update.author ) {
                invalid!("Updating a group can only be done by an admin".to_string())
            }

            if group.admins.len() == 0 {
                invalid!("Admin list cannot be empty".to_string())
            }

            valid!()
        },
        _ => invalid!(format!("Update validation not implemented for entry type: {:#?}", update.entry_type )),
    }
}
