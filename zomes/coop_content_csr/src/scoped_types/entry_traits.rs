use hdk::prelude::*;

pub trait GroupLinks {
    fn group_auth_addrs(base: &ActionHash) -> ExternResult<Vec<EntryHash>>;
    fn group_auth_archive_addrs(base: &ActionHash) -> ExternResult<Vec<EntryHash>>;
}

pub trait GroupAuthLinks {
    fn base_address(&self) -> ExternResult<EntryHash>;
    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn update_links(&self) -> ExternResult<Vec<Link>>;
    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn shortcuts(&self) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash, AnyLinkableHash)>>;
}


pub trait GroupAuthArchiveLinks {
    fn base_address(&self) -> ExternResult<EntryHash>;
    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn update_links(&self) -> ExternResult<Vec<Link>>;
    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn shortcuts(&self) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash, AnyLinkableHash)>>;
}

