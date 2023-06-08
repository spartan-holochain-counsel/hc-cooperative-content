
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
