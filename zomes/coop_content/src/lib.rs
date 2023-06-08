mod validation;

use hdi::prelude::*;
use hdi_extensions::{
    ScopedTypeConnector, scoped_type_connector,
};

pub use coop_content_types::{
    // Content entry types
    GroupEntry,

    // Anchor entry types
    PathEntry,
    GroupMemberAnchorEntry,
    GroupMemberArchiveAnchorEntry,
};



#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def]
    Group(GroupEntry),

    // Anchors
    #[entry_def]
    Path(PathEntry),

    #[entry_def]
    GroupMemberAnchor(GroupMemberAnchorEntry),

    #[entry_def]
    GroupMemberArchiveAnchor(GroupMemberArchiveAnchorEntry),
}

scoped_type_connector!(
    EntryTypesUnit::Path,
    EntryTypes::Path( PathEntry )
);
scoped_type_connector!(
    EntryTypesUnit::Group,
    EntryTypes::Group( GroupEntry )
);
scoped_type_connector!(
    EntryTypesUnit::GroupMemberAnchor,
    EntryTypes::GroupMemberAnchor( GroupMemberAnchorEntry )
);
scoped_type_connector!(
    EntryTypesUnit::GroupMemberArchiveAnchor,
    EntryTypes::GroupMemberArchiveAnchor( GroupMemberArchiveAnchorEntry )
);



#[hdk_link_types]
pub enum LinkTypes {
    Group,
    GroupMember,
    GroupMemberArchive,
    Content,
    ContentUpdate,
}
