use std::cmp::Ordering;
use xrpl_rust_sdk_core::core::types::AccountId;
use plugin_transactor::{ApplyView, Journal, ReadView, SLE, STArray};
use rippled_bridge::{Keylet, TER, UInt256};
use rippled_bridge::TEScodes::tesSUCCESS;
use crate::cftoken::{CFToken, CFTokenID};
use crate::cftoken_issuance::CFTokenIssuanceID;
use crate::cftoken_page::{CFTOKEN_PAGE_TYPE, CFTokenPage, CFTokens, keylet};
use crate::const_cftoken_page::ConstCFTokenPage;
use crate::result_codes::TECCodes::tecNO_SUITABLE_CFTOKEN_PAGE;

pub const MAX_TOKENS_PER_PAGE: usize = 32;
pub const PAGE_MASK: [u8; 32] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF
];


fn locate_page_in_read_view<'a>(view: &'a ReadView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<ConstCFTokenPage> {
    let first = keylet::cftpage(&keylet::cftpage_min(owner), issuance_id);
    let last = keylet::cftpage_min(owner);

    // This CFT can only be found in the first page with a key that's strictly
    // greater than `first`, so look for that, up until the maximum possible
    // page.
    let key = view.succ(&first, &last.next()).unwrap_or(last.key);
    view.read_typed(&Keylet::new(
        CFTOKEN_PAGE_TYPE as i16,
        key
    ))
}

fn locate_page_in_apply_view<'a>(view: &'a mut ApplyView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<CFTokenPage> {
    let first = keylet::cftpage(&keylet::cftpage_min(owner), issuance_id);
    let last = keylet::cftpage_min(owner);

    // This CFT can only be found in the first page with a key that's strictly
    // greater than `first`, so look for that, up until the maximum possible
    // page.
    let key = view.succ(&first, &last.next()).unwrap_or(last.key);
    view.peek_typed(&Keylet::new(
        CFTOKEN_PAGE_TYPE as i16,
        key
    ))
}

fn get_or_create_page<'a>(
    view: &'a mut ApplyView,
    owner: &'a AccountId,
    issuance_id: &'a CFTokenID,
    journal: &'a Journal
) -> Option<CFTokenPage> {
    let base = keylet::cftpage_min(owner);
    let first = keylet::cftpage(&base, issuance_id);
    let last = keylet::cftpage_max(owner);

    // This CFT can only be found in the first page with a key that's strictly
    // greater than `first`, so look for that, up until the maximum possible
    // page.
    let k = view.succ(&first, &last.next()).unwrap_or(last.key);
    let page_keylet = Keylet::new(
        CFTOKEN_PAGE_TYPE as i16,
        k,
    );
    let page = view.peek_typed::<CFTokenPage>(&page_keylet);

    let account_root = view.peek(&Keylet::account(owner)).unwrap();

    match page {
        None =>
        // A suitable page doesn't exist; we'll have to create one.
            Some(create_new_page(view, &last, &account_root, journal)),
        Some(mut page) => {
            let mut narr = page.get_tokens();
            if narr.len() < MAX_TOKENS_PER_PAGE {
                // The right page still has space: we're good.
                Some(page)
            } else {
                // WE KNOW THAT TOKENS HAS 32 ELEMENTS, SO ASSUME THAT FROM NOW ON

                // We need to split the page in two: the first half of the items in this
                // page will go into the new page; the rest will stay with the existing
                // page.
                //
                // Note we can't always split the page exactly in half.  All equivalent
                // CFTs must be kept on the same page.  So when the page contains
                // equivalent CFTs, the split may be lopsided in order to keep equivalent
                // CFTs on the same page.

                let cmp = narr.get(((MAX_TOKENS_PER_PAGE / 2) - 1)).unwrap().low_64();

                // TODO: Revisit this algorithm
                let mut split_pos = narr.tokens[(MAX_TOKENS_PER_PAGE / 2)..].iter()
                    .position(|t| t.low_64() != cmp);

                let mut started_from_beginning = false;
                // If we get all the way from the middle to the end with only
                // equivalent NFTokens then check the front of the page for a
                // place to make the split.
                if split_pos.is_none() {
                    started_from_beginning = true;
                    split_pos = narr.tokens.iter()
                        .position(|t| t.low_64() == cmp)
                }

                // There should be no circumstance when splitIter == end(), but if it
                // were to happen we should bail out because something is confused.
                if split_pos.is_none() {
                    return None;
                }

                // If splitIter == begin(), then the entire page is filled with
                // equivalent tokens.  This requires special handling.
                if started_from_beginning && split_pos == Some(0) {
                    // TODO: Impl low_64 on IssuanceID
                    split_pos = match issuance_id.as_ref()[24..].cmp(cmp) {
                        Ordering::Equal =>
                        // If the passed in id belongs exactly on this (full) page
                        // this account simply cannot store the CFT.
                            None,
                        Ordering::Less => {
                            // If neither of those conditions apply then put all of
                            // narr into carr and produce an empty narr where the new CFT
                            // will be inserted.  Leave the split at narr[0].
                            split_pos
                        }
                        Ordering::Greater =>
                        // We need to leave the entire contents of this page in
                        // narr so carr stays empty.  The new CFT will be
                        // inserted in carr.  This keeps the CFTs that must be
                        // together all on their own page.
                            Some(narr.len() - 1)
                    }
                }

                split_pos.map(|pos| {
                    // Move all CFTokens from narr[pos..] into carr
                    let carr = CFTokens::new(narr.tokens.drain(pos..).collect());

                    // Note that we use uint256::next() because there's a subtlety in the way
                    // CFT pages are structured.  The low 64-bits of CFT ID must be strictly
                    // less than the low 64-bits of the enclosing page's index.  In order to
                    // accommodate that requirement we use an index one higher than the
                    // largest CFT in the page.
                    let token_id_for_new_page = if narr.len() == MAX_TOKENS_PER_PAGE {
                        // This is ugly, but we don't have a ++ operator on Hash256, so we need to
                        // convert to a UInt256 and call the bridged next() function to add
                        // 1 to the ID, then convert back to CFTokenID.
                        // TODO: Consider adding math operators to Hash256 in xrpl-rust-sdk
                        CFTokenIssuanceID::from(
                            UInt256::from(narr.get(MAX_TOKENS_PER_PAGE - 1).unwrap().issuance_id()).next()
                        )
                    } else {
                        carr.get(0).unwrap().issuance_id()
                    };

                    let new_page_k = keylet::cftpage(&base, &token_id_for_new_page);
                    let mut new_page = CFTokenPage::from(
                        &new_page_k
                    );
                    new_page.set_tokens(narr);
                    new_page.set_next_page_min(&page_keylet.into());

                    let new_page_k_as_u256 = new_page_k.into();
                    if let Some(ppm) = page.get_previous_page_min() {
                        new_page.set_previous_page_min(&ppm);

                        if let Some(mut p3) = view.peek_typed::<CFTokenPage>(
                            &Keylet::new(CFTOKEN_PAGE_TYPE as i16, UInt256::from(ppm))
                        ) {
                            p3.set_next_page_min(&new_page_k_as_u256);
                            view.update_object(&p3);
                        }
                    }

                    view.insert_object(&new_page);

                    page.set_tokens(carr);
                    page.set_previous_page_min(&new_page_k_as_u256);
                    view.update_object(&page);

                    view.adjust_owner_count(
                        &account_root,
                        1,
                        journal
                    );

                    if first.key < new_page_k.key {
                        new_page
                    } else {
                        page
                    }
                })
            }
        }
    }
}

fn create_new_page(
    view: &mut ApplyView,
    keylet: &Keylet,
    account_root: &SLE,
    journal: &Journal
) -> CFTokenPage {
    let mut page = CFTokenPage::from(keylet);
    page.set_tokens(CFTokens::new_empty());
    view.insert_object(&page);
    view.adjust_owner_count(
        account_root,
        1,
        journal
    );
    page
}

// TODO: Implement Ord/PartialOrd on CFTokenID instead of a compareTokens port

pub fn insert_token<'a>(
    view: &'a mut ApplyView,
    owner: &'a AccountId,
    cft: CFToken,
    journal: &Journal
) -> TER {
    get_or_create_page(view, owner, &cft.token_id(), journal)
        .map_or(tecNO_SUITABLE_CFTOKEN_PAGE.into(), |mut page| {
            let mut tokens = page.get_tokens();
            tokens.insert_sorted(cft);
            page.set_tokens(tokens);
            view.update_object(&page);
            tesSUCCESS.into()
        })
}

// TODO: Do we need mergePages port? It's only called in removeToken and we aren't enabling
//  CFToken deletion yet

// TODO: Maybe implement removeToken if we end up allowing CFToken deletion

pub fn find_token<'a>(view: &'a ReadView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<CFToken<'a>> {
    // If the page couldn't be found, the given CFT isn't owned by this account. locate_page_in_read_view
    // will be None if this is the case.
    locate_page_in_read_view(view, owner, issuance_id)
        .map(|page|
            // We found a candidate page, but the given CFT may not be in it.
            page.get_tokens().tokens.into_iter()
                .find(|token| &token.token_id() == issuance_id)
        ).flatten()
}

pub fn find_token_and_page<'a>(view: &'a mut ApplyView, owner: &'a AccountId, issuance_id: &'a CFTokenID) -> Option<(CFToken<'a>, CFTokenPage)> {
    // If the page couldn't be found, the given CFT isn't owned by this account. locate_page_in_apply_view
    // will be None if this is the case.
    locate_page_in_apply_view(view, owner, issuance_id)
        .map(|page| {
            // We found a candidate page, but the given CFT may not be in it.
            // If it is, return a tuple of the CFT and the CFTokenPage
            page.get_tokens().tokens.into_iter()
                .find(|token| &token.token_id() == issuance_id)
                .map(|found| (found, page))
        }).flatten()
}