use alloc::vec::Vec;
use ckb_std::{ckb_constants::Source, high_level::load_cell_data};
use script_utils::{
    nft::Nft,
};

use crate::validator::{validate_immutable_nft_fields, validate_nft_claim, validate_nft_ext_info, validate_nft_lock, validate_nft_transfer};
use crate::error::Error;

fn load_nft_data(source: Source) -> Result<Vec<u8>, Error> {
    load_cell_data(0, source).map_err(|_| Error::NFTDataInvalid)
}

pub fn handle_update_nft() -> Result<(), Error> {
    let nft_data = (
        load_nft_data(Source::GroupInput)?,
        load_nft_data(Source::GroupOutput)?,
    );
    let nfts = (
        Nft::from_data(&nft_data.0[..], true)?,
        Nft::from_data(&nft_data.1[..], true)?,
    );
    validate_immutable_nft_fields(&nfts)?;
    validate_nft_claim(&nfts)?;
    validate_nft_lock(&nfts)?;
    validate_nft_transfer(&nfts.0, &nfts.1)?;
    validate_nft_ext_info(&nfts.0, &nft_data)?;
    Ok(())
}
