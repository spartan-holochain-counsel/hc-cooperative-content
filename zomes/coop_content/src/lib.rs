mod validation;

use hdi::prelude::*;
// use serde::de::{ Deserializer, Error };

pub use coop_content_types::{
    PathEntry,
    GroupEntry,
    CommonFields,
};



#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def]
    Path(PathEntry),
    #[entry_def]
    Group(GroupEntry),
}


#[hdk_link_types]
pub enum LinkTypes {
    Group,
    GroupMember,
    GroupMemberArchive,
    Subject,
    SubjectUpdate,
}

// impl<'de> Deserialize<'de> for LinkTypes {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
// 	D: Deserializer<'de>,
//     {
// 	let name : &str = Deserialize::deserialize(deserializer)?;
// 	match name {
// 	    "Group" => Ok(LinkTypes::Group),
// 	    "GroupMember" => Ok(LinkTypes::GroupMember),
// 	    "GroupMemberArchive" => Ok(LinkTypes::GroupMemberArchive),
// 	    "Subject" => Ok(LinkTypes::Subject),
// 	    "SubjectUpdate" => Ok(LinkTypes::SubjectUpdate),

// 	    value => Err(D::Error::custom(format!("No LinkTypes value matching '{}'", value ))),
// 	}
//     }
// }
