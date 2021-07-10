use alloc::vec::Vec;
use ckb_std::{
    ckb_constants::Source,
    ckb_types::{bytes::Bytes, packed::*, prelude::*},
    high_level::{load_cell_data, load_cell_type, load_cell_type_hash, QueryIter},
};

const ID_LEN: usize = 4;
pub const DYN_MIN_LEN: usize = 2; // the length of dynamic data size(u16)

pub enum Action {
    Create,
    Update,
    Destroy,
}

fn load_type_args(type_: &Script) -> Bytes {
    let type_args: Bytes = type_.args().unpack();
    type_args
}

fn parse_type_args_id(type_: Script, slice_start: usize) -> Option<u32> {
    let id_slice = &load_type_args(&type_)[slice_start..];
    if id_slice.len() != ID_LEN {
        return None;
    }
    let mut ids = [0u8; ID_LEN];
    ids.copy_from_slice(&id_slice[..]);
    Some(u32::from_be_bytes(ids))
}

fn parse_type_opt(type_opt: &Option<Script>, predicate: &dyn Fn(&Bytes) -> bool) -> bool {
    match type_opt {
        Some(type_) => predicate(&load_type_args(&type_)),
        None => false,
    }
}

pub fn count_cells_by_type_args(source: Source, predicate: &dyn Fn(&Bytes) -> bool) -> usize {
    QueryIter::new(load_cell_type, source)
        .filter(|type_opt| parse_type_opt(type_opt, predicate))
        .count()
}

pub fn count_cells_by_type_hash(source: Source, predicate: &dyn Fn(&[u8]) -> bool) -> usize {
    QueryIter::new(load_cell_type_hash, source)
        .filter(|type_hash_opt| type_hash_opt.map_or(false, |type_hash| predicate(&type_hash)))
        .count()
}

pub fn load_output_index_by_type_args(args: &Bytes) -> Option<usize> {
    QueryIter::new(load_cell_type, Source::Output)
        .position(|type_opt| type_opt.map_or(false, |type_| load_type_args(&type_)[..] == args[..]))
}

pub fn load_cell_data_by_type_args(
    source: Source,
    predicate: &dyn Fn(&Bytes) -> bool,
) -> Option<Vec<u8>> {
    QueryIter::new(load_cell_type, source)
        .position(|type_opt| type_opt.map_or(false, |type_| predicate(&load_type_args(&type_))))
        .map(|index| load_cell_data(index, source).map_or_else(|_| Vec::new(), |data| data))
}

pub fn load_cell_data_by_type_hash(
    source: Source,
    predicate: &dyn Fn(&[u8]) -> bool,
) -> Option<Vec<u8>> {
    QueryIter::new(load_cell_type_hash, source)
        .position(|type_hash_opt| type_hash_opt.map_or(false, |type_hash| predicate(&type_hash)))
        .map(|index| load_cell_data(index, source).map_or_else(|_| Vec::new(), |data| data))
}

pub fn load_output_type_args_ids(
    slice_start: usize,
    predicate: &dyn Fn(&Bytes) -> bool,
) -> Vec<u32> {
    QueryIter::new(load_cell_type, Source::Output)
        .filter(|type_opt| parse_type_opt(type_opt, predicate))
        .filter_map(|type_opt| type_opt.and_then(|type_| parse_type_args_id(type_, slice_start)))
        .collect()
}

pub fn load_smt_cell_count_by_code_hash(
    source: Source,
    predicate: &dyn Fn(&[u8]) -> bool,
) -> (i32, Vec<u8>) {
    let mut data: Vec<u8> = Vec::default();
    let count = QueryIter::new(load_cell_type, source)
        .enumerate()
        .filter(|(_, type_opt)| {
            type_opt.as_ref().map_or(false, |type_| predicate(&type_.code_hash().as_slice()))
        })
        .fold(0, |acc, (index, _)| {
            let cell_data = load_cell_data(index, source).map_or_else(|_| Vec::new(), |data| data);
            match cell_data[0] {
                0 | 1 | 3 | 4 => {
                    data = cell_data;
                    acc + 1
                }
                _ => acc,
            }
        });
    (count, data)
}

pub fn load_cell_data_by_code_hash(
    source: Source,
    args: &[u8],
    predicate: &dyn Fn(&[u8]) -> bool,
) -> Option<Vec<u8>> {
    QueryIter::new(load_cell_type, source)
        .position(|type_opt| {
            type_opt.map_or(false, |type_| {
                predicate(&type_.code_hash().as_slice()) && type_.args().as_slice() == args
            })
        })
        .map(|index| load_cell_data(index, source).map_or_else(|_| Vec::new(), |data| data))
}

pub fn parse_dyn_vec_len(data: &[u8]) -> usize {
    let mut size_buf = [0u8; 2];
    size_buf.copy_from_slice(&data[..]);
    let size = u16::from_be_bytes(size_buf) as usize;
    size + DYN_MIN_LEN
}

pub fn u32_from_slice(data: &[u8]) -> u32 {
    let mut buf = [0u8; 4];
    buf.copy_from_slice(data);
    u32::from_be_bytes(buf)
}