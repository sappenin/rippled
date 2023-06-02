use xrpl_rust_sdk_core::core::types::AccountId;
use plugin_transactor::ReadView;
use crate::cftoken::{CFToken, CFTokenID};

pub fn find_cftoken<'a>(view: &'a ReadView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<CFToken<'a>> {
    todo!()
}