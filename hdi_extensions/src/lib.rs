mod macros;

use core::convert::{ TryFrom, TryInto };
use hdi::prelude::{
    must_get_valid_record, must_get_entry,
    wasm_error,
    ExternResult, WasmError, WasmErrorInner,
    Deserialize, Serialize, SerializedBytesError,
    ActionHash, EntryHash, ExternalHash, AnyDhtHash, AnyLinkableHash,
    Record, Action, Entry, Create, EntryCreationAction,
    AppEntryDef, ScopedEntryDefIndex,
    EntryType, EntryTypesHelper,
    LinkTypeFilter, LinkTypeFilterExt, LinkTag,
};
use hdi::prelude::holo_hash::AnyLinkableHashPrimitive;



//
// Custom Error Handling
//
#[derive(Debug)]
pub enum HdiExtError<'a> {
    ExpectedRecordNotEntry(&'a ActionHash),
}

impl<'a> From<HdiExtError<'a>> for WasmError {
    fn from(error: HdiExtError) -> Self {
        guest_error!(format!("{:?}", error ))
    }
}

pub fn convert_deserialize_error(error: WasmError) -> WasmError {
    match error {
	WasmError { error: WasmErrorInner::Serialize(SerializedBytesError::Deserialize(msg)), .. } =>
	    guest_error!(
		format!("Could not deserialize any-linkable address to expected type: {}", msg )
	    ),
	err => err,
    }
}


//
// Tracing Actions
//
pub fn trace_origin(action_address: &ActionHash) -> ExternResult<Vec<(ActionHash, Action)>> {
    let mut history = vec![];
    let mut next_addr = Some(action_address.to_owned());

    while let Some(addr) = next_addr {
	let record = must_get_valid_record( addr )?;

	next_addr = match record.action() {
	    Action::Update(update) => Some(update.original_action_address.to_owned()),
	    Action::Create(_) => None,
	    _ => return Err(guest_error!(format!("Wrong action type '{}'", record.action().action_type() )))?,
	};

	history.push( (record.signed_action.hashed.hash, record.signed_action.hashed.content) );
    }

    Ok( history )
}

pub fn get_root_origin(action_address: &ActionHash) -> ExternResult<(ActionHash, Action)> {
    Ok( trace_origin( action_address )?.last().unwrap().to_owned() )
}


//
// Entry Struct
//
pub trait ScopedTypeConnector<T,U>
where
    ScopedEntryDefIndex: for<'a> TryFrom<&'a T, Error = WasmError>,
{
    fn unit() -> U;
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
	impl ScopedTypeConnector<$types,$units> for $entry {

	    fn unit() -> $units {
		$units::$unit_name
	    }

	    fn app_entry_def () -> AppEntryDef {
		// We know this is always defined because the hdi macros (hdk_entry_defs, unit_enum)
		// ensure that there will be a corresponding entry type for each unit.
		AppEntryDef::try_from( Self::unit() ).unwrap()
	    }

	    fn check_record_entry_type (record: &Record) -> bool {
		match EntryCreationAction::try_from( record.action().to_owned() ) {
		    Ok(creation_action) => match creation_action.entry_type() {
			EntryType::App(aed) => Self::app_entry_def() == *aed,
			_ => false,
		    },
		    _ => false,
		}
	    }

	    /// This "try from" checks the record's `EntryType` to make sure it matches the expected
	    /// `AppEntryDef` and then uses the official `TryFrom<Record>`.
	    fn try_from_record (record: &Record) -> Result<Self, WasmError> {
		let creation_action = EntryCreationAction::try_from( record.action().to_owned() )
		    .map_err(|_| hdi_extensions::guest_error!(
			format!("ID does not belong to a Creation Action")
		    ))?;

		if let EntryType::App(aed) = creation_action.entry_type() {
		    if Self::app_entry_def() == *aed {
			Ok( record.to_owned().try_into()? )
		    } else {
			Err(hdi_extensions::guest_error!(
			    format!("Entry def mismatch: {:?} != {:?}", Self::app_entry_def(), aed )
			))
		    }
		} else {
		    Err(hdi_extensions::guest_error!(
			format!("Action type ({}) does not contain an entry", ActionType::from(record.action()) )
		    ))
		}
	    }

	    fn to_input(&self) -> $types {
		$types::$name(self.clone())
	    }
	}
    };
}


//
// HoloHash Extentions
//
pub trait AnyLinkableHashTransformer : Sized {
    fn try_from_string(input: &str) -> ExternResult<Self>;
    fn must_be_action_hash(&self) -> ExternResult<ActionHash>;
    fn must_be_entry_hash(&self) -> ExternResult<EntryHash>;
}

impl AnyLinkableHashTransformer for AnyLinkableHash {
    fn try_from_string(input: &str) -> ExternResult<Self> {
	let action_result = ActionHash::try_from( input.to_string() );
	let entry_result = EntryHash::try_from( input.to_string() );
	let external_result = ExternalHash::try_from( input.to_string() );

	Ok(
	    match (action_result.is_ok(), entry_result.is_ok(), external_result.is_ok()) {
		(true, false, false) => action_result.unwrap().into(),
		(false, true, false) => entry_result.unwrap().into(),
		(false, false, true) => external_result.unwrap().into(),
		(false, false, false) => Err(guest_error!(
		    format!("String '{}' must be an Action or Entry hash", input )
		))?,
		_ => Err(guest_error!(
		    format!("String '{}' matched multiple hash types; this should not be possible", input )
		))?,
	    }
	)
    }

    fn must_be_action_hash(&self) -> ExternResult<ActionHash> {
	match self.to_owned().into_action_hash() {
	    Some(hash) => Ok( hash ),
	    None => Err(guest_error!(
		format!("Any-linkable hash must be an action hash; not '{}'", self )
	    ))?,
	}
    }

    fn must_be_entry_hash(&self) -> ExternResult<EntryHash> {
	match self.to_owned().into_entry_hash() {
	    Some(hash) => Ok( hash ),
	    None => Err(guest_error!(
		format!("Any-linkable hash must be an entry hash; not '{}'", self )
	    ))?,
	}
    }
}

pub trait AnyDhtHashTransformer : Sized {
    fn try_from_string(input: &str) -> ExternResult<Self>;
}

impl AnyDhtHashTransformer for AnyDhtHash {
    fn try_from_string(input: &str) -> ExternResult<Self> {
	let action_result = ActionHash::try_from( input.to_string() );
	let entry_result = EntryHash::try_from( input.to_string() );

	Ok(
	    match (action_result.is_ok(), entry_result.is_ok()) {
		(true, false) => action_result.unwrap().into(),
		(false, true) => entry_result.unwrap().into(),
		(false, false) => Err(guest_error!(
		    format!("String '{}' must be an Action or Entry hash", input )
		))?,
		(true, true) => Err(guest_error!(
		    format!("String '{}' matched Action and Entry hash; this should not be possible", input )
		))?,
	    }
	)
    }
}


//
// Advanced "get" Methods
//
pub fn must_get_any_linkable_entry<T,E>(addr: &AnyLinkableHash) -> ExternResult<T>
where
    T: TryFrom<Record, Error = E> + TryFrom<Entry, Error = E>,
    E: std::fmt::Debug,
    WasmError: From<E>,
{
    match addr.to_owned().into_primitive() {
	AnyLinkableHashPrimitive::Action(action_hash) => Ok(
	    must_get_valid_record( action_hash )?.try_into()
		.map_err(|error| convert_deserialize_error( WasmError::from(error) ) )?
	),
	AnyLinkableHashPrimitive::Entry(entry_hash) => Ok(
	    must_get_entry( entry_hash )?.content.try_into()
		.map_err(|error| convert_deserialize_error( WasmError::from(error) ) )?
	),
	AnyLinkableHashPrimitive::External(external_hash) => Err(guest_error!(
	    format!("Cannot get an entry from any-linkable external hash ({})", external_hash )
	))?,
    }
}

pub fn any_linkable_deserialize_check<T>(addr: &AnyLinkableHash) -> ExternResult<T>
where
    T: TryFrom<Record, Error = WasmError> + TryFrom<Entry, Error = WasmError>,
{
    must_get_any_linkable_entry( addr )
}

pub fn get_create_action(action_addr: &ActionHash) -> ExternResult<(Record, Create)> {
    let create_record = must_get_valid_record( action_addr.to_owned() )?;
    let create_action = match create_record.action().clone() {
	Action::Create(action) => action,
	_ => Err(guest_error!(format!("Action address ({}) is not a create action", action_addr )))?,
    };

    Ok( (create_record, create_action) )
}

pub fn get_creation_action(action_addr: &ActionHash) -> ExternResult<EntryCreationAction> {
    let create_record = must_get_valid_record( action_addr.to_owned() )?;
    match create_record.signed_action.hashed.content {
	Action::Create(create) => Ok( create.into() ),
	Action::Update(update) => Ok( update.into() ),
	_ => Err(guest_error!(format!("Action address ({}) is not a create action", action_addr ))),
    }
}

pub fn get_app_entry<ET,A>(action: &A) -> ExternResult<ET>
where
    ET: EntryTypesHelper,
    WasmError: From<<ET as EntryTypesHelper>::Error>,
    A: Into<EntryCreationAction> + Clone,
{
    let action : EntryCreationAction = action.to_owned().into();
    let entry_def = derive_app_entry_def( &action )?;
    let entry = must_get_entry( action.entry_hash().to_owned() )?.content;

    ET::deserialize_from_type(
	entry_def.zome_index.clone(),
	entry_def.entry_index.clone(),
	&entry,
    )?.ok_or(guest_error!(
	format!("No match for entry def ({:?}) in expected entry types", entry_def )
    ))
}


//
// EntryTypesHelper extensions
//
pub fn derive_app_entry_def<A>(action: &A) -> ExternResult<AppEntryDef>
where
    A: Into<EntryCreationAction> + Clone,
{
    let action : EntryCreationAction = action.to_owned().into();
    match action.entry_type().to_owned() {
	EntryType::App(app_entry_def) => Ok( app_entry_def ),
	entry_type => Err(guest_error!(
	    format!("Expected an app entry type; not {:?}", entry_type )
	)),
    }
}

pub fn derive_app_entry_unit<ETU,A>(action: &A) -> ExternResult<ETU>
where
    ETU: TryFrom<ScopedEntryDefIndex, Error = WasmError>,
    A: Into<EntryCreationAction> + Clone,
{
    let action : EntryCreationAction = action.to_owned().into();
    let entry_def = derive_app_entry_def( &action )?;
    ETU::try_from(ScopedEntryDefIndex {
	zome_index: entry_def.zome_index,
	zome_type: entry_def.entry_index,
    })
}


//
// Standard Inputs
//
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateEntryInput<T> {
    pub base: ActionHash,
    pub entry: T,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LinkBaseTargetInput {
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetLinksInputBuffer {
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
    pub link_type: String,
    pub tag: Option<String>,
}

#[derive(Clone, Serialize, Debug)]
pub struct GetLinksInput<T>
where
    T: LinkTypeFilterExt + TryFrom<String, Error = WasmError> + Clone,
{
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
    pub link_type_filter: LinkTypeFilter,
    pub tag: Option<LinkTag>,
    pub link_type: Option<T>,
}

impl<T> TryFrom<GetLinksInputBuffer> for GetLinksInput<T>
where
    T: LinkTypeFilterExt + TryFrom<String, Error = WasmError> + Clone,
{
    type Error = WasmError;

    fn try_from(buffer: GetLinksInputBuffer) -> Result<Self, Self::Error> {
	let (link_type, link_type_filter) = match buffer.link_type.as_str() {
	    ".." => ( None, (..).try_into_filter()? ),
	    name => {
		let link_type = T::try_from( name.to_string() )?;
		( Some(link_type.clone()), link_type.try_into_filter()? )
	    },
	};

	Ok(Self {
	    base: buffer.base,
	    target: buffer.target,
	    tag: buffer.tag.map(|text| text.into_bytes().into() ),
	    link_type,
	    link_type_filter,
	})
    }
}

impl<'de,T> serde::Deserialize<'de> for GetLinksInput<T>
where
    T: LinkTypeFilterExt + TryFrom<String, Error = WasmError> + Clone,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
	D: serde::Deserializer<'de>,
    {
	let buffer : GetLinksInputBuffer = Deserialize::deserialize(deserializer)?;
	let error_msg = format!("Buffer could be converted: {:#?}", buffer );

	Ok(
	    buffer.try_into()
		.or(Err(serde::de::Error::custom(error_msg)))?
	)
    }
}
