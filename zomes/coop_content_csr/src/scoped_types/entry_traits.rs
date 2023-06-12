use hdk::prelude::*;

pub trait GroupLinks {
    fn group_auth_addrs(base: &ActionHash) -> ExternResult<Vec<EntryHash>>;
    fn group_auth_archive_addrs(base: &ActionHash) -> ExternResult<Vec<EntryHash>>;
}

pub trait GroupAuthLinks {
    fn content_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
}


pub trait GroupAuthArchiveLinks {
    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
}

