
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
macro_rules! guest_error {
    ( $message:expr ) => {
	wasm_error!(WasmErrorInner::Guest( $message ))
    };
}
