use xrpl_rust_sdk_core::core::types::AccountId;
use plugin_transactor::{ApplyView, ReadView};
use rippled_bridge::rippled::SLE;
use rippled_bridge::TER;
use crate::cftoken::{CFToken, CFTokenID};
use crate::cftoken_page::CFTokenPage;


pub fn locate_page_in_read_view<'a>(view: &'a ReadView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> CFTokenPage {
    todo!()
}

pub fn locate_page_in_apply_view<'a>(view: &'a mut ApplyView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> CFTokenPage {
    todo!()
}

pub fn get_page_for_token<'a>(view: &'a mut ApplyView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<CFTokenPage> {
    todo!()
}

pub fn create_page<'a>(view: &'a mut ApplyView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Result<CFTokenPage, TER> {
    todo!()
}

// TODO: Implement Ord/PartialOrd on CFTokenID instead of a compareTokens port

pub fn insert_token<'a>(view: &'a mut ApplyView, owner: &'a AccountId, cft: &CFToken) -> TER {
    todo!()
}

// TODO: Do we need mergePages port? It's only called in removeToken and we aren't enabling
//  CFToken deletion yet

// TODO: Maybe implement removeToken if we end up allowing CFToken deletion

pub fn find_token<'a>(view: &'a ReadView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<CFToken<'a>> {
    todo!()
}

pub fn find_token_and_page<'a>(view: &'a mut ApplyView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<(CFToken<'a>, CFTokenPage)> {
    todo!()
}