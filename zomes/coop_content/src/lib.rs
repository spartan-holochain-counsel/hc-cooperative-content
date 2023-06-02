mod validation;

use hdi::prelude::*;
// use serde::de::{ Deserializer, Error };

pub use coop_content_types::{
    PathEntry,
    GroupEntry,
    CommonFields,
};



pub trait ScopedTypeConnector<T>
where
    ScopedEntryDefIndex: for<'a> TryFrom<&'a T, Error = WasmError>,
{
    fn unit() -> EntryTypesUnit;
    fn app_entry_def() -> AppEntryDef;
    fn check_record_entry_type(record: &Record) -> bool;
    fn try_from_record(record: &Record) -> Result<Self, Self::Error>
    where
	Self: TryFrom<Record>;
    fn to_input(&self) -> T;
}


#[macro_export]
macro_rules! scoped_type_connector {
    ($units:ident::$unit_name:ident, $types:ident::$name:ident( $entry:ident ) ) => {
	scoped_type_connector!( $units::$unit_name, $types::$name, $entry );
    };
    ($units:ident::$unit_name:ident, $types:ident::$name:ident, $entry:ident ) => {
	impl ScopedTypeConnector<$types> for $entry {

	    fn unit() -> $units {
		$units::$unit_name
	    }

	    fn app_entry_def () -> AppEntryDef {
		// We know this is always defined because the hdi macros (hdk_entry_defs, unit_enum)
		// ensure that there will be a corresponding entry type for each unit.
		AppEntryDef::try_from( Self::unit() ).unwrap()
	    }

	    fn check_record_entry_type (record: &Record) -> bool {
		if let Action::Create(Create { entry_type: EntryType::App(aed), .. }) = record.action() {
		    return Self::app_entry_def() == *aed;
		}
		false
	    }

	    /// This "try from" checks the record's `EntryType` to make sure it matches the expected
	    /// `AppEntryDef` and then uses the official `TryFrom<Record>`.
	    fn try_from_record (record: &Record) -> Result<Self, WasmError> {
		// TODO: handle Action::Update too
		// match EntryCreationAction::try_from( record.action().to_owned() )
		// 	.map_err(|_| wasm_error!(WasmErrorInner::Guest(format!("ID does not belong to a Creation Action"))))?
		// {
		if let Action::Create(Create { entry_type: EntryType::App(aed), .. }) = record.action() {
		    if Self::app_entry_def() == *aed {
			Ok( record.to_owned().try_into()? )
		    } else {
			Err(wasm_error!(WasmErrorInner::Guest(
			    format!("Entry def mismatch: {:?} != {:?}", Self::app_entry_def(), aed )
			)))
		    }
		} else {
		    Err(wasm_error!(WasmErrorInner::Guest(
			format!("Action type ({}) does not contain an entry", ActionType::from(record.action()) )
		    )))
		}
	    }

	    fn to_input(&self) -> $types {
		$types::$name(self.clone())
	    }
	}
    };
}




#[hdk_entry_defs]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    #[entry_def]
    Path(PathEntry),
    #[entry_def]
    Group(GroupEntry),
}

scoped_type_connector!( EntryTypesUnit::Path,	EntryTypes::Path( PathEntry ) );
scoped_type_connector!( EntryTypesUnit::Group,	EntryTypes::Group( GroupEntry ) );



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
