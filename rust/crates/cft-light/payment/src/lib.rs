use std::ffi::{c_char, CString};
use std::pin::Pin;
use std::str::Utf8Error;
use std::vec;
use cxx::{CxxString, CxxVector, let_cxx_string, SharedPtr, UniquePtr};
use once_cell::sync::OnceCell;
use xrpl_rust_sdk_core::core::crypto::ToFromBase58;
use xrpl_rust_sdk_core::core::types::{AccountId, Hash160, XrpAmount};
use plugin_transactor::{ApplyContext, Feature, PreclaimContext, preflight1, preflight2, PreflightContext, ReadView, SField, SLE, STTx, TF_UNIVERSAL_MASK, Transactor, TxConsequences};
use plugin_transactor::transactor::{MakeTxConsequences, SOElement, WriteToSle};
use rippled_bridge::{CreateNewSFieldPtr, Keylet, LedgerNameSpace, NotTEC, ParseLeafTypeFnPtr, rippled, SOEStyle, STypeFromSFieldFnPtr, STypeFromSITFnPtr, TECcodes, TEFcodes, TEMcodes, TER, TEScodes, XRPAmount};
use rippled_bridge::rippled::{account, asString, FakeSOElement, getVLBuffer, make_empty_stype, make_stvar, make_stype, OptionalSTVar, push_soelement, SerialIter, sfAccount, SFieldInfo, sfRegularKey, STBase, STPluginType, STypeExport, Value};
use rippled_bridge::TEScodes::tesSUCCESS;

struct Payment;

impl Transactor for Payment {
    fn pre_flight(ctx: PreflightContext) -> NotTEC {
        let preflight1 = preflight1(&ctx);
        if preflight1 != TEScodes::tesSUCCESS {
            return preflight1;
        }

        if ctx.tx().flags() & TF_UNIVERSAL_MASK != 0 {
            return TEMcodes::temINVALID_FLAG.into();
        }

        preflight2(&ctx)
    }

    fn pre_claim(ctx: PreclaimContext) -> TER {
        TEScodes::tesSUCCESS.into()
    }

    fn do_apply<'a>(ctx: &'a mut ApplyContext<'a>, m_prior_balance: XrpAmount, m_source_balance: XrpAmount) -> TER {
        return tesSUCCESS.into();
    }

    fn tx_format() -> Vec<SOElement> {
        vec![
            SOElement {
                field_code: SField::sf_destination().code(),
                style: SOEStyle::soeREQUIRED,
            },
            SOElement {
                field_code: SField::sf_amount().code(),
                style: SOEStyle::soeREQUIRED,
            },
            SOElement {
                field_code: SField::sf_invoice_id().code(),
                style: SOEStyle::soeOPTIONAL,
            },
            SOElement {
                field_code: SField::sf_destination_tag().code(),
                style: SOEStyle::soeOPTIONAL,
            }
        ]
    }
}

impl MakeTxConsequences for Payment {
    fn make_tx_consequences(ctx: PreflightContext) -> TxConsequences {
        TxConsequences::with_potential_spend(
            &ctx.tx(),
            ctx.tx().get_amount(&SField::sf_amount()).xrp()
        )
    }
}

pub fn field_code(type_id: i32, field_id: i32) -> i32 {
    (type_id << 16) | field_id
}

// TODO: Consider writing a macro that generates this for you given a T: Transactor
#[no_mangle]
pub fn preflight(ctx: &rippled::PreflightContext) -> NotTEC {
    Payment::pre_flight(PreflightContext::new(ctx))
}

#[no_mangle]
pub fn makeTxConsequences(ctx: &rippled::PreflightContext) -> rippled_bridge::tx_consequences::TxConsequences {
    Payment::make_tx_consequences(PreflightContext::new(ctx)).into()
}

#[no_mangle]
pub fn preclaim(ctx: &rippled::PreclaimContext) -> TER {
    Payment::pre_claim(PreclaimContext::new(ctx))
}

#[no_mangle]
pub unsafe fn calculateBaseFee(view: &rippled::ReadView, tx: &rippled::STTx) -> XRPAmount {
    Payment::calculate_base_fee(ReadView::new(view), STTx::new(tx)).into()
}

#[no_mangle]
pub fn doApply(mut ctx: Pin<&mut rippled::ApplyContext>, mPriorBalance: rippled::XRPAmount, mSourceBalance: rippled::XRPAmount) -> TER {
    Payment::do_apply(&mut ApplyContext::new(&mut ctx.as_mut()), mPriorBalance.into(), mSourceBalance.into())
}

#[no_mangle]
pub fn getTxType() -> u16 {
    33
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
        CString::new("Payment2").unwrap()
    });
    let ptr = c_string.as_ptr();
    ptr
}

#[no_mangle]
pub unsafe fn getTTName() -> *const i8 {
    let c_string = TT_ONCE.get_or_init(|| {
        CString::new("ttPAYMENT_2").unwrap()
    });
    let ptr = c_string.as_ptr();
    ptr
}

#[no_mangle]
pub unsafe fn getTxFormat(mut elements: Pin<&mut CxxVector<FakeSOElement>>) {
    let tx_format = Payment::tx_format();
    for element in tx_format {
        push_soelement(element.field_code, element.style, elements.as_mut());
    }
}