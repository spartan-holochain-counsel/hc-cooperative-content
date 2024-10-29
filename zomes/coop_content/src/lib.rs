mod validation;

pub use coop_content_types;
pub use coop_content_types::hdi;
pub use coop_content_types::hdi_extensions;
pub use coop_content_types::*;

use serde::{
    Deserialize, Deserializer,
};
use hdi::prelude::*;
use hdi_extensions::{
    guest_error,
    ScopedTypeConnector, scoped_type_connector,
};




/// The entry types defined for this `coop_content` integrity zome
#[hdk_entry_types]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_type]
    Group(GroupEntry),

    // Anchors
    #[entry_type]
    ContributionsAnchor(ContributionsAnchorEntry),

    #[entry_type]
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
    GroupInvite,
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
                "GroupInvite" => LinkTypes::GroupInvite,
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
