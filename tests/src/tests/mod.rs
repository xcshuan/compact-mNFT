use ckb_hash::new_blake2b;
use ckb_testtool::bytes::Bytes;
use ckb_types::prelude::{Builder, Entity};
use rand::{thread_rng, Rng};

mod misc;

const MAX_CYCLES: u64 = 1_000_000_000;

pub fn random_20bytes() -> Bytes {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; 20];
    rng.fill(&mut buf[..]);
    Bytes::from(buf)
}

pub fn random_32bytes() -> Bytes {
    let mut rng = thread_rng();
    let mut buf = vec![0u8; 32];
    rng.fill(&mut buf[..]);
    Bytes::from(buf)
}


pub fn calculate_type_id(input_out_point: ckb_types::packed::OutPoint) -> [u8; 32] {
    let input = ckb_types::packed::CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let mut hasher = new_blake2b();
    let output_index: u64 = 0;
    hasher.update(&input.as_bytes());
    hasher.update(&output_index.to_le_bytes());
    let mut expected_type_id = [0u8; 32];
    hasher.finalize(&mut expected_type_id);
    expected_type_id
}