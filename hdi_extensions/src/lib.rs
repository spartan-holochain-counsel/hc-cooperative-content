mod macros;

use hdi::prelude::*;
use holo_hash::AnyLinkableHashPrimitive;



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
pub trait AnyLinkableHashTransformer {
    fn must_be_action_hash(&self) -> ExternResult<ActionHash>;
    fn must_be_entry_hash(&self) -> ExternResult<EntryHash>;
}

impl AnyLinkableHashTransformer for AnyLinkableHash {
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



//
// Standard Inputs
//
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UpdateInput<T> {
    pub base: ActionHash,
    pub entry: T,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LinkBaseTargetInput {
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetLinksInputBuffer {
    pub base: AnyLinkableHash,
    pub target: AnyLinkableHash,
    pub link_type: String,
    pub tag: Option<String>,
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
