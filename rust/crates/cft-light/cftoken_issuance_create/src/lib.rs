use std::ffi::{c_char, CString};
use std::pin::Pin;
use std::str::Utf8Error;
use std::vec;
use cxx::{CxxString, CxxVector, let_cxx_string, SharedPtr, UniquePtr};
use once_cell::sync::OnceCell;
use xrpl_rust_sdk_core::core::crypto::ToFromBase58;
use xrpl_rust_sdk_core::core::types::{AccountId, XrpAmount};
use plugin_transactor::{ApplyContext, Feature, PreclaimContext, preflight1, preflight2, PreflightContext, ReadView, SField, SLE, STTx, TF_UNIVERSAL_MASK, Transactor};
use plugin_transactor::transactor::SOElement;
use rippled_bridge::{CreateNewSFieldPtr, Keylet, LedgerNameSpace, NotTEC, ParseLeafTypeFnPtr, rippled, SOEStyle, STypeFromSFieldFnPtr, STypeFromSITFnPtr, TECcodes, TEFcodes, TEMcodes, TER, TEScodes, XRPAmount};
use rippled_bridge::rippled::{account, asString, FakeSOElement, getVLBuffer, make_empty_stype, make_stvar, make_stype, OptionalSTVar, push_soelement, SerialIter, sfAccount, SFieldInfo, sfRegularKey, STBase, STPluginType, STypeExport, Value};
use rippled_bridge::TEScodes::tesSUCCESS;

struct CFTokenIssuanceCreate;

const CFT_ISSUANCE_TYPE: u16 = 0x007Eu16;

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
        let keylet = Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
            .key(ctx.tx.get_account_id(&SField::sf_account()))
            .key(ctx.tx.get_uint160(&SField::get_plugin_field(17, 5)))
            .build();
        if ctx.view.exists(&keylet) {
            return TECcodes::tecDUPLICATE.into();
        }
        TEScodes::tesSUCCESS.into()
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
        let issuance_keylet = Keylet::builder(CFT_ISSUANCE_TYPE as i16, CFT_ISSUANCE_TYPE)
            .key(source_account_id)
            .key(ctx.tx.get_uint160(&asset_code))
            .build();

        let mut slep = SLE::from(&issuance_keylet);
        slep.set_field_u32(&SField::sf_flags(), ctx.tx.flags()); // sfFlags
        slep.set_field_account(&SField::sf_issuer(), &source_account_id); // sfIssuer
        slep.set_field_h160(&asset_code, &ctx.tx.get_uint160(&asset_code)); // sfAssetCode
        slep.set_field_u8(&SField::get_plugin_field(16, 19), ctx.tx.get_u8(&SField::get_plugin_field(16, 19))); // sfAssetScale
        slep.set_field_u64(&SField::get_plugin_field(3, 20), ctx.tx.get_u64(&SField::get_plugin_field(3, 20))); // sfMaximumAmount
        // Only set sfTransferFee if specified in the transaction and != 0. Otherwise,
        // the transaction will succeed but loading the ledger entry will fail because
        // you can't set an soeDEFAULT field to the default value (0)
        let transfer_fee = ctx.tx.get_u16(&SField::sf_transfer_fee());
        if transfer_fee != 0 {
            slep.set_field_u16(&SField::sf_transfer_fee(), transfer_fee);
        }

        // Only set sfCFTMetadata if it's specified in the transaction.
        if ctx.tx.is_field_present(&SField::get_plugin_field(7, 22)) {
            slep.set_field_blob(&SField::get_plugin_field(7, 22), &ctx.tx.get_blob(&SField::get_plugin_field(7, 22)));
        }

        ctx.view.insert(&slep);

        let page  = ctx.view.dir_insert(&Keylet::owner_dir(&source_account_id), &issuance_keylet, &source_account_id);
        if page.is_none() {
            return TECcodes::tecDIR_FULL.into();
        }

        slep.set_field_u64(&SField::sf_owner_node(), page.unwrap());

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


// Things we need:
// 1. New SType called CFTAmount (don't need until Payment)
// 2. New SFields for CFTokenIssuanceCreate [DONE (in C++)]
// 2. New ledger object called CFTokenIssuance [DONE (in C++)]
// 3. New ledger object called CFToken [DONE (in C++)]
// 4. New transactor CFTokenIssuanceCreate
// 5. Add new ledger objects to LedgerEntry.cpp doLedgerEntry RPC handler
// 6. Add keylets to look up CFTokenIssuances by
//      1.
// 7. Add keylets to look up CFTokens by
//      1.

//////////////
// TODO: Figure Out where these go.
//////////////

// LedgerFormats.h
// /** A ledger object which identifies an the issuance details of a CFT.
//
//        \sa keylet::cftissuance
//  */
// ltCFTOKEN_ISSUANCE = 0x0033,

// /** A ledger object which contains a list of CFTs

       // \sa keylet::cftpage_min, keylet::cftpage_max, keylet::cftpage
 // */
// ltCFTOKEN_PAGE = 0x0034,


////////////
// For LAter
////////////

// /**
// @ingroup protocol
//  */
// enum LedgerSpecificFlags {

// }

// TODO: Define SField in xrpl-rs?
// TODO: Where to define the TxType? TxFormats.h?
    // ttCFTOKEN_ISSUANCE_CREATE = 30


// SFields for CFTokenIssuance
// sfFlags --> already exists in SField.cpp
// sfIssuer ==> "Issuer" | ACCOUNT, 4 --> already exists in SField.cpp?
// sfCurrencyCode ==> "CFTCurrencyCode" | UINT160 | 5
// sfAssetScale ==> "AssetScale" | UINT8 | 4
// sfTransferFee ==> "TransferFee" | UINT16 |
// sfMaximumAmount ==> "MaximumAmount" | UINT64 | 14
// sfOutstandingAmount ==> "OutstandingAmount" | UINT64 | 15
// sfLockedAmount ==> "LockedAmount" | UINT64 | 16

// sfMetadata => "Metadata" | BLOB | --> CONSTRUCT_UNTYPED_SFIELD(sfMetadata,            "Metadata",             METADATA,    257);
// CONSTRUCT_TYPED_SFIELD(sfOwnerNode,             "OwnerNode",            UINT64,     4);