use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, ckb_types::{packed::Byte, prelude::{Entity, Unpack}}, high_level::load_witness_args};
use mol::NftTransactionVec;
use script_utils::{
    class::Class,
    error::Error,
    helpers::{
        DISTRIBUTE_TRANSACTION, EXTRACT_TRANSACTION, INSERT_TRANSACTION,
        TRANSFER_TRANSACTION, UPDATE_TRANSACTION,
    },
};

pub fn handle_destroying_class(input_class: Class) -> Result<(), Error> {
    if input_class.issued > 0 {
        return Err(Error::ClassCellCannotDestroyed);
    }
    Ok(())
}

pub fn handle_update_class(input_class: Class, output_class: Class) -> Result<(), Error> {
    if output_class.issued < input_class.issued {
        return Err(Error::ClassIssuedInvalid);
    }

    if !input_class.immutable_equal(&output_class) {
        return Err(Error::ClassImmutableFieldsNotSame);
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

    // class_cell内不存在Issue交易
    for tx in txs.into_iter() {
        match <Byte as Into<u8>>::into(tx.typ()) {
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
