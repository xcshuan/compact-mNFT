use crate::{error::Error, helpers::u32_from_slice, misc::new_blake2b};
use alloc::vec::Vec;
use core::result::Result;
use sparse_merkle_tree::{traits::Value, H256};

pub const NFT_DATA_MIN_LEN: usize = 75;

//in cell, owner will be in lock_script, issuer_id, class_id and token_id will be in type_args
pub const NFT_DATA_MIN_LEN_IN_CELL: usize = 15;
pub const NFT_TYPE_ARGS_LEN: usize = 28;

/// NFT cell data structure
/// This structure contains the following information:
/// 1) version: u8
/// 2) issuer_id: [u8; 20],
/// 3) class_id: u32,
/// 4) token_id: u32,
/// 5) characteristic: [u8; 8]
/// 6) configure: u8
/// 7) state: u8
/// 8) nonce: u32
/// 9) owner:[u8;32]
/// 10) extinfo_data: <size: u16> + <vartext>
/// The filed of 10) can be changed and it also can be missing and it will not be validated.
#[derive(Debug, Clone, Default)]
pub struct Nft {
    pub version: u8,
    pub issuer_id: [u8; 20],
    pub class_id: u32,
    pub token_id: u32,
    pub characteristic: [u8; 8],
    pub configure: u8,
    pub state: u8,
    pub nonce: u32,
    pub owner: [u8; 32],
}

impl Nft {
    pub fn from_data_single_cell(data: &[u8]) -> Result<Self, Error> {
        if data.len() < NFT_DATA_MIN_LEN_IN_CELL {
            return Err(Error::NFTDataInvalid);
        }

        let version: u8 = data[0];
        if version != 0 {
            return Err(Error::VersionInvalid);
        }

        let mut characteristic = [0u8; 8];
        characteristic.copy_from_slice(&data[1..9]);

        let configure: u8 = data[9];
        let state: u8 = data[10];

        let nonce = u32_from_slice(&data[11..15]);

        return Ok(Nft {
            version,
            issuer_id: [0u8; 20],
            class_id: 0,
            token_id: 0,
            characteristic,
            configure,
            state,
            nonce,
            owner: [0u8; 32],
        });
    }

    pub fn from_data_leaf(data: &[u8]) -> Result<Self, Error> {
        if data.len() < NFT_DATA_MIN_LEN {
            return Err(Error::NFTDataInvalid);
        }

        let version: u8 = data[0];
        if version != 0 {
            return Err(Error::VersionInvalid);
        }

        let mut issuer_id = [0u8; 20];
        issuer_id.copy_from_slice(&data[1..21]);

        let class_id = u32_from_slice(&data[21..25]);

        let token_id = u32_from_slice(&data[25..29]);

        let mut characteristic = [0u8; 8];
        characteristic.copy_from_slice(&data[29..37]);

        let configure: u8 = data[37];
        let state: u8 = data[38];

        let nonce = u32_from_slice(&data[39..43]);

        let mut owner = [0u8; 32];
        owner.copy_from_slice(&data[43..75]);

        return Ok(Nft {
            version,
            issuer_id,
            class_id,
            token_id,
            characteristic,
            configure,
            state,
            nonce,
            owner,
        });
    }

    pub fn from_data(data: &[u8], in_cell: bool) -> Result<Self, Error> {
        //single cell mode
        if in_cell {
            return Self::from_data_single_cell(data);
        // leaf mode
        } else {
            return Self::from_data_leaf(data);
        }
    }

    pub fn allow_claim(&self) -> bool {
        self.configure & 0b0000_0001 == 0b0000_0000
    }

    pub fn allow_lock(&self) -> bool {
        self.configure & 0b0000_0010 == 0b0000_0000
    }

    pub fn allow_ext_info(&self) -> bool {
        self.configure & 0b0000_0100 == 0b0000_0000
    }

    pub fn allow_transfer_before_claim(&self) -> bool {
        self.configure & 0b0001_0000 == 0b0000_0000
    }

    pub fn allow_transfer_after_claim(&self) -> bool {
        self.configure & 0b0010_0000 == 0b0000_0000
    }

    pub fn allow_destroying_before_claim(&self) -> bool {
        self.configure & 0b0100_0000 == 0b0000_0000
    }

    pub fn allow_destroying_after_claim(&self) -> bool {
        self.configure & 0b1000_0000 == 0b0000_0000
    }

    pub fn is_claimed(&self) -> bool {
        self.state & 0b0000_0001 == 0b0000_0001
    }

    pub fn is_locked(&self) -> bool {
        self.state & 0b0000_0010 == 0b0000_0010
    }

    pub fn immutable_equal(&self, other: &Nft) -> bool {
        self.issuer_id == other.issuer_id
            && self.class_id == self.class_id
            && self.token_id == other.token_id
            && self.configure == other.configure
            && self.characteristic == other.characteristic
    }
}

impl Nft {
    fn is_zero(&self) -> bool {
        return self.issuer_id == [0u8; 20] && self.owner == [0u8; 32];
    }

    fn to_leaf_data(&self) -> Vec<u8> {
        let mut leaf_data = Vec::with_capacity(NFT_DATA_MIN_LEN);
        leaf_data.extend_from_slice(&self.version.to_le_bytes());
        leaf_data.extend_from_slice(&self.issuer_id);
        leaf_data.extend_from_slice(&self.class_id.to_le_bytes());
        leaf_data.extend_from_slice(&self.token_id.to_le_bytes());
        leaf_data.extend_from_slice(&self.characteristic);
        leaf_data.extend_from_slice(&self.configure.to_le_bytes());
        leaf_data.extend_from_slice(&self.state.to_le_bytes());
        leaf_data.extend_from_slice(&self.nonce.to_le_bytes());
        leaf_data.extend_from_slice(&self.owner);
        leaf_data
    }

    pub fn to_key(&self) -> H256 {
        if self.is_zero() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.issuer_id);
        hasher.update(&self.class_id.to_le_bytes());
        hasher.update(&self.token_id.to_le_bytes());
        hasher.update(&self.characteristic);
        hasher.update(&self.configure.to_le_bytes());
        hasher.finalize(&mut buf);
        buf.into()
    }
}

impl Value for Nft {
    fn to_h256(&self) -> H256 {
        if self.is_zero() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.issuer_id);
        hasher.update(&self.class_id.to_le_bytes());
        hasher.update(&self.token_id.to_le_bytes());
        hasher.update(&self.characteristic);
        hasher.update(&self.configure.to_le_bytes());
        hasher.update(&self.state.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.owner);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}
