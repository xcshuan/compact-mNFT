use alloc::vec::Vec;
use blake2b_ref::{Blake2b, Blake2bBuilder};
use sparse_merkle_tree::{
    default_store::DefaultStore,
    traits::{Hasher, Value},
    SparseMerkleTree, H256,
};

pub const BLAKE2B_KEY: &[u8] = &[];
pub const BLAKE2B_LEN: usize = 32;
pub const PERSONALIZATION: &[u8] = b"ckb-default-hash";

pub struct CKBBlake2bHasher(Blake2b);

impl Default for CKBBlake2bHasher {
    fn default() -> Self {
        let blake2b = Blake2bBuilder::new(BLAKE2B_LEN)
            .personal(PERSONALIZATION)
            .key(BLAKE2B_KEY)
            .build();
        CKBBlake2bHasher(blake2b)
    }
}

impl Hasher for CKBBlake2bHasher {
    fn write_h256(&mut self, h: &H256) {
        self.0.update(h.as_slice());
    }
    fn finish(self) -> H256 {
        let mut hash = [0u8; 32];
        self.0.finalize(&mut hash);
        hash.into()
    }
}

pub fn new_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(PERSONALIZATION)
        .key(BLAKE2B_KEY)
        .build()
}

#[derive(Debug, Default, Clone)]
pub struct NftValue(pub Vec<u8>);

pub type SMT = SparseMerkleTree<CKBBlake2bHasher, NftValue, DefaultStore<NftValue>>;

pub fn new_smt(pairs: Vec<(H256, NftValue)>) -> SMT {
    let mut smt = SMT::default();
    for (key, value) in pairs {
        smt.update(key, value).unwrap();
    }
    smt
}

impl Value for NftValue {
    fn to_h256(&self) -> H256 {
        let mut buf = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(&self.0);
        hasher.finalize(&mut buf);
        buf.into()
    }

    fn zero() -> Self {
        Default::default()
    }
}
