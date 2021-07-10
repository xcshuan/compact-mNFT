// Import from `core` instead of from `std` since we are in no-std mode
use core::result::Result;

// Import CKB syscalls and structures
// https://nervosnetwork.github.io/ckb-std/riscv64imac-unknown-none-elf/doc/ckb_std/index.html
use ckb_std::{
    ckb_constants::Source::{GroupInput, GroupOutput, Input},
    ckb_types::prelude::*,
    high_level::{load_cell_data, load_cell_type},
};
use script_utils::{class::Class, helpers::load_smt_cell_count_by_code_hash, issuer::Issuer};

use crate::{
    class::{handle_destroying_class, handle_update_class},
    error::Error,
    issuer::{handle_creation_issuer, handle_update_issuer},
};

use crate::{issuer::handle_destroying_issuer, nft::handle_update_nft};

fn check_code_hash<'a>(hash: &'a [u8]) -> impl Fn(&[u8]) -> bool + 'a {
    move |code_hash: &[u8]| code_hash[0..32] == hash[0..32]
}

pub fn main() -> Result<(), Error> {
    // 得到当前脚本的code_hash
    let type_ = load_cell_type(0, GroupInput).unwrap().unwrap();
    let code_hash = type_.code_hash();

    // 计算输入的cell中同code_hash，且具有smt属性的cell的数量
    let (input_smt_cell_count, input_smt_cell_data) =
        load_smt_cell_count_by_code_hash(Input, &check_code_hash(code_hash.as_slice()));
    // 计算输入的cell中同code_hash，且具有smt属性的cell的数量
    let (output_smt_cell_count, output_smt_cell_data) =
        load_smt_cell_count_by_code_hash(Input, &check_code_hash(code_hash.as_slice()));

    match (input_smt_cell_count, output_smt_cell_count) {
        // 没有SMT_cell，只有一种情况，即游离态NFT的update，NFT只能在smt中创建或者销毁
        (0, 0) => {
            return handle_update_nft();
        }
        // 某种SMT_cell的销毁
        (1, 0) => {
            // 如果当前type_script是NFT_cell，则处理nft_cell的update
            let data = load_cell_data(0, GroupInput).unwrap();
            if data != input_smt_cell_data {
                return handle_update_nft();
            }
            //否则的话，当前type_script处理销毁
            match input_smt_cell_data[0] {
                // issuer_cell
                0 => {
                    let input_issuer = Issuer::from_data(&input_smt_cell_data[1..])?;
                    return handle_destroying_issuer(input_issuer);
                }
                // class_cell
                1 => {
                    let input_class = Class::from_data(&input_smt_cell_data[1..], true)?;
                    return handle_destroying_class(input_class);
                }
                // single_owner cell
                3 => {
                    todo!()
                }
                // multi_owner cell
                4 => {
                    todo!()
                }
                _ => return Err(Error::ClassDataInvalid),
            }
        }
        // 某种SMT_cell的创建，可以被创建的，包括issuer，single_owner，multi_owner
        // class_cell的创建只能从issuer中抽取
        (0, 1) => {
            // 如果当前type_script是NFT_cell，则处理nft_cell的update
            let data = load_cell_data(0, GroupOutput).unwrap();
            if data != output_smt_cell_data {
                return handle_update_nft();
            }
            match output_smt_cell_data[0] {
                // issuer_cell
                0 => {
                    let issuer = Issuer::from_data(&output_smt_cell_data[1..])?;
                    return handle_creation_issuer(issuer);
                }
                // single_owner cell
                3 => {
                    todo!()
                }
                // multi_owner cell
                4 => {
                    todo!()
                }
                _ => return Err(Error::ClassDataInvalid),
            }
        }
        // 某种smt_cell的更新
        // 不允许与此交易附带游离NFT更新
        (1, 1) => {
            // 仅由SMT_cell来检查具体逻辑
            let data = load_cell_data(0, GroupInput).unwrap();
            if data != input_smt_cell_data {
                return Ok(());
            }

            // 必须是同种类型的
            if input_smt_cell_data[0] != output_smt_cell_data[0] {
                return Err(Error::ClassDataInvalid);
            }
            match input_smt_cell_data[0] {
                0 => {
                    let input_issuer = Issuer::from_data(&input_smt_cell_data[1..])?;
                    let output_issuer = Issuer::from_data(&output_smt_cell_data[1..])?;
                    return handle_update_issuer(input_issuer, output_issuer);
                }
                1 => {
                    let input_class = Class::from_data(&input_smt_cell_data[1..], true)?;
                    let output_class = Class::from_data(&output_smt_cell_data[1..], true)?;
                    return handle_update_class(input_class, output_class);
                }
                3 => {
                    todo!()
                }
                4 => {
                    todo!()
                }
                _ => return Err(Error::ClassDataInvalid),
            }
        }
        // 只有一种可能，就是从issuer_cell中抽取了class_cell出去
        // 不允许与此交易附带游离NFT更新
        (1, n) => {
            // 仅由SMT_cell来检查具体逻辑
            let data = load_cell_data(0, GroupInput).unwrap();
            if data != input_smt_cell_data {
                return Ok(());
            }
            // 必须是issuer_cell
            if input_smt_cell_data[0] != 0 {
                return Err(Error::ClassDataInvalid);
            }
        }
        // 只有一种可能，就是在issuer_cell中插入了class_cell进来
        // 不允许与此交易附带游离NFT更新
        (n, 1) => {
            // 仅由SMT_cell来检查具体逻辑
            let data = load_cell_data(0, GroupOutput).unwrap();
            if data != output_smt_cell_data {
                return Ok(());
            }
            // 必须是issuer_cell
            if output_smt_cell_data[0] != 0 {
                return Err(Error::ClassDataInvalid);
            }
        }
        // 其他情况，暂时报错
        (_, _) => return Err(Error::ClassDataInvalid),
    }
    Ok(())
}
