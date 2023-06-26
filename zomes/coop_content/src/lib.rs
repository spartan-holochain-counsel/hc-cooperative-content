mod validation;

use serde::{
    Deserialize, Deserializer,
};
use hdi::prelude::*;
use hdi_extensions::{
    guest_error,
    ScopedTypeConnector, scoped_type_connector,
};

pub use coop_content_types::*;



#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def]
    Group(GroupEntry),

    // Anchors
    #[entry_def]
    GroupAuthAnchor(GroupAuthAnchorEntry),

    #[entry_def]
    GroupAuthArchiveAnchor(GroupAuthArchiveAnchorEntry),
}

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

impl TryFrom<String> for LinkTypes {
    type Error = WasmError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
	Ok(
	    match name.as_str() {
		"Group" => LinkTypes::Group,
		"GroupAuth" => LinkTypes::GroupAuth,
		"GroupAuthArchive" => LinkTypes::GroupAuthArchive,
		"Content" => LinkTypes::Content,
		"ContentUpdate" => LinkTypes::ContentUpdate,
		_ => return Err(guest_error!(format!("Unknown LinkTypes variant: {}", name ))),
	    }
	)
    }
}

impl<'de> Deserialize<'de> for LinkTypes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
	D: Deserializer<'de>,
    {
	let s: String = Deserialize::deserialize(deserializer)?;
	Ok(
	    LinkTypes::try_from( s.clone() )
		.or(Err(serde::de::Error::custom(format!("Unknown LinkTypes variant: {}", s))))?
	)
    }
}
