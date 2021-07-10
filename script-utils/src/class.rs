use crate::error::Error;
use crate::helpers::{parse_dyn_vec_len, u32_from_slice, DYN_MIN_LEN};
use crate::misc::new_blake2b;
use alloc::vec::Vec;
use core::result::Result;
use sparse_merkle_tree::{traits::Value, H256};


const FIXED_LEN: usize = 66;

const CLASS_DATA_MIN_LEN: usize = 72;

const FIXED_LEN_IN_CELL: usize = 10;
// in cell, owner will be in lock_script, issuer_id and class_id will be in type_args
// FIXED_LEN_IN_CELL + DYN_MIN_LEN * 3
const CLASS_DATA_MIN_LEN_IN_CELL: usize = 16;

pub const CLASS_TYPE_ARGS_LEN: usize = 24;

/// Class cell data structure
/// This structure contains the following information:
/// 1) version: u8
/// 2) issuer_id: [u8; 20],
/// 3) class_id: u32,
/// 4) total: u32
/// 5) issued: u32
/// 6) configure: u8
/// 7) owner: [u8; 32],
/// 8) name: <size: u16> + <content>
/// 9) description: <size: u16> + <content>
/// 10) renderer: <size: u16> + <content>
/// 11) extinfo_data: <size: u16> + <content>
/// The fields of 1), 2), 3), 4), 6), 8) and 9) cannot be changed after they are set and they cannot be
/// missing. The fields of 5) and 10) can be changed and it cannot be missing.
/// The filed of 11) can be changed and it also can be missing and it will not be validated.
#[derive(Debug, Clone, Default)]
pub struct Class {
    pub version: u8,
    pub issuer_id: [u8; 20],
    pub class_id: u32,
    pub total: u32,
    pub issued: u32,
    pub configure: u8,
    pub owner: [u8; 32],
    pub name: Vec<u8>,
    pub description: Vec<u8>,
}

impl Class {
    pub fn from_data(data: &[u8], in_cell: bool) -> Result<Self, Error> {
        if in_cell {
            if data.len() < CLASS_DATA_MIN_LEN_IN_CELL {
                return Err(Error::ClassDataInvalid);
            }

            let version: u8 = data[0];
            if version != 0 {
                return Err(Error::VersionInvalid);
            }

            let total = u32_from_slice(&data[1..5]);
            let issued = u32_from_slice(&data[5..9]);

            if total > 0 && issued > total {
                return Err(Error::ClassTotalSmallerThanIssued);
            }

            let configure: u8 = data[9];

            let name_len =
                parse_dyn_vec_len(&data[FIXED_LEN_IN_CELL..(FIXED_LEN_IN_CELL + DYN_MIN_LEN)]);
            // DYN_MIN_LEN: the min length of description
            if data.len() < FIXED_LEN_IN_CELL + name_len + DYN_MIN_LEN {
                return Err(Error::ClassDataInvalid);
            }
            let name = data[FIXED_LEN_IN_CELL..(FIXED_LEN_IN_CELL + name_len)].to_vec();

            let description_index = FIXED_LEN_IN_CELL + name_len;
            let description_len =
                parse_dyn_vec_len(&data[description_index..(description_index + DYN_MIN_LEN)]);
            // DYN_MIN_LEN: the min length of renderer
            if data.len() < description_index + description_len + DYN_MIN_LEN {
                return Err(Error::ClassDataInvalid);
            }
            let description =
                data[description_index..(description_index + description_len)].to_vec();

            let renderer_index = FIXED_LEN_IN_CELL + name_len + description_len;
            let renderer_len =
                parse_dyn_vec_len(&data[renderer_index..(renderer_index + DYN_MIN_LEN)]);

            if data.len() < renderer_index + renderer_len {
                return Err(Error::ClassDataInvalid);
            }

            return Ok(Class {
                version,
                issuer_id: [0u8; 20],
                class_id: 0,
                total,
                issued,
                configure,
                owner: [0u8; 32],
                name,
                description,
            });
        }

        if data.len() < CLASS_DATA_MIN_LEN {
            return Err(Error::ClassDataInvalid);
        }

        let version: u8 = data[0];
        if version != 0 {
            return Err(Error::VersionInvalid);
        }

        let mut issuer_id = [0u8; 20];
        issuer_id.copy_from_slice(&data[1..21]);

        let mut u32_buf = [0u8; 4];

        u32_buf.copy_from_slice(&data[21..25]);
        let class_id = u32::from_le_bytes(u32_buf);

        let total = u32_from_slice(&data[25..29]);
        let issued = u32_from_slice(&data[29..33]);

        if total > 0 && issued > total {
            return Err(Error::ClassTotalSmallerThanIssued);
        }

        let configure: u8 = data[33];

        let mut owner = [0u8; 32];

        owner.copy_from_slice(&data[34..FIXED_LEN]);

        let name_len = parse_dyn_vec_len(&data[FIXED_LEN..(FIXED_LEN + DYN_MIN_LEN)]);
        // DYN_MIN_LEN: the min length of description
        if data.len() < FIXED_LEN + name_len + DYN_MIN_LEN {
            return Err(Error::ClassDataInvalid);
        }
        let name = data[FIXED_LEN..(FIXED_LEN + name_len)].to_vec();

        let description_index = FIXED_LEN + name_len;
        let description_len =
            parse_dyn_vec_len(&data[description_index..(description_index + DYN_MIN_LEN)]);
        // DYN_MIN_LEN: the min length of renderer
        if data.len() < description_index + description_len + DYN_MIN_LEN {
            return Err(Error::ClassDataInvalid);
        }
        let description = data[description_index..(description_index + description_len)].to_vec();

        let renderer_index = FIXED_LEN + name_len + description_len;
        let renderer_len = parse_dyn_vec_len(&data[renderer_index..(renderer_index + DYN_MIN_LEN)]);

        if data.len() < renderer_index + renderer_len {
            return Err(Error::ClassDataInvalid);
        }

        return Ok(Class {
            version,
            issuer_id,
            class_id,
            total,
            issued,
            configure,
            owner,
            name,
            description,
        });
    }

    pub fn immutable_equal(&self, other: &Class) -> bool {
        self.issuer_id == other.issuer_id
            && self.class_id == self.class_id
            && self.total == other.total
            && self.configure == other.configure
            && self.name == other.name
            && self.description == other.description
    }
}

impl Class {
    fn is_zero(&self) -> bool {
        return self.issuer_id == [0u8; 20] && self.total == 0 && self.name.len() == 0;
    }

    pub fn to_h256(&self) -> H256 {
        if self.is_zero() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.issuer_id);
        hasher.update(&self.class_id.to_le_bytes());
        hasher.update(&self.total.to_le_bytes());
        hasher.update(&self.configure.to_le_bytes());
        hasher.update(&self.name);
        hasher.update(&self.description);
        hasher.finalize(&mut buf);
        buf.into()
    }
}

impl Value for Class {
    fn to_h256(&self) -> H256 {
        if self.is_zero() {
            return H256::zero();
        }
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&self.version.to_le_bytes());
        hasher.update(&self.issuer_id);
        hasher.update(&self.class_id.to_le_bytes());
        hasher.update(&self.total.to_le_bytes());
        hasher.update(&self.configure.to_le_bytes());
        hasher.update(&self.name);
        hasher.update(&self.description);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}
