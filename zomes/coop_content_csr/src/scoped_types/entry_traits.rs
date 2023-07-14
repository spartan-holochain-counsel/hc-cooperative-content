use crate::hdk::prelude::*;

pub trait GroupLinks {
    fn group_auth_anchor_hashes(base: &ActionHash) -> ExternResult<Vec<EntryHash>>;
    fn group_auth_archive_anchor_hashes(base: &ActionHash) -> ExternResult<Vec<EntryHash>>;
}

pub trait ContributionsLinks {
    fn base_hash(&self) -> ExternResult<EntryHash>;
    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn update_links(&self) -> ExternResult<Vec<Link>>;
    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn shortcuts(&self) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash, AnyLinkableHash)>>;
}


pub trait ArchivedContributionsLinks {
    fn base_hash(&self) -> ExternResult<EntryHash>;
    fn create_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn update_links(&self) -> ExternResult<Vec<Link>>;
    fn update_targets(&self) -> ExternResult<Vec<AnyLinkableHash>>;
    fn shortcuts(&self) -> ExternResult<Vec<(AnyLinkableHash, AnyLinkableHash, AnyLinkableHash)>>;
}

