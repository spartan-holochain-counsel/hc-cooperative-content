mod validation;

pub use coop_content_sdk;
pub use coop_content_sdk::hdi;
pub use coop_content_sdk::hdk;
pub use coop_content_sdk::hdi_extensions;
pub use coop_content_sdk::hdk_extensions;
pub use coop_content_sdk::holo_hash;

use serde::{
    Deserialize, Deserializer,
};
use coop_content_sdk::*;
use hdi::prelude::*;
use hdi_extensions::{
    guest_error,
    ScopedTypeConnector, scoped_type_connector,
};




/// The entry types defined for this `coop_content` integrity zome
#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def]
    Group(GroupEntry),

    // Anchors
    #[entry_def]
    ContributionsAnchor(ContributionsAnchorEntry),

    #[entry_def]
    ArchivedContributionsAnchor(ArchivedContributionsAnchorEntry),
}

scoped_type_connector!(
    EntryTypesUnit::Group,
    EntryTypes::Group( GroupEntry )
);
scoped_type_connector!(
    EntryTypesUnit::ContributionsAnchor,
    EntryTypes::ContributionsAnchor( ContributionsAnchorEntry )
);
scoped_type_connector!(
    EntryTypesUnit::ArchivedContributionsAnchor,
    EntryTypes::ArchivedContributionsAnchor( ArchivedContributionsAnchorEntry )
);



/// The link types defined for this `coop_content` integrity zome
#[hdk_link_types]
pub enum LinkTypes {
    Group,
    GroupAuth,
    GroupAuthArchive,
    Contribution,
    ContributionUpdate,
}

impl TryFrom<String> for LinkTypes {
    type Error = WasmError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Ok(
            match name.as_str() {
                "Group" => LinkTypes::Group,
                "GroupAuth" => LinkTypes::GroupAuth,
                "GroupAuthArchive" => LinkTypes::GroupAuthArchive,
                "Contribution" => LinkTypes::Contribution,
                "ContributionUpdate" => LinkTypes::ContributionUpdate,
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
