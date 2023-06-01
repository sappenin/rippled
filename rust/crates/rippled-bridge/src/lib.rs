use std::ffi::{c_char, c_void};
use std::fmt::Formatter;
use std::mem::MaybeUninit;
use std::ops::Deref;
use std::pin::Pin;
use cxx::{CxxString, CxxVector, ExternType, SharedPtr, type_id, UniquePtr};
use cxx::kind::Trivial;
use cxx::vector::VectorElement;
use sha2::{Sha512, Digest};
use xrpl_rust_sdk_core::core::types::{ACCOUNT_ONE, AccountId, Hash160, Hash256, XrpAmount};

pub mod ter;
pub mod flags;
pub mod tx_consequences;

pub use ter::{TER, NotTEC, TEFcodes, TEMcodes, TELcodes, TECcodes, TEScodes, TERcodes};
pub use flags::{LedgerSpecificFlags, ApplyFlags};
use crate::rippled::{OptionalSTVar, SerialIter, SField, STBase, STPluginType, Value};

#[cxx::bridge]
pub mod rippled {

    extern "Rust" {
        // This function is unused, but exists only to ensure that line 11's interface is bridge
        // compatible.
        /*pub fn preflight(ctx: &PreflightContext) -> NotTEC;
        pub fn preclaim(ctx: &PreclaimContext) -> TER;
        pub fn calculateBaseFee(view: &ReadView, tx: &STTx) -> XRPAmount;
        pub fn doApply(mut ctx: Pin<&mut ApplyContext>, mPriorBalance: XRPAmount, mSourceBalance: XRPAmount) -> TER;*/
    }

    // Safety: the extern "C++" block is responsible for deciding whether to expose each signature
    // inside as safe-to-call or unsafe-to-call. If an extern block contains at least one
    // safe-to-call signature, it must be written as an unsafe extern block, which serves as
    // an item level unsafe block to indicate that an unchecked safety claim is being made about
    // the contents of the block.

    // These are C++ functions that can be called by Rust.
    // Within the extern "C++" part, we list types and functions for which C++ is the source of
    // truth, as well as the header(s) that declare those APIs.
    #[namespace = "ripple"]
    unsafe extern "C++" {
        ////////////////////////////////
        // One or more headers with the matching C++ declarations for the
        // enclosing extern "C++" block. Our code generators don't read it
        // but it gets #include'd and used in static assertions to ensure
        // our picture of the FFI boundary is accurate.
        ////////////////////////////////
        include!("rippled-bridge/include/rippled_api.h");

        ////////////////////////////////
        // Zero or more opaque types which both languages can pass around
        // but only C++ can see the fields.
        // type BlobstoreClient;
        ////////////////////////////////
        type TEFcodes = super::TEFcodes;
        type TEMcodes = super::TEMcodes;
        type TELcodes = super::TELcodes;
        type TERcodes = super::TERcodes;
        type TEScodes = super::TEScodes;
        type TECcodes = super::TECcodes;
        type AccountID = super::AccountID;
        type NotTEC = super::NotTEC;
        type TER = super::TER;
        pub type PreflightContext;
        pub type PreclaimContext;
        pub type ApplyContext;
        pub type XRPAmount = super::XRPAmount;
        pub type ReadView;
        pub type ApplyView;
        pub type STTx;
        pub type Rules;
        pub type uint256 = super::UInt256;
        pub type uint160 = super::UInt160;
        type Transactor;
        pub type SField;
        pub type STObject;
        type Keylet = super::Keylet;
        type LedgerEntryType = super::LedgerEntryType;
        pub type SLE;
        pub type Application;
        pub type Fees;
        type ApplyFlags = super::ApplyFlags;
        pub type FakeSOElement /*= super::FakeSOElement*/;
        type SOEStyle = super::SOEStyle;
        pub type SFieldInfo;
        pub type SerialIter;
        pub type STBase;
        #[namespace = "Json"]
        pub type Value;
        pub type Buffer;
        pub type STPluginType;
        pub type STAmount;
        pub type STBlob;
        #[namespace = "beast"]
        pub type Journal;
        pub type SeqProxy = super::tx_consequences::SeqProxy;


        ////////////////////////////////
        // Functions implemented in C++.
        ////////////////////////////////

        pub fn preflight1(ctx: &PreflightContext) -> NotTEC;
        pub fn preflight2(ctx: &PreflightContext) -> NotTEC;

        // In AccountId.h --> AccountID const & xrpAccount();
        pub fn xrpAccount<'a>() -> &'a AccountID;

        pub fn data(self: &AccountID) -> *const u8;
        pub fn begin(self: &AccountID) -> *const u8;
        pub fn end(self: &AccountID) -> *const u8;

        #[namespace = "ripple::keylet"]
        pub fn account(id: &AccountID) -> Keylet;

        #[namespace = "ripple::keylet"]
        pub fn signers(id: &AccountID) -> Keylet;

        #[namespace = "ripple::keylet"]
        pub fn ownerDir(id: &AccountID) -> Keylet;
    }

    unsafe extern "C++" {
        include!("rippled-bridge/include/rippled_api.h");

        pub type STypeExport;
        pub type CreateNewSFieldPtr = super::CreateNewSFieldPtr;
        pub type ParseLeafTypeFnPtr = super::ParseLeafTypeFnPtr;
        pub type STypeFromSITFnPtr = super::STypeFromSITFnPtr;
        pub type STypeFromSFieldFnPtr = super::STypeFromSFieldFnPtr;
        pub type OptionalSTVar;
        pub type OptionalUInt64;
        pub type ConstSLE;

        pub fn base64_decode_ptr(s: &CxxString) -> UniquePtr<CxxString>;

        pub fn fixMasterKeyAsRegularKey() -> &'static uint256;

        pub fn tfUniversalMask() -> u32;

        pub fn enabled(self: &Rules, s_field: &uint256) -> bool;
        pub fn getRules(self: &PreflightContext) -> &Rules;

        pub fn drops(self: &XRPAmount) -> i64;

        pub fn defaultCalculateBaseFee(view: &ReadView, tx: &STTx) -> XRPAmount;

        pub fn getTx(self: &PreflightContext) -> &STTx;

        // Note: getFlags is a method on STObject. In C++, we can call getFlags on anything
        // that extends STObject, but we can't call getFlags on an STTx in rust. We _could_
        // pass in self: &STTx here, but we'd have to duplicate this function for everything
        // we want to map that extends STObject. Instead, I defined an `upcast` function
        // in rippled_api.h that casts an STTx to an STObject, which I can call and then
        // use the returned STObject to call getFlags on.
        // See solution here: https://github.com/dtolnay/cxx/issues/797
        pub fn getFlags(self: &STObject) -> u32;

        pub fn isFieldPresent(self: &STObject, field: &SField) -> bool;
        pub fn isFlag(self: &SLE, f: u32) -> bool;
        pub fn isFlag(self: &STObject, f: u32) -> bool;

        pub fn getAccountID(self: &STObject, field: &SField) -> AccountID;
        pub fn getFieldH160(self: &STObject, field: &SField) -> uint160;
        pub fn getFieldU32(self: &STObject, field: &SField) -> u32;
        pub fn getFieldH256(self: &STObject, field: &SField) -> uint256;
        pub fn getFieldU8(self: &STObject, field: &SField) -> u8;
        pub fn getFieldU16(self: &STObject, field: &SField) -> u16;
        pub fn getFieldU64(self: &STObject, field: &SField) -> u64;
        pub fn getFieldBlob(self: &STObject, field: &SField) -> &'static STBlob;
        pub fn getFieldAmount(self: &STObject, field: &SField) -> &'static STAmount;
        pub fn getPluginType(self: &STObject, field: &SField) -> &'static STPluginType;

        pub fn getSeqProxy(self: &STTx) -> SeqProxy;

        pub fn setFlag(sle: &SharedPtr<SLE>, f: u32) -> bool;
        pub fn setAccountID(sle: &SharedPtr<SLE>, field: &SField, v: &AccountID);
        pub fn setFieldAmountXRP(sle: &SharedPtr<SLE>, field: &SField, v: &XRPAmount);
        pub fn setPluginType(sle: &SharedPtr<SLE>, field: &SField, v: &STPluginType);
        pub fn setFieldU8(sle: &SharedPtr<SLE>, field: &SField, v: u8);
        pub fn setFieldU16(sle: &SharedPtr<SLE>, field: &SField, v: u16);
        pub fn setFieldU32(sle: &SharedPtr<SLE>, field: &SField, v: u32);
        pub fn setFieldU64(sle: &SharedPtr<SLE>, field: &SField, v: u64);
        pub fn setFieldH160(sle: &SharedPtr<SLE>, field: &SField, v: &uint160);
        pub fn setFieldBlob(sle: &SharedPtr<SLE>, field: &SField, v: &STBlob);


        pub fn makeFieldAbsent(sle: &SharedPtr<SLE>, field: &SField);

        pub fn xrp(self: &STAmount) -> XRPAmount;
        pub fn negative(self: &STAmount) -> bool;
        pub fn is_zero(amount: &STAmount) -> bool;
        pub fn native(self: &STAmount) -> bool;
        pub fn st_amount_gt(amount1: &STAmount, amount2: &STAmount) -> bool;
        pub fn st_amount_eq(amount1: &STAmount, amount2: &STAmount) -> bool;

        pub fn sfRegularKey() -> &'static SField;
        pub fn sfAccount() -> &'static SField;
        pub fn sfSequence() -> &'static SField;
        pub fn sfOwnerCount() -> &'static SField;
        pub fn sfOwnerNode() -> &'static SField;
        pub fn sfBalance() -> &'static SField;
        pub fn sfFlags() -> &'static SField;
        pub fn sfIssuer() -> &'static SField;
        pub fn sfTransferFee() -> &'static SField;
        pub fn sfFee() -> &'static SField;
        pub fn sfAmount() -> &'static SField;
        pub fn sfInvoiceId() -> &'static SField;
        pub fn sfDestinationTag() -> &'static SField;
        pub fn sfDestination() -> &'static SField;

        // pub fn sfTicketSequence() -> &'static SField;
        pub fn getSField(type_id: i32, field_id: i32) -> &'static SField;

        pub fn getCode(self: &SField) -> i32;

        pub fn upcast(stTx: &STTx) -> &STObject;
        pub fn upcast_sle(sle: &SharedPtr<SLE>) -> SharedPtr<STObject>;

        pub fn getView(self: &PreclaimContext) -> &ReadView;
        pub fn getTx(self: &PreclaimContext) -> &STTx;

        pub fn exists(self: &ReadView, key: &Keylet) -> bool;
        pub fn read(read_view: &ReadView, key: &Keylet) -> SharedPtr<ConstSLE>;
        pub fn getFlags(self: &ConstSLE) -> u32;
        pub fn fees<'a, 'b>(self: &'a ReadView) -> &'b Fees;

        pub fn toBase58(v: &AccountID) -> UniquePtr<CxxString>;
        pub fn view<'a, 'b>(self: Pin<&'a mut ApplyContext>) -> Pin<&'b mut ApplyView>;
        pub fn getBaseFee<'a>(self: Pin<&'a mut ApplyContext>) -> XRPAmount;
        pub fn getTx<'a, 'b>(self: &'a ApplyContext) -> &'b STTx;
        pub fn getApp<'a, 'b>(self: Pin<&'a mut ApplyContext>) -> Pin<&'b mut Application>;
        pub fn getJournal<'a, 'b>(self: &'a ApplyContext) -> &'b Journal;

        pub fn peek(self: Pin<&mut ApplyView>, k: &Keylet) -> SharedPtr<SLE>;
        pub fn insert(self: Pin<&mut ApplyView>, sle: &SharedPtr<SLE>);
        pub fn update(self: Pin<&mut ApplyView>, sle: &SharedPtr<SLE>);
        pub fn dir_insert(apply_view: Pin<&mut ApplyView>, directory: &Keylet, key: &Keylet, account_id: &AccountID) -> UniquePtr<OptionalUInt64>;
        pub fn has_value(optional: &UniquePtr<OptionalUInt64>) -> bool;
        pub fn get_value(optional: &UniquePtr<OptionalUInt64>) -> u64;
        pub fn seq(self: &ApplyView) -> u32;
        pub fn adjustOwnerCount(view: Pin<&mut ApplyView>, sle: &SharedPtr<SLE>, amount: i32, j: &Journal);

        pub fn fees<'a, 'b>(self: &'a ApplyView) -> &'b Fees;
        pub fn flags(self: &ApplyView) -> ApplyFlags;

        pub fn accountReserve(self: &Fees, owner_count: usize) -> XRPAmount;

        // pub fn setFlag(self: Pin<&mut SLE>, f: u32) -> bool;

        pub fn minimumFee(app: Pin<&mut Application>, baseFee: XRPAmount, fees: &Fees, flags: ApplyFlags) -> XRPAmount;

        // Set
        //  flags (uint32),
        //  issuer (account),
        //  assetCode (uint160),
        //  assetScale (uint8),
        //  maximumAmount(uint64),
        //  outsandingAmount(uint64),
        //  lockedAmount(uin64)
        //  transferFee(uint16)
        //  metadata (blob),
        //  ownerNode (uint64)
        pub fn push_soelement(field_code: i32, style: SOEStyle, vec: Pin<&mut CxxVector<FakeSOElement>>);
        pub unsafe fn push_stype_export(
            tid: i32,
            create_new_sfield_fn: CreateNewSFieldPtr,
            parse_leaf_type_fn: ParseLeafTypeFnPtr,
            from_sit_constructor_ptr: STypeFromSITFnPtr,
            from_sfield_constructor_ptr: STypeFromSFieldFnPtr,
            vec: Pin<&mut CxxVector<STypeExport>>,
        );
        pub unsafe fn push_sfield_info(tid: i32, fv: i32, txt_name: *const c_char, vec: Pin<&mut CxxVector<SFieldInfo>>);

        pub fn isString(self: &Value) -> bool;
        pub fn asString(json_value: &Value) -> UniquePtr<CxxString>;
        pub fn make_empty_stvar_opt() -> UniquePtr<OptionalSTVar>;
        pub fn make_stvar(field: &SField, data: &[u8]) -> UniquePtr<OptionalSTVar>;
        pub fn bad_type(error: Pin<&mut Value>, json_name: &CxxString, field_name: &CxxString);
        pub fn invalid_data(error: Pin<&mut Value>, json_name: &CxxString, field_name: &CxxString);
        pub fn getVLBuffer(sit: Pin<&mut SerialIter>) -> UniquePtr<Buffer>;
        pub fn make_stype(field: &SField, buffer: UniquePtr<Buffer>) -> UniquePtr<STPluginType>;
        pub fn make_empty_stype(field: &SField) -> UniquePtr<STBase>;
        pub unsafe fn constructSField(tid: i32, fv: i32, fname: *const c_char) -> &'static SField;

        pub unsafe fn data(self: &STPluginType) -> *const u8;
        pub fn size(self: &STPluginType) -> usize;

        pub unsafe fn data(self: &STBlob) -> *const u8;
        pub fn size(self: &STBlob) -> usize;

        pub fn new_sle(keylet: &Keylet) -> SharedPtr<SLE>;

        // FIXME: Probably a memory leak
        pub unsafe fn new_st_blob(sfield: &SField, data: *const u8, size: usize) -> &STBlob;
    }

}

#[repr(transparent)]
pub struct CreateNewSFieldPtr(
    pub extern "C" fn(
        tid: i32,
        fv: i32,
        field_name: *const c_char,
    ) -> &'static SField
);

unsafe impl ExternType for CreateNewSFieldPtr {
    type Id = type_id!("CreateNewSFieldPtr");
    type Kind = Trivial;
}

// https://github.com/dtolnay/cxx/issues/895#issuecomment-913095541
#[repr(transparent)]
pub struct ParseLeafTypeFnPtr(
    pub extern "C" fn(
        field: &SField,
        json_name: &CxxString,
        field_name: &CxxString,
        name: &SField,
        value: &Value,
        error: Pin<&mut Value>,
    ) -> *mut OptionalSTVar
);

unsafe impl ExternType for ParseLeafTypeFnPtr {
    type Id = type_id!("ParseLeafTypeFnPtr");
    type Kind = Trivial;
}

#[repr(transparent)]
pub struct STypeFromSITFnPtr(
    pub extern "C" fn(
        sit: Pin<&mut SerialIter>,
        name: &SField,
    ) -> *mut STPluginType
);

unsafe impl ExternType for STypeFromSITFnPtr {
    type Id = type_id!("STypeFromSITFnPtr");
    type Kind = Trivial;
}

#[repr(transparent)]
pub struct STypeFromSFieldFnPtr(
    pub extern "C" fn(
        name: &SField
    ) -> *mut STBase
);

unsafe impl ExternType for STypeFromSFieldFnPtr {
    type Id = type_id!("STypeFromSFieldFnPtr");
    type Kind = Trivial;
}

#[repr(C)]
#[derive(PartialEq)]
pub struct AccountID {
    data_: [u8; 20],
}

impl From<AccountID> for AccountId {
    fn from(value: AccountID) -> Self {
        AccountId::from(value.data_)
    }
}

impl From<&AccountId> for AccountID {
    fn from(value: &AccountId) -> Self {
        AccountID {
            data_: value.as_ref().try_into().unwrap()
        }
    }
}

impl From<AccountId> for AccountID {
    fn from(value: AccountId) -> Self {
        AccountID {
            data_: value.as_ref().try_into().unwrap()
        }
    }
}

unsafe impl cxx::ExternType for AccountID {
    type Id = type_id!("ripple::AccountID");
    type Kind = Trivial;
}

#[repr(C)]
#[derive(PartialEq)]
pub struct XRPAmount {
    drops_: i64,
}

impl XRPAmount {
    pub fn zero() -> Self {
        XRPAmount { drops_: 0 }
    }
}

impl From<XrpAmount> for XRPAmount {
    fn from(value: XrpAmount) -> Self {
        XRPAmount {
            drops_: value.get_drops() as i64
        }
    }
}

impl From<XRPAmount> for XrpAmount {
    fn from(value: XRPAmount) -> Self {
        XrpAmount::of_drops(value.drops_ as u64).unwrap()
    }
}

unsafe impl cxx::ExternType for XRPAmount {
    type Id = type_id!("ripple::XRPAmount");
    type Kind = Trivial;
}

#[repr(i16)]
#[derive(Clone, Copy)]
pub enum LedgerEntryType {
    /// A ledger object which describes an account.
    /// \sa keylet::account

    ltACCOUNT_ROOT = 0x0061,

    /// A ledger object which contains a list of object identifiers.
    ///       \sa keylet::page, keylet::quality, keylet::book, keylet::next and
    ///           keylet::ownerDir

    ltDIR_NODE = 0x0064,

    /// A ledger object which describes a bidirectional trust line.
    ///       @note Per Vinnie Falco this should be renamed to ltTRUST_LINE
    ///       \sa keylet::line

    ltRIPPLE_STATE = 0x0072,

    /// A ledger object which describes a ticket.

    ///    \sa keylet::ticket
    ltTICKET = 0x0054,

    /// A ledger object which contains a signer list for an account.
    /// \sa keylet::signers
    ltSIGNER_LIST = 0x0053,

    /// A ledger object which describes an offer on the DEX.
    /// \sa keylet::offer
    ltOFFER = 0x006f,

    /// A ledger object that contains a list of ledger hashes.
    ///
    ///       This type is used to store the ledger hashes which the protocol uses
    ///       to implement skip lists that allow for efficient backwards (and, in
    ///       theory, forward) forward iteration across large ledger ranges.
    ///       \sa keylet::skip
    ltLEDGER_HASHES = 0x0068,

    /// The ledger object which lists details about amendments on the network.
    ///       \note This is a singleton: only one such object exists in the ledger.
    ///
    ///       \sa keylet::amendments
    ltAMENDMENTS = 0x0066,

    /// The ledger object which lists the network's fee settings.
    ///
    ///       \note This is a singleton: only one such object exists in the ledger.
    ///
    ///       \sa keylet::fees
    ltFEE_SETTINGS = 0x0073,

    /// A ledger object describing a single escrow.
    ///
    ///       \sa keylet::escrow
    ltESCROW = 0x0075,

    /// A ledger object describing a single unidirectional XRP payment channel.
    ///
    ///       \sa keylet::payChan
    ltPAYCHAN = 0x0078,

    /// A ledger object which describes a check.
    ///       \sa keylet::check
    ltCHECK = 0x0043,

    /// A ledger object which describes a deposit preauthorization.
    ///
    ///       \sa keylet::depositPreauth
    ltDEPOSIT_PREAUTH = 0x0070,

    /// The ledger object which tracks the current negative UNL state.
    ///
    ///       \note This is a singleton: only one such object exists in the ledger.
    ///
    ///       \sa keylet::negativeUNL

    ltNEGATIVE_UNL = 0x004e,

    /// A ledger object which contains a list of NFTs
    ///       \sa keylet::nftpage_min, keylet::nftpage_max, keylet::nftpage
    ltNFTOKEN_PAGE = 0x0050,

    /// A ledger object which identifies an offer to buy or sell an NFT.
    ///       \sa keylet::nftoffer
    ltNFTOKEN_OFFER = 0x0037,

    //---------------------------------------------------------------------------
    /// A special type, matching any ledger entry type.
    ///
    ///       The value does not represent a concrete type, but rather is used in
    ///       contexts where the specific type of a ledger object is unimportant,
    ///       unknown or unavailable.
    ///
    ///      Objects with this special type cannot be created or stored on the
    ///      ledger.
    ///
    ///      \sa keylet::unchecked
    ltANY = 0,

    /// A special type, matching any ledger type except directory nodes.
    ///
    ///      The value does not represent a concrete type, but rather is used in
    ///       contexts where the ledger object must not be a directory node but
    ///       its specific type is otherwise unimportant, unknown or unavailable.
    ///
    ///      Objects with this special type cannot be created or stored on the
    ///     ledger.
    ///
    ///      \sa keylet::child
    ltCHILD = 0x1CD2,

    //---------------------------------------------------------------------------
    /// A legacy, deprecated type.
    ///
    ///      \deprecated **This object type is not supported and should not be used.**
    ///                   Support for this type of object was never implemented.
    ///                  No objects of this type were ever created.
    ltNICKNAME = 0x006e,

    /// A legacy, deprecated type.
    ///
    ///   \deprecated **This object type is not supported and should not be used.**
    ///               Support for this type of object was never implemented.
    ///               No objects of this type were ever created.
    ltCONTRACT = 0x0063,

    /// A legacy, deprecated type.
    ///
    ///   \deprecated **This object type is not supported and should not be used.**
    ///             Support for this type of object was never implemented.
    ///            No objects of this type were ever created.
    ltGENERATOR_MAP = 0x0067,
}

impl From<LedgerEntryType> for i16 {
    fn from(value: LedgerEntryType) -> Self {
        value as i16
    }
}

unsafe impl cxx::ExternType for LedgerEntryType {
    type Id = type_id!("ripple::LedgerEntryType");
    type Kind = Trivial;
}

pub struct KeyletBuilder {
    ledger_entry_type: i16,
    namespace: u16,
    key_bytes: Vec<u8>
}

impl KeyletBuilder {
    pub fn new<L: Into<i16>, NS: Into<u16>>(ledger_entry_type: L, namespace: NS) -> Self {
        KeyletBuilder {
            ledger_entry_type: ledger_entry_type.into(),
            namespace: namespace.into(),
            key_bytes: Vec::new()
        }
    }

    pub fn key<T: AsRef<[u8]>>(mut self, key: T) -> Self {
        self.key_bytes.extend_from_slice(key.as_ref());
        self
    }

    pub fn build(self) -> Keylet {
        Keylet::new(self.ledger_entry_type, self.namespace, self.key_bytes)
    }
}

#[repr(C)]
pub struct Keylet {
    key: [u8; 32],
    pub r#type: i16,
}

impl Keylet {
    pub fn account(account_id: &AccountId) -> Self {
        rippled::account(&account_id.into())
    }

    pub fn signers(account_id: &AccountId) -> Self {
        rippled::signers(&account_id.into())
    }

    pub fn owner_dir(account_id: &AccountId) -> Self {
        rippled::ownerDir(&account_id.into())
    }
    pub fn builder<L: Into<i16>, NS: Into<u16>>(ledger_entry_type: L, namespace: NS) -> KeyletBuilder {
        KeyletBuilder::new(ledger_entry_type, namespace)
    }

    fn new(ledger_entry_type: i16, namespace: u16, bytes: Vec<u8>) -> Self {
        let mut sha512 = Sha512::new();
        sha512.update(namespace.to_be_bytes());
        sha512.update(&bytes);
        Keylet {
            key: sha512.finalize()[..32].try_into().unwrap(),
            r#type: ledger_entry_type,
        }
    }
}

impl From<Keylet> for Hash256 {
    fn from(value: Keylet) -> Self {
        Hash256::from(value.key)
    }
}

#[test]
fn test_new_keylet() {
    let account_k_1 = Keylet::builder(LedgerEntryType::ltACCOUNT_ROOT, LedgerNameSpace::Account)
        .key(ACCOUNT_ONE)
        .build();
    let account_k_2 = Keylet::account(&ACCOUNT_ONE);

    assert_eq!(account_k_1.key, account_k_2.key)
}

unsafe impl cxx::ExternType for Keylet {
    type Id = type_id!("ripple::Keylet");
    type Kind = Trivial;
}

#[repr(u16)]
pub enum LedgerNameSpace {
    Account,
    DirNode,
    OwnerDir,
    SkipList,
    Amendments,
    FeeSettings,
    SignerList,
    NegativeUnl,
}

impl From<LedgerNameSpace> for u16 {
    fn from(value: LedgerNameSpace) -> Self {
        let c = match value {
            LedgerNameSpace::Account => 'a',
            LedgerNameSpace::DirNode => 'd',
            LedgerNameSpace::OwnerDir => 'O',
            LedgerNameSpace::SkipList => 's',
            LedgerNameSpace::Amendments => 'f',
            LedgerNameSpace::FeeSettings => 'e',
            LedgerNameSpace::SignerList => 'S',
            LedgerNameSpace::NegativeUnl => 'N'
        };

        c as u16
    }
}

#[repr(C)]
pub struct FakeSOElement2 {
    pub field_code: i32,
    pub style: SOEStyle,
}

unsafe impl cxx::ExternType for FakeSOElement2 {
    type Id = type_id!("ripple::FakeSOElement2");
    type Kind = Trivial;
}

#[repr(C)]
#[derive(PartialEq, Clone, Copy)]
pub enum SOEStyle {
    soeINVALID = -1,
    soeREQUIRED = 0,
    // required
    soeOPTIONAL = 1,
    // optional, may be present with default value
    soeDEFAULT = 2,   // optional, if present, must not have default value
}

unsafe impl cxx::ExternType for SOEStyle {
    type Id = type_id!("ripple::SOEStyle");
    type Kind = Trivial;
}

#[repr(C)]
pub struct UInt160 {
    data: [u8; 20]
}

impl From<UInt160> for Hash160 {
    fn from(value: UInt160) -> Self {
        Hash160::try_from(value.data.as_ref()).unwrap()
    }
}

impl From<Hash160> for UInt160 {
    fn from(value: Hash160) -> Self {
        UInt160 {
            data: value.as_ref().try_into().unwrap()
        }
    }
}

impl From<&Hash160> for UInt160 {
    fn from(value: &Hash160) -> Self {
        UInt160 {
            data: value.as_ref().try_into().unwrap()
        }
    }
}

unsafe impl cxx::ExternType for UInt160 {
    type Id = type_id!("ripple::uint160");
    type Kind = Trivial;
}

#[repr(C)]
pub struct UInt256 {
    data: [u8; 32]
}

impl From<UInt256> for Hash256 {
    fn from(value: UInt256) -> Self {
        Hash256::try_from(value.data.as_ref()).unwrap()
    }
}

impl From<Hash256> for UInt256 {
    fn from(value: Hash256) -> Self {
        UInt256 {
            data: value.as_ref().try_into().unwrap()
        }
    }
}

impl From<&Hash256> for UInt256 {
    fn from(value: &Hash256) -> Self {
        UInt256 {
            data: value.as_ref().try_into().unwrap()
        }
    }
}

unsafe impl cxx::ExternType for UInt256 {
    type Id = type_id!("ripple::uint256");
    type Kind = Trivial;
}