use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{
        bytes::Bytes,
        packed::Byte,
        prelude::{Entity, Unpack},
    },
    debug,
    high_level::{load_script, load_witness_args},
};
use mol::{NftTransactionVec, RawIssueTransaction};
use script_utils::{
    helpers::{
        DISTRIBUTE_TRANSACTION, EXTRACT_TRANSACTION, INSERT_TRANSACTION, ISSUE_TRANSACTION,
        TRANSFER_TRANSACTION, UPDATE_TRANSACTION,
    },
    issuer::Issuer,
    misc::SMT,
};

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
    if input_issuer.version != output_issuer.version {
        return Err(Error::ClassDataInvalid);
    }
    if output_issuer.set_count < input_issuer.set_count {
        return Err(Error::IssuerSetCountError);
    }
    if output_issuer.class_count < input_issuer.class_count {
        return Err(Error::IssuerClassCountError);
    }

    let witness_args = load_witness_args(0, Source::GroupInput)?;
    let lock_type = witness_args.lock();
    //得到交易
    let txs = if let Some(lock_type) = lock_type.to_opt() {
        let lock_type: Vec<u8> = lock_type.unpack();
        NftTransactionVec::from_compatible_slice(&lock_type).unwrap()
    } else {
        return Err(Error::ItemMissing);
    };

    // let mut old_keys = Vec::new();
    // let mut new_keys = Vec::new();

    for tx in txs.into_iter() {
        match <Byte as Into<u8>>::into(tx.typ()) {
            ISSUE_TRANSACTION => {
                let _issue_tx =
                    RawIssueTransaction::from_compatible_slice(tx.transaction().as_slice())
                        .unwrap();
            }
            DISTRIBUTE_TRANSACTION => {}
            TRANSFER_TRANSACTION => {}
            UPDATE_TRANSACTION => {}
            EXTRACT_TRANSACTION => {}
            INSERT_TRANSACTION => {}
            _ => return Err(Error::NFTDataInvalid),
        }
    }

    Ok(())
}

pub fn handle_destroying_issuer(input_issuer: Issuer) -> Result<(), Error> {
    if input_issuer.class_count != 0 || input_issuer.set_count != 0 {
        return Err(Error::IssuerCellCannotDestroyed);
    }
    Ok(())
}
