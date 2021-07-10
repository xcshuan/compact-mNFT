use script_utils::{class::Class, error::Error};

pub fn handle_destroying_class(input_class: Class) -> Result<(), Error> {
    if input_class.issued > 0 {
        return Err(Error::ClassCellCannotDestroyed);
    }
    Ok(())
}

pub fn handle_update_class(input_class: Class, output_class: Class) -> Result<(), Error> {
    Ok(())
}
