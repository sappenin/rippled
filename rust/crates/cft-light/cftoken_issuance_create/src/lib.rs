use std::ffi::{c_char, CString};
use std::pin::Pin;
use std::str::Utf8Error;
use std::vec;
use cxx::{CxxString, CxxVector, let_cxx_string, SharedPtr, UniquePtr};
use once_cell::sync::OnceCell;
use xrpl_rust_sdk_core::core::crypto::ToFromBase58;
use xrpl_rust_sdk_core::core::types::{AccountId, Hash160, XrpAmount};
use cftoken_core::cftoken_issuance;
use cftoken_core::cftoken_issuance::{CFTokenIssuance};
use plugin_transactor::{ApplyContext, Feature, PreclaimContext, preflight1, preflight2, PreflightContext, ReadView, SField, SLE, STTx, TF_UNIVERSAL_MASK, Transactor};
use plugin_transactor::transactor::{SOElement};
use rippled_bridge::{CreateNewSFieldPtr, Keylet, LedgerNameSpace, NotTEC, ParseLeafTypeFnPtr, rippled, SOEStyle, STypeFromSFieldFnPtr, STypeFromSITFnPtr, TECcodes, TEFcodes, TEMcodes, TER, TEScodes, XRPAmount};
use rippled_bridge::rippled::{account, asString, FakeSOElement, getVLBuffer, make_empty_stype, make_stvar, make_stype, OptionalSTVar, push_soelement, SerialIter, sfAccount, SFieldInfo, sfRegularKey, STBase, STPluginType, STypeExport, Value};
use rippled_bridge::TECcodes::tecDUPLICATE;
use rippled_bridge::TEScodes::tesSUCCESS;

struct CFTokenIssuanceCreate;

impl Transactor for CFTokenIssuanceCreate {
    fn pre_flight(ctx: PreflightContext) -> NotTEC {
        // TODO: If we end up adding tx flags, & them with a CFTokenIssuanceCreate flag mask
        //  to make sure the flags are valid
        // TODO: Check that transfer fee is between 0 and 50,000
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
        // TODO: Anything else to check? I don't think we need to check if the directory is full
        //  because when we go to insert the issuance, dirInsert() will return null if the
        //  directory is full
        let keylet = cftoken_issuance::keylet(
            &ctx.tx.get_account_id(&SField::sf_account()),
            &ctx.tx.get_uint160(&SField::get_plugin_field(17, 5))
        );

        if ctx.view.exists(&keylet) {
            return tecDUPLICATE.into();
        }
        tesSUCCESS.into()
    }

    fn do_apply<'a>(ctx: &'a mut ApplyContext<'a>, m_prior_balance: XrpAmount, m_source_balance: XrpAmount) -> TER {
        let source_account_id = &ctx.tx.get_account_id(&SField::sf_account());
        let account_root = ctx.view.peek(&Keylet::account(source_account_id));
        if account_root.is_none() {
            return TEFcodes::tefINTERNAL.into();
        }

        let account_root = account_root.unwrap();

        // Make sure source account has enough funds available to cover the reserve.
        let owner_count = account_root.get_uint32(&SField::sf_owner_count());
        let reserve = ctx.view.fees().account_reserve(owner_count as usize + 1);
        let balance = account_root.get_amount(&SField::sf_balance()).xrp();
        if balance < reserve {
            return TECcodes::tecINSUFFICIENT_RESERVE.into();
        }

        let asset_code = SField::get_plugin_field(17, 5);
        let issuance_keylet = cftoken_issuance::keylet(
            &source_account_id,
            &ctx.tx.get_uint160(&asset_code)
        );

        let mut issuance = CFTokenIssuance::new(&issuance_keylet)
            .set_issuer(&source_account_id)
            .set_asset_code(&ctx.tx.get_uint160(&asset_code))
            .set_asset_scale(ctx.tx.get_u8(&SField::get_plugin_field(16, 19)))
            .set_maximum_amount(ctx.tx.get_u64(&SField::get_plugin_field(3, 20)))
            .set_transfer_fee(ctx.tx.get_u16(&SField::sf_transfer_fee()))
            .set_flags(0);

        if ctx.tx.is_field_present(&SField::get_plugin_field(7, 22)) { // sfCFTMetadata
            issuance = issuance.set_cft_metadata(
                ctx.tx.get_blob(&SField::get_plugin_field(7, 22)).as_ref()
            );
        }

        ctx.view.insert_object(&issuance);

        let page  = ctx.view.dir_insert(&Keylet::owner_dir(&source_account_id), &issuance_keylet, &source_account_id);
        if page.is_none() {
            return TECcodes::tecDIR_FULL.into();
        }

        issuance.set_owner_node(page.unwrap());

        // Adjust owner count
        ctx.view.adjust_owner_count(&account_root, 1, &ctx.journal);
        ctx.view.update(&account_root);

        return tesSUCCESS.into();
    }

    fn tx_format() -> Vec<SOElement> {
        vec![
            // TODO: AssetCode is typed as an STUint160, which means you can't pass in
            //  "USD" or other ISO codes in JSON without either (1) changing parseLeafType<STUint160>
            //  or (2) typing AssetCode as a new SType called STCurrency and defining our own parseLeafType
            //  function in Rust. We should eventually do the latter.
            SOElement {
                field_code: field_code(17, 5), // AssetCode
                style: SOEStyle::soeREQUIRED,
            },
            SOElement {
                field_code: field_code(16, 19), // AssetScale
                style: SOEStyle::soeREQUIRED,
            },
            SOElement {
                field_code: field_code(1, 4), // TransferFee
                style: SOEStyle::soeOPTIONAL,
            },
            SOElement {
                field_code: field_code(3, 20), // MaximumAmount
                style: SOEStyle::soeREQUIRED,
            },
            SOElement {
                field_code: field_code(7, 22), // Metadata
                style: SOEStyle::soeOPTIONAL,
            },
        ]
    }
}

pub fn field_code(type_id: i32, field_id: i32) -> i32 {
    (type_id << 16) | field_id
}

// TODO: Consider writing a macro that generates this for you given a T: Transactor
#[no_mangle]
pub fn preflight(ctx: &rippled::PreflightContext) -> NotTEC {
    CFTokenIssuanceCreate::pre_flight(PreflightContext::new(ctx))
}

#[no_mangle]
pub fn preclaim(ctx: &rippled::PreclaimContext) -> TER {
    CFTokenIssuanceCreate::pre_claim(PreclaimContext::new(ctx))
}

#[no_mangle]
pub unsafe fn calculateBaseFee(view: &rippled::ReadView, tx: &rippled::STTx) -> XRPAmount {
    CFTokenIssuanceCreate::calculate_base_fee(ReadView::new(view), STTx::new(tx)).into()
}

#[no_mangle]
pub fn doApply(mut ctx: Pin<&mut rippled::ApplyContext>, mPriorBalance: rippled::XRPAmount, mSourceBalance: rippled::XRPAmount) -> TER {
    CFTokenIssuanceCreate::do_apply(&mut ApplyContext::new(&mut ctx.as_mut()), mPriorBalance.into(), mSourceBalance.into())
}

#[no_mangle]
pub fn getTxType() -> u16 {
    32
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
        CString::new("CFTokenIssuanceCreate").unwrap()
    });
    let ptr = c_string.as_ptr();
    ptr
}

#[no_mangle]
pub unsafe fn getTTName() -> *const i8 {
    let c_string = TT_ONCE.get_or_init(|| {
        CString::new("ttCF_TOKEN_ISSUANCE_CREATE").unwrap()
    });
    let ptr = c_string.as_ptr();
    ptr
}

#[no_mangle]
pub unsafe fn getTxFormat(mut elements: Pin<&mut CxxVector<FakeSOElement>>) {
    let tx_format = CFTokenIssuanceCreate::tx_format();
    for element in tx_format {
        push_soelement(element.field_code, element.style, elements.as_mut());
    }
}