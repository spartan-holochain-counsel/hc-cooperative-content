use hdi::prelude::*;
// use crate::{
//     CommonFields,
//     GroupEntry,
//     // EntryTypes,
//     // LinkTypes,
// };


#[hdk_extern]
fn validate(_op: Op) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
