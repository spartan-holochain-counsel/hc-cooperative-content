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
    GroupAuthAnchorEntry,
    GroupAuthArchiveAnchorEntry,
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
    GroupAuthAnchor(GroupAuthAnchorEntry),

    #[entry_def]
    GroupAuthArchiveAnchor(GroupAuthArchiveAnchorEntry),
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
    EntryTypesUnit::GroupAuthAnchor,
    EntryTypes::GroupAuthAnchor( GroupAuthAnchorEntry )
);
scoped_type_connector!(
    EntryTypesUnit::GroupAuthArchiveAnchor,
    EntryTypes::GroupAuthArchiveAnchor( GroupAuthArchiveAnchorEntry )
);



#[hdk_link_types]
pub enum LinkTypes {
    Group,
    GroupAuth,
    GroupAuthArchive,
    Content,
    ContentUpdate,
}
