pub use coop_content_sdk::hdi;
pub use coop_content_sdk::hdi_extensions;

use hdi::prelude::*;
use coop_content_sdk::group_ref;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupRef {
    id: ActionHash,
    rev: ActionHash,
}


//
// Content Entry
//
#[hdk_entry_helper]
#[derive(Clone)]
pub struct ContentEntry {
    pub text: String,
    pub author: AgentPubKey,
    pub group_ref: GroupRef,

    // common fields
    pub published_at: u64,
    pub last_updated: u64,
}
group_ref!( ContentEntry, group_ref.id, group_ref.rev );
