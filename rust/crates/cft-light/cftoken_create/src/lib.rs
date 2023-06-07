use std::cmp::max;
use std::ffi::{c_char, CString};
use std::pin::Pin;
use std::str::Utf8Error;
use std::vec;
use cxx::{CxxString, CxxVector, let_cxx_string, SharedPtr, UniquePtr};
use once_cell::sync::OnceCell;
use xrpl_rust_sdk_core::core::crypto::ToFromBase58;
use xrpl_rust_sdk_core::core::types::{AccountId, Hash160, Hash256, XrpAmount};
use plugin_transactor::{ApplyContext, ConstSLE, Feature, PreclaimContext, preflight1, preflight2, PreflightContext, ReadView, SField, SLE, STAmount, STObject, STTx, TF_PAYMENT_MASK, TF_UNIVERSAL_MASK, Transactor, TxConsequences};
use plugin_transactor::transactor::{MakeTxConsequences, SOElement};
use rippled_bridge::{CreateNewSFieldPtr, Keylet, LedgerNameSpace, NotTEC, ParseLeafTypeFnPtr, rippled, SOEStyle, STypeFromSFieldFnPtr, STypeFromSITFnPtr, TECcodes, TEFcodes, TEMcodes, TER, TEScodes, XRPAmount};
use rippled_bridge::LedgerSpecificFlags::lsfRequireDestTag;
use rippled_bridge::rippled::{account, asString, FakeSOElement, getVLBuffer, make_empty_stype, make_stvar, make_stype, OptionalSTVar, push_soelement, SerialIter, sfAccount, SFieldInfo, sfRegularKey, STBase, STPluginType, STypeExport, Value};
use rippled_bridge::TECcodes::{tecDST_TAG_NEEDED, tecNO_DST_INSUF_XRP, tecUNFUNDED_PAYMENT};
use rippled_bridge::TEFcodes::tefINTERNAL;
use rippled_bridge::TEMcodes::{temBAD_AMOUNT, temBAD_ISSUER, temINVALID_FLAG, temREDUNDANT};
use rippled_bridge::TERcodes::terNO_ACCOUNT;
use rippled_bridge::TEScodes::tesSUCCESS;
use cftoken_core::cftoken::{CFToken};
use cftoken_core::{cftoken_issuance, CFTokenFields};
use cftoken_core::cftoken_utils::find_token;

struct CFTokenCreate;

impl Transactor for CFTokenCreate {
    fn pre_flight(ctx: PreflightContext) -> NotTEC {
        let preflight1 = preflight1(&ctx);
        if preflight1 != tesSUCCESS {
            return preflight1;
        }

        let tx = ctx.tx();
        if tx.flags() & TF_UNIVERSAL_MASK != 0 {
            return temINVALID_FLAG.into();
        }

        if ctx.tx().get_account_id(&SField::sf_account()) == ctx.tx().get_account_id(&SField::sf_issuer()) {
            // TODO: Revisit if this is the correct error code
            return temBAD_ISSUER.into();
        }

        preflight2(&ctx)
    }

    fn pre_claim(ctx: PreclaimContext) -> TER {
        // 1. Source account exists.
        // 2. Issuance exists.
        // 3. Account doesn't already hold the token.
        // 4. (future) Make sure the issuance isn't frozen. This will happen in payment, but
        //    might as well save the ledger some storage by refusing to create a CFToken.
        // 4. (future) lsfDisableIncomingCFTs is not enabled on the issuance.
        //     (Setting that flag on the issuance would allow the issuer to effectively
        //     stop anyone who doesn't already hold the token from holding the token. This
        //     could be used to create a "closed loop" system with your CFT.

        let source_account_id = ctx.tx.get_account_id(&SField::sf_account());
        let source_keylet = Keylet::account(&source_account_id);
        // TODO: Maybe we don't need to check if the source account exists -- this may
        //  happen at a higher layer?
        match ctx.view.read(&source_keylet) {
            None => terNO_ACCOUNT.into(),
            Some(_) => {
                let issuance_keylet = cftoken_issuance::keylet(
                    &ctx.tx.get_account_id(&SField::sf_issuer()),
                    &ctx.tx.get_uint160(&SField::sf_asset_code())
                );
                if ctx.view.read(&issuance_keylet).is_none() {
                    return temBAD_ISSUER.into();
                }
                match find_token(&ctx.view, &source_account_id, &issuance_keylet.into()) {
                    None => tesSUCCESS.into(),
                    Some(_) => temREDUNDANT.into()
                }
            }
        }
    }

    fn do_apply<'a>(ctx: &'a mut ApplyContext<'a>, m_prior_balance: XrpAmount, m_source_balance: XrpAmount) -> TER {

        tesSUCCESS.into()
    }

    fn tx_format() -> Vec<SOElement> {
        vec![]
    }
}

pub fn field_code(type_id: i32, field_id: i32) -> i32 {
    (type_id << 16) | field_id
}

// TODO: Consider writing a macro that generates this for you given a T: Transactor
#[no_mangle]
pub fn preflight(ctx: &rippled::PreflightContext) -> NotTEC {
    CFTokenCreate::pre_flight(PreflightContext::new(ctx))
}

#[no_mangle]
pub fn preclaim(ctx: &rippled::PreclaimContext) -> TER {
    CFTokenCreate::pre_claim(PreclaimContext::new(ctx))
}

#[no_mangle]
pub unsafe fn calculateBaseFee(view: &rippled::ReadView, tx: &rippled::STTx) -> XRPAmount {
    CFTokenCreate::calculate_base_fee(ReadView::new(view), STTx::new(tx)).into()
}

#[no_mangle]
pub fn doApply(mut ctx: Pin<&mut rippled::ApplyContext>, mPriorBalance: rippled::XRPAmount, mSourceBalance: rippled::XRPAmount) -> TER {
    CFTokenCreate::do_apply(&mut ApplyContext::new(&mut ctx.as_mut()), mPriorBalance.into(), mSourceBalance.into())
}

#[no_mangle]
pub fn getTxType() -> u16 {
    34
}

static FIELD_NAMES_ONCE: OnceCell<Vec<CString>> = OnceCell::new();

/// This method is called by rippled to get the SField information from this Plugin Transactor.
#[no_mangle]
pub fn getSFields(mut s_fields: Pin<&mut CxxVector<SFieldInfo>>) {
    // SFields are all defined in C++ so they can be used in the CFTokenIssuance SLE
}

#[no_mangle]
pub fn getSTypes(mut s_types: Pin<&mut CxxVector<STypeExport>>) {
    // No new STypes for this one
}

static NAME_ONCE: OnceCell<CString> = OnceCell::new();
static TT_ONCE: OnceCell<CString> = OnceCell::new();

#[no_mangle]
pub unsafe fn getTxName() -> *const i8 {
    let c_string = NAME_ONCE.get_or_init(|| {
        CString::new("CFTokenCreate").unwrap()
    });
    let ptr = c_string.as_ptr();
    ptr
}

#[no_mangle]
pub unsafe fn getTTName() -> *const i8 {
    let c_string = TT_ONCE.get_or_init(|| {
        CString::new("ttCFTOKEN_CREATE").unwrap()
    });
    let ptr = c_string.as_ptr();
    ptr
}

#[no_mangle]
pub unsafe fn getTxFormat(mut elements: Pin<&mut CxxVector<FakeSOElement>>) {
    let tx_format = CFTokenCreate::tx_format();
    for element in tx_format {
        push_soelement(element.field_code, element.style, elements.as_mut());
    }
}