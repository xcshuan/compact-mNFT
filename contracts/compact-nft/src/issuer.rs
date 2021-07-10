use ckb_std::{
    ckb_types::{bytes::Bytes, prelude::Unpack},
    debug,
    high_level::load_script,
};
use script_utils::{issuer::Issuer, misc::SMT};

use crate::{
    error::Error,
    type_id::{check_type_id, TYPE_ID_SIZE},
};

pub fn handle_creation_issuer(issuer: Issuer) -> Result<(), Error> {
    // check type_id
    {
        let script = load_script()?;
        let args: Bytes = Unpack::unpack(&script.args());

        debug!("script args is {:?}", args);
        if args.len() < TYPE_ID_SIZE {
            return Err(Error::TypeArgsInvalid);
        }
        let mut type_id = [0u8; TYPE_ID_SIZE];
        type_id.copy_from_slice(&args[..TYPE_ID_SIZE]);
        check_type_id(type_id)?;
    }

    if issuer.class_count != 0 {
        return Err(Error::IssuerClassCountError);
    }
    if issuer.set_count != 0 {
        return Err(Error::IssuerSetCountError);
    }

    // check SMT is empty
    let smt = SMT::default();
    if !issuer.smt_root.eq(smt.root().as_slice()) {
        return Err(Error::IssuerDataInvalid);
    }

    Ok(())
}

pub fn handle_update_issuer(input_issuer: Issuer, output_issuer: Issuer) -> Result<(), Error> {
    Ok(())
}

pub fn handle_destroying_issuer(input_issuer: Issuer) -> Result<(), Error> {
    if input_issuer.class_count != 0 || input_issuer.set_count != 0 {
        return Err(Error::IssuerCellCannotDestroyed);
    }
    Ok(())
}
