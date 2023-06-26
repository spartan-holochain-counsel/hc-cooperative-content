use hdi::prelude::*;
use hdk::prelude::debug;
use hdi_extensions::{
    get_create_action,
    derive_app_entry_unit,
    // Macros
    invalid,
};
use crate::{
    EntryTypesUnit,
};


pub fn validation(
    original_action_hash: ActionHash,
    _original_entry_hash: EntryHash,
    _delete: Delete
) -> ExternResult<ValidateCallbackResult> {
    let (_create_record, create) = get_create_action( &original_action_hash )?;

    match derive_app_entry_unit( &create )? {
	EntryTypesUnit::Group => {
	    debug!("Checking delete EntryTypesUnit::Group");
	    invalid!("Groups cannot be deleted; they can be marked as 'dead' using counter-signing".to_string())
	},
	EntryTypesUnit::GroupAuthAnchor => {
	    debug!("Checking delete of EntryTypesUnit::GroupAuthAnchor");
	    invalid!("Anchors are required for the continuity of group content evolution".to_string())
	},
	EntryTypesUnit::GroupAuthArchiveAnchor => {
	    debug!("Checking delete EntryTypesUnit::GroupAuthArchiveAnchor");
	    invalid!("Anchors are required for the continuity of group content evolution".to_string())
	},
	// entry_type_unit => invalid!(format!("Delete validation not implemented for entry type: {:?}", entry_type_unit )),
    }
}
