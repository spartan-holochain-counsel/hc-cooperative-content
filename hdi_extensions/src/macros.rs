
#[macro_export]
macro_rules! valid {
    () => {
	return Ok(ValidateCallbackResult::Valid)
    };
}

#[macro_export]
macro_rules! invalid {
    ( $message:expr ) => {
	return Ok(ValidateCallbackResult::Invalid($message))
    };
}

#[macro_export]
macro_rules! unwrap_validation {
    ( $($code:tt)+ ) => {
	match $($code)+ {
	    Ok(ValidateCallbackResult::Invalid(msg)) => return Ok(ValidateCallbackResult::Invalid(msg)),
	    Err(err) => return Err(err),
	    _ => (),
	}
    };
}
