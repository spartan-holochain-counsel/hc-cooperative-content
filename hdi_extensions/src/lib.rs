use hdi::prelude::*;


//
// Custom Errors
//
#[derive(Debug)]
pub enum HdiExtError<'a> {
    ExpectedRecordNotEntry(&'a ActionHash),
}

impl<'a> From<HdiExtError<'a>> for WasmError {
    fn from(error: HdiExtError) -> Self {
        wasm_error!(WasmErrorInner::Guest( format!("{:?}", error ) ))
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
	    _ => return Err(wasm_error!(WasmErrorInner::Guest(format!("Wrong action type '{}'", record.action().action_type() ))))?,
	};

	history.push( (record.signed_action.hashed.hash, record.signed_action.hashed.content) );
    }

    Ok( history )
}

pub fn get_origin(action_address: &ActionHash) -> ExternResult<(ActionHash, Action)> {
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
		    .map_err(|_| wasm_error!(WasmErrorInner::Guest(
			format!("ID does not belong to a Creation Action")
		    )))?;

		if let EntryType::App(aed) = creation_action.entry_type() {
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
