use crate::hdk::prelude::{
    hdi,
    debug,
};
use crate::hdi::prelude::*;
use crate::hdi_extensions::{
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
            debug!("Checking EntryTypes::Group({:#?})", group );
            if !group.admins.contains( &create.author ) {
                invalid!("The author of a group entry must be an admin of the group".to_string())
            }

            valid!()
        },
        EntryTypes::GroupAuthAnchor(anchor) => {
            debug!("Checking EntryTypes::GroupAuthAnchor({:#?})", anchor );
            valid!()
        },
        EntryTypes::GroupAuthArchiveAnchor(anchor) => {
            debug!("Checking EntryTypes::GroupAuthArchiveAnchor({:#?})", anchor );
            valid!()
        },
        // _ => invalid!(format!("Create validation not implemented for entry type: {:#?}", create.entry_type )),
    }
}
