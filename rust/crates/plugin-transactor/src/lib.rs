extern crate core;

pub mod transactor;

use core::slice;
use std::cmp::Ordering;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use cxx::{CxxVector, SharedPtr, UniquePtr};
pub use transactor::Transactor;

use xrpl_rust_sdk_core::core::types::{AccountId, Currency, Hash160, Hash256, XrpAmount};
use rippled_bridge::{AccountID, ApplyFlags, Keylet, LedgerSpecificFlags, NotTEC, UInt160, UInt256, XRPAmount};
use rippled_bridge::rippled::{OptionalUInt64, setFlag};
use rippled_bridge::TEScodes::tesSUCCESS;
use rippled_bridge::tx_consequences::SeqProxy;
use rippled_bridge::type_ids::SerializedTypeID;
use crate::transactor::{AsSTObject, ConstLedgerObject, LedgerObject};

pub struct PreflightContext<'a> {
    instance: &'a rippled_bridge::rippled::PreflightContext,
}

impl PreflightContext<'_> {
    pub fn new(instance: &rippled_bridge::rippled::PreflightContext) -> PreflightContext {
        PreflightContext { instance }
    }

    pub fn rules(&self) -> Rules {
        Rules::new(self.instance.getRules())
    }

    pub fn tx(&self) -> STTx {
        STTx::new(self.instance.getTx())
    }
}

pub struct PreclaimContext<'a> {
    instance: &'a rippled_bridge::rippled::PreclaimContext,
    pub view: ReadView<'a>,
    pub tx: STTx<'a>,
}

impl PreclaimContext<'_> {
    pub fn new(instance: &rippled_bridge::rippled::PreclaimContext) -> PreclaimContext {
        PreclaimContext {
            instance,
            view: ReadView::new(instance.getView()),
            tx: STTx::new(instance.getTx()),
        }
    }
}

pub struct STTx<'a> {
    instance: &'a rippled_bridge::rippled::STTx,
}

impl STTx<'_> {
    pub fn new(instance: &rippled_bridge::rippled::STTx) -> STTx {
        STTx { instance }
    }

    pub fn flags(&self) -> u32 {
        rippled_bridge::rippled::upcast(self.instance).getFlags()
    }

    pub fn get_account_id(&self, field: &SField) -> AccountId {
        self.as_st_object().getAccountID(field.instance).into()
    }

    pub fn get_uint160(&self, field: &SField) -> Hash160 {
        self.as_st_object().getFieldH160(field.instance).into()
    }

    pub fn get_amount(&self, field: &SField) -> STAmount {
        STAmount::new(self.as_st_object().deref().getFieldAmount(field.instance))
    }

    pub fn get_u8(&self, field: &SField) -> u8 {
        self.as_st_object().getFieldU8(field.instance)
    }

    pub fn get_u16(&self, field: &SField) -> u16 {
        self.as_st_object().getFieldU16(field.instance)
    }

    pub fn get_u32(&self, field: &SField) -> u32 {
        self.as_st_object().getFieldU32(field.instance)
    }

    pub fn get_u64(&self, field: &SField) -> u64 {
        self.as_st_object().getFieldU64(field.instance)
    }

    pub fn get_blob(&self, field: &SField) -> STBlob {
        STBlob::new(self.as_st_object().getFieldBlob(field.instance))
    }

    pub fn get_plugin_type(&self, field: &SField) -> STPluginType {
        STPluginType::new(self.as_st_object().getPluginType(field.instance))
    }

    pub fn is_field_present(&self, field: &SField) -> bool {
        self.as_st_object().isFieldPresent(field.instance)
    }

    pub fn seq_proxy(&self) -> SeqProxy {
        self.instance.getSeqProxy()
    }

    fn as_st_object(&self) -> &rippled_bridge::rippled::STObject {
        rippled_bridge::rippled::upcast(self.instance)
    }
}

pub struct STPluginType<'a> {
    instance: &'a rippled_bridge::rippled::STPluginType,
}

impl AsRef<[u8]> for STPluginType<'_> {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let data: *const u8 = self.instance.data();
            let size: usize = self.instance.size();
            slice::from_raw_parts(data, size)
        }
    }
}

impl<T> PartialEq<T> for STPluginType<'_> where T: AsRef<[u8]> {
    fn eq(&self, other: &T) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl STPluginType<'_> {
    pub(crate) fn new(instance: &rippled_bridge::rippled::STPluginType) -> STPluginType {
        STPluginType { instance }
    }
}

pub struct SField<'a> {
    instance: &'a rippled_bridge::rippled::SField,
}

impl SField<'_> {
    pub fn sf_regular_key() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfRegularKey()
        }
    }

    pub fn sf_account() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfAccount()
        }
    }

    pub fn sf_sequence() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfSequence()
        }
    }

    pub fn sf_owner_count() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfOwnerCount()
        }
    }

    pub fn sf_owner_node() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfOwnerNode()
        }
    }

    pub fn sf_balance() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfBalance()
        }
    }

    pub fn sf_flags() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfFlags()
        }
    }

    pub fn sf_issuer() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfIssuer()
        }
    }

    pub fn sf_transfer_fee() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfTransferFee()
        }
    }

    pub fn sf_fee() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfFee()
        }
    }

    pub fn sf_amount() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfAmount()
        }
    }

    pub fn sf_invoice_id() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfInvoiceId()
        }
    }

    pub fn sf_destination() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfDestination()
        }
    }

    pub fn sf_destination_tag() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfDestinationTag()
        }
    }

    pub fn sf_previous_page_min() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfPreviousPageMin()
        }
    }

    pub fn sf_next_page_min() -> Self {
        SField {
            instance: rippled_bridge::rippled::sfNextPageMin()
        }
    }

    pub fn get_plugin_field<T: Into<i32>>(type_id: T, field_id: i32) -> Self {
        SField {
            instance: rippled_bridge::rippled::getSField(type_id.into(), field_id)
        }
    }

    pub fn code(&self) -> i32 {
        self.instance.getCode()
    }
}

pub struct Rules<'a> {
    instance: &'a rippled_bridge::rippled::Rules,
}

impl Rules<'_> {
    pub(crate) fn new(instance: &rippled_bridge::rippled::Rules) -> Rules {
        Rules { instance }
    }

    pub fn enabled(&self, feature: &Feature) -> bool {
        self.instance.enabled(feature.instance)
    }
}

pub struct Feature<'a> {
    instance: &'a rippled_bridge::rippled::uint256,
}

impl Feature<'_> {
    pub fn fix_master_key_as_regular_key() -> Self {
        Feature {
            instance: rippled_bridge::rippled::fixMasterKeyAsRegularKey()
        }
    }
}

pub struct ReadView<'a> {
    instance: &'a rippled_bridge::rippled::ReadView,
}

impl<'a> ReadView<'a> {
    pub fn new(instance: &rippled_bridge::rippled::ReadView) -> ReadView {
        ReadView { instance }
    }

    pub(crate) fn instance(&self) -> &'a rippled_bridge::rippled::ReadView {
        self.instance
    }

    pub fn exists(&self, key: &Keylet) -> bool {
        self.instance.exists(key)
    }

    pub fn read(&self, key: &Keylet) -> Option<ConstSLE> {
        let maybe_sle = rippled_bridge::rippled::read(self.instance, key);
        if maybe_sle.is_null() {
            None
        } else {
            Some(ConstSLE::new(maybe_sle))
        }
    }

    pub fn read_typed<T: ConstLedgerObject>(&self, key: &Keylet) -> Option<T> {
        self.read(key)
            .map(|sle| T::from(sle))
    }

    pub fn succ(&self, key: &Keylet, last: &Keylet) -> Option<UInt256> {
        let result = rippled_bridge::rippled::read_view_succ(self.instance, key, last);
        if rippled_bridge::rippled::opt_uint256_has_value(&result) {
            Some(rippled_bridge::rippled::opt_uint256_get_value(&result))
        } else {
            None
        }
    }

    pub fn fees(&self) -> Fees {
        Fees::new(self.instance.fees())
    }
}

pub struct Fees<'a> {
    instance: &'a rippled_bridge::rippled::Fees,
}

impl Fees<'_> {
    pub fn new(instance: &rippled_bridge::rippled::Fees) -> Fees {
        Fees { instance }
    }

    pub fn account_reserve(&self, owner_count: u32) -> XrpAmount {
        self.instance.accountReserve(owner_count as usize).into()
    }
}

pub struct STAmount<'a> {
    instance: &'a rippled_bridge::rippled::STAmount,
}

impl STAmount<'_> {
    pub(crate) fn new(instance: &rippled_bridge::rippled::STAmount) -> STAmount {
        STAmount {
            instance
        }
    }

    pub fn negative(&self) -> bool {
        self.instance.negative()
    }

    pub fn xrp(&self) -> XrpAmount {
        self.instance.xrp().into()
    }

    pub fn is_zero(&self) -> bool {
        rippled_bridge::rippled::is_zero(self.instance)
    }

    pub fn native(&self) -> bool {
        self.instance.native()
    }

    pub fn is_cft(&self) -> bool {
        self.instance.isCft()
    }

    pub fn mantissa(&self) -> u64 {
        self.instance.mantissa()
    }

    pub fn issuer(&self) -> AccountId {
        AccountId::from(self.instance.getIssuer())
    }

    pub fn currency(&self) -> Currency {
        Currency::from(self.instance.getCurrency())
    }
}

impl PartialEq<Self> for STAmount<'_> {
    fn eq(&self, other: &Self) -> bool {
        rippled_bridge::rippled::st_amount_eq(self.instance, other.instance)
    }
}

impl PartialOrd for STAmount<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else if rippled_bridge::rippled::st_amount_gt(self.instance, other.instance) {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Less)
        }
    }
}

pub struct STBlob<'a> {
    instance: &'a rippled_bridge::rippled::STBlob,
}

impl STBlob<'_> {
    pub fn new(instance: &rippled_bridge::rippled::STBlob) -> STBlob {
        STBlob { instance }
    }

    pub fn from_slice<'a>(field: &'a SField, slice: &'a [u8]) -> STBlob<'a> {
        unsafe {
            STBlob::new(rippled_bridge::rippled::new_st_blob(field.instance, slice.as_ptr(), slice.len()))
        }
    }
}

impl AsRef<[u8]> for STBlob<'_> {
    fn as_ref(&self) -> &[u8] {
        unsafe {
            let data: *const u8 = self.instance.data();
            let size: usize = self.instance.size();
            slice::from_raw_parts(data, size)
        }
    }
}

pub struct ConstSLE {
    instance: SharedPtr<rippled_bridge::rippled::ConstSLE>,
}

impl ConstSLE {
    pub fn new(instance: SharedPtr<rippled_bridge::rippled::ConstSLE>) -> Self {
        ConstSLE { instance }
    }

    pub fn get_account_id(&self, field: &SField) -> AccountId {
        self.as_st_object().deref().getAccountID(field.instance).into()
    }

    pub fn get_field_uint32(&self, field: &SField) -> u32 {
        self.as_st_object().deref().getFieldU32(field.instance)
    }

    pub fn get_field_uint64(&self, field: &SField) -> u64 {
        self.as_st_object().deref().getFieldU64(field.instance)
    }

    pub fn get_field_amount(&self, field: &SField) -> STAmount {
        STAmount::new(self.as_st_object().deref().getFieldAmount(field.instance))
    }

    pub fn get_field_array(&self, field: &SField) -> ConstSTArray {
        ConstSTArray::new(self.as_st_object().getFieldArray(field.instance))
    }

    pub fn is_field_present(&self, field: &SField) -> bool {
        self.as_st_object().isFieldPresent(field.instance)
    }

    pub fn get_field_h256(&self, field: &SField) -> Hash256 {
        self.as_st_object().deref().getFieldH256(field.instance).into()
    }

    pub fn flags(&self) -> u32 {
        self.instance.getFlags()
    }

    fn as_st_object(&self) -> &rippled_bridge::rippled::STObject {
        rippled_bridge::rippled::upcast_const_sle(&self.instance)
    }
}

pub struct SLE {
    instance: SharedPtr<rippled_bridge::rippled::SLE>,
}

impl SLE {
    pub fn new(instance: SharedPtr<rippled_bridge::rippled::SLE>) -> Self {
        SLE { instance }
    }

    pub fn get_account_id(&self, field: &SField) -> AccountId {
        self.as_st_object().deref().getAccountID(field.instance).into()
    }

    pub fn get_field_uint32(&self, field: &SField) -> u32 {
        self.as_st_object().deref().getFieldU32(field.instance)
    }

    pub fn get_field_uint64(&self, field: &SField) -> u64 {
        self.as_st_object().deref().getFieldU64(field.instance)
    }

    pub fn get_field_h256(&self, field: &SField) -> Hash256 {
        self.as_st_object().deref().getFieldH256(field.instance).into()
    }

    pub fn get_field_amount(&self, field: &SField) -> STAmount {
        STAmount::new(self.as_st_object().deref().getFieldAmount(field.instance))
    }

    pub fn peek_field_array(&self, field: &SField) -> STArray {
        STArray::new(rippled_bridge::rippled::peekFieldArray(Pin::new(&mut self.as_st_object()), field.instance))
    }

    pub fn make_field_absent(&self, field: &SField) {
        rippled_bridge::rippled::makeFieldAbsent(&self.instance, field.instance);
    }

    pub fn is_flag(&self, flag: LedgerSpecificFlags) -> bool {
        self.instance.deref().isFlag(flag.into())
    }

    pub fn set_flag(&self, flag: LedgerSpecificFlags) {
        setFlag(&self.instance, flag.into());
    }

    pub fn set_field_u8(&mut self, sfield: &SField, value: u8) {
        rippled_bridge::rippled::setFieldU8(&self.instance, sfield.instance, value);
    }

    pub fn set_field_u16(&mut self, sfield: &SField, value: u16) {
        rippled_bridge::rippled::setFieldU16(&self.instance, sfield.instance, value);
    }

    pub fn set_field_u32(&mut self, sfield: &SField, value: u32) {
        rippled_bridge::rippled::setFieldU32(&self.instance, sfield.instance, value);
    }

    pub fn set_field_u64(&mut self, sfield: &SField, value: u64) {
        rippled_bridge::rippled::setFieldU64(&self.instance, sfield.instance, value);
    }

    pub fn set_field_h160(&mut self, sfield: &SField, value: &Hash160) {
        rippled_bridge::rippled::setFieldH160(&self.instance, sfield.instance, &UInt160::from(value));
    }

    pub fn set_field_h256(&mut self, sfield: &SField, value: &Hash256) {
        rippled_bridge::rippled::setFieldH256(&self.instance, sfield.instance, &UInt256::from(value));
    }

    pub fn set_field_account(&mut self, sfield: &SField, value: &AccountId) {
        rippled_bridge::rippled::setAccountID(&mut self.instance, sfield.instance, &AccountID::from(value));
    }

    pub fn set_field_amount_xrp(&self, sfield: &SField, value: XrpAmount) {
        rippled_bridge::rippled::setFieldAmountXRP(&self.instance, sfield.instance, &XRPAmount::from(value));
    }

    pub fn set_field_blob(&mut self, sfield: &SField, value: &STBlob) {
        rippled_bridge::rippled::setFieldBlob(&self.instance, sfield.instance, value.instance);
    }

    pub fn set_field_blob2(&mut self, sfield: &SField, value: &[u8]) {
        rippled_bridge::rippled::setFieldBlob(&self.instance, sfield.instance, &STBlob::from_slice(sfield, value).instance);
    }

    pub fn set_plugin_type(&self, field: &SField, value: &STPluginType) {
        rippled_bridge::rippled::setPluginType(&self.instance, field.instance, value.instance);
    }

    pub fn set_field_array(&mut self, field: &SField, value: STArray) {
        match value {
            STArray::UniquePtr(up) => rippled_bridge::rippled::setFieldArray(&self.instance, field.instance, up.deref()),
            STArray::Pin(p) => rippled_bridge::rippled::setFieldArray(&self.instance, field.instance, &p)
        }

    }

    pub fn is_field_present(&self, field: &SField) -> bool {
        self.as_st_object().isFieldPresent(field.instance)
    }

    fn as_st_object(&self) -> SharedPtr<rippled_bridge::rippled::STObject> {
        rippled_bridge::rippled::upcast_sle(&self.instance)
    }
}

impl From<&Keylet> for SLE {
    fn from(value: &Keylet) -> Self {
        SLE::new(
            rippled_bridge::rippled::new_sle(value)
        )
    }
}

pub enum STObject<'a> {
    UniquePtr(UniquePtr<rippled_bridge::rippled::STObject>),
    Pin(Pin<&'a mut rippled_bridge::rippled::STObject>)
}

impl<'a> AsRef<rippled_bridge::rippled::STObject> for STObject<'a> {
    fn as_ref(&self) -> &rippled_bridge::rippled::STObject {
        match self {
            STObject::UniquePtr(up) => up.deref(),
            STObject::Pin(p) => p.deref()
        }
    }
}

impl<'a> STObject<'a> {
    pub fn new(instance: UniquePtr<rippled_bridge::rippled::STObject>) -> STObject<'a> {
        STObject::UniquePtr(instance)
    }

    pub fn new_inner(field: SField) -> STObject {
        STObject::UniquePtr(rippled_bridge::rippled::create_inner_object(field.instance))
    }

    pub fn get_account_id(&self, field: &SField) -> AccountId {
        match self {
            STObject::UniquePtr(up) => up.getAccountID(field.instance).into(),
            STObject::Pin(p) => p.deref().getAccountID(field.instance).into()
        }
    }

    pub fn get_field_uint32(&self, field: &SField) -> u32 {
        match self {
            STObject::UniquePtr(up) => up.getFieldU32(field.instance).into(),
            STObject::Pin(p) => p.deref().getFieldU32(field.instance).into()
        }
    }

    pub fn get_field_uint64(&self, field: &SField) -> u64 {
        match self {
            STObject::UniquePtr(up) => up.getFieldU64(field.instance).into(),
            STObject::Pin(p) => p.deref().getFieldU64(field.instance).into()
        }
    }

    pub fn get_field_h256(&self, field: &SField) -> Hash256 {
        match self {
            STObject::UniquePtr(up) => up.getFieldH256(field.instance).into(),
            STObject::Pin(p) => {
                p.getFieldH256(field.instance).into()
            }
        }
    }

    pub fn get_field_amount(&self, field: &SField) -> STAmount {
        STAmount::new(match self {
            STObject::UniquePtr(up) => up.getFieldAmount(field.instance).into(),
            STObject::Pin(p) => p.deref().getFieldAmount(field.instance).into()
        })
    }

    pub fn set_field_u32(&mut self, sfield: &SField, value: u32) {
        match self {
            STObject::UniquePtr(up) => up.pin_mut().setFieldU32(sfield.instance, value),
            STObject::Pin(p) => p.as_mut().setFieldU32(sfield.instance, value)
        }
    }

    pub fn set_field_u64(&mut self, sfield: &SField, value: u64) {
        match self {
            STObject::UniquePtr(up) => up.pin_mut().setFieldU64(sfield.instance, value),
            STObject::Pin(p) => p.as_mut().setFieldU64(sfield.instance, value)
        }
    }

    pub fn set_field_h256(&mut self, sfield: &SField, value: &Hash256) {
        match self {
            STObject::UniquePtr(up) => up.pin_mut().setFieldH256(sfield.instance, &UInt256::from(value)),
            STObject::Pin(p) => p.as_mut().setFieldH256(sfield.instance, &UInt256::from(value))
        }
    }

    pub fn is_flag(&self, flag: LedgerSpecificFlags) -> bool {
        match self {
            STObject::UniquePtr(up) => up.isFlag(flag.into()).into(),
            STObject::Pin(p) => p.isFlag(flag.into()).into()
        }
    }


}

pub struct ConstSTObject<'a> {
    instance: &'a rippled_bridge::rippled::STObject,
}

impl <'a> ConstSTObject<'a> {
    pub fn new(instance: &rippled_bridge::rippled::STObject) -> ConstSTObject {
        ConstSTObject { instance }
    }

    pub fn get_account_id(&self, field: &SField) -> AccountId {
        self.instance.getAccountID(field.instance).into()
    }

    pub fn get_uint32(&self, field: &SField) -> u32 {
        self.instance.getFieldU32(field.instance)
    }

    pub fn get_uint64(&self, field: &SField) -> u64 {
        self.instance.getFieldU64(field.instance)
    }

    pub fn get_h256(&self, field: &SField) -> Hash256 {
        self.instance.getFieldH256(field.instance).into()
    }

    pub fn get_amount(&self, field: &SField) -> STAmount {
        STAmount::new(self.instance.getFieldAmount(field.instance))
    }

    pub fn is_flag(&self, flag: LedgerSpecificFlags) -> bool {
        self.instance.isFlag(flag.into())
    }
}

pub struct ConstSTArray<'a> {
    instance: &'a rippled_bridge::rippled::STArray
}

impl <'a>ConstSTArray<'a> {
    pub fn new(instance: &'a rippled_bridge::rippled::STArray) -> ConstSTArray<'a> {
        ConstSTArray { instance }
    }

    pub fn size(&self) -> usize {
        self.instance.size()
    }

    pub fn get(&self, index: usize) -> Option<ConstSTObject<'static>> {
        if index > self.size() - 1 {
            None
        } else {
            Some(ConstSTObject::new(rippled_bridge::rippled::get_from_const_st_array(self.instance, index)))
        }
    }
}

pub enum STArray<'a> {
    UniquePtr(UniquePtr<rippled_bridge::rippled::STArray>),
    Pin(Pin<&'a mut rippled_bridge::rippled::STArray>),
}

impl <'a> STArray<'a> {
    pub fn new_empty() -> STArray<'a> {
        STArray::UniquePtr(rippled_bridge::rippled::new_st_array())
    }

    pub fn new(instance: Pin<&'a mut rippled_bridge::rippled::STArray>) -> STArray<'a> {
        Self::Pin(instance)
    }

    pub fn size(&self) -> usize {
        match self {
            STArray::UniquePtr(up) => up.size(),
            STArray::Pin(p) => p.deref().size()
        }
    }

    pub fn get<'b, 'c>(&'c mut self, index: usize) -> Option<STObject<'b>> {
        if index > self.size() - 1 {
            None
        } else {
            let pinned_self = match self {
                STArray::UniquePtr(up) => up.pin_mut(),
                STArray::Pin(p) => p.as_mut()
            };

            Some(STObject::Pin(rippled_bridge::rippled::get_from_st_array(pinned_self, index)))
        }
    }

    pub fn push_back<T: AsRef<STObject<'a>>>(&mut self, elem: &T) {
        let pinned_self = match self {
            STArray::UniquePtr(up) => up.pin_mut(),
            STArray::Pin(p) => p.as_mut()
        };
        pinned_self.push_back(&elem.as_ref().as_ref())
    }
}

pub struct ApplyView<'a> {
    instance: Pin<&'a mut rippled_bridge::rippled::ApplyView>,
    fees: Fees<'a>,
    flags: ApplyFlags,
}

impl ApplyView<'_> {
    pub fn new(mut instance: Pin<&mut rippled_bridge::rippled::ApplyView>) -> ApplyView {
        let fees = instance.as_mut().fees();
        let flags = instance.as_mut().flags();
        ApplyView {
            instance,
            fees: Fees::new(fees),
            flags,
        }
    }

    pub fn read(&self, keylet: &Keylet) -> Option<ConstSLE> {
        let maybe_sle = rippled_bridge::rippled::read(self.as_read_view().instance, keylet);
        if maybe_sle.is_null() {
            None
        } else {
            Some(ConstSLE::new(maybe_sle))
        }
    }

    pub fn peek(&mut self, keylet: &Keylet) -> Option<SLE> {
        let maybe_sle = self.instance.as_mut().peek(keylet);
        if maybe_sle.is_null() {
            None
        } else {
            Some(SLE::new(maybe_sle))
        }
    }

    pub fn peek_typed<T: LedgerObject>(&mut self, keylet: &Keylet) -> Option<T> {
        self.peek(keylet)
            .map(|sle| T::from(sle))
    }

    pub fn insert(&mut self, sle: &SLE) {
        self.instance.as_mut().insert(&sle.instance);
    }

    pub fn insert_object<T: LedgerObject>(&mut self, object: &T) {
        self.instance.as_mut().insert(&object.get_sle().instance)
    }

    pub fn update(&mut self, sle: &SLE) {
        self.instance.as_mut().update(&sle.instance);
    }

    pub fn update_object<T: LedgerObject>(&mut self, object: &T) {
        self.instance.as_mut().update(&object.get_sle().instance);
    }

    pub fn dir_insert(&mut self, directory: &Keylet, key: &Keylet, account_id: &AccountId) -> Option<u64> {
        let result: UniquePtr<OptionalUInt64> = rippled_bridge::rippled::dir_insert(self.instance.as_mut(), directory, key, &account_id.into());
        if rippled_bridge::rippled::has_value(&result) {
            Some(rippled_bridge::rippled::get_value(&result))
        } else {
            None
        }
    }

    pub fn fees(&self) -> &Fees {
        &self.fees
    }

    pub fn flags(&self) -> ApplyFlags {
        self.flags
    }

    pub fn seq(&self) -> u32 {
        self.instance.seq()
    }

    pub fn succ(&mut self, key: &Keylet, last: &Keylet) -> Option<UInt256> {
        let result = rippled_bridge::rippled::apply_view_succ(self.instance.as_mut(), key, last);
        if rippled_bridge::rippled::opt_uint256_has_value(&result) {
            Some(rippled_bridge::rippled::opt_uint256_get_value(&result))
        } else {
            None
        }
    }

    pub fn adjust_owner_count(&mut self, sle: &SLE, amount: i32, j: &Journal) {
        rippled_bridge::rippled::adjustOwnerCount(self.instance.as_mut(), &sle.instance, amount, j.instance);
    }

    fn as_read_view(&self) -> ReadView {
        ReadView::new(rippled_bridge::rippled::upcast_apply_view(&self.instance))
    }
}

pub struct Journal<'a> {
    instance: &'a rippled_bridge::rippled::Journal,
}

impl Journal<'_> {
    pub fn new(instance: &rippled_bridge::rippled::Journal) -> Journal {
        Journal { instance }
    }
}

pub struct Application<'a> {
    instance: Pin<&'a mut rippled_bridge::rippled::Application>,
}

impl Application<'_> {
    pub fn new(instance: Pin<&mut rippled_bridge::rippled::Application>) -> Application {
        Application { instance }
    }
}

pub struct ApplyContext<'a> {
    instance: &'a mut Pin<&'a mut rippled_bridge::rippled::ApplyContext>,
    pub tx: STTx<'a>,
    pub view: ApplyView<'a>,
    pub app: Application<'a>,
    pub base_fee: XrpAmount,
    pub journal: Journal<'a>,
}

impl<'a> ApplyContext<'a> {
    pub fn new(instance: &'a mut Pin<&'a mut rippled_bridge::rippled::ApplyContext>) -> ApplyContext<'a> {
        let tx = instance.getTx();
        let view = instance.as_mut().view();
        let app = instance.as_mut().getApp();
        let base_fee = instance.as_mut().getBaseFee();
        let journal = instance.as_mut().getJournal();
        ApplyContext {
            instance,
            tx: STTx::new(tx),
            view: ApplyView::new(view),
            app: Application::new(app),
            base_fee: base_fee.into(),
            journal: Journal::new(journal),
        }
    }
}

pub struct TxConsequences {
    inner: rippled_bridge::tx_consequences::TxConsequences,
}

impl TxConsequences {
    pub fn with_potential_spend(tx: &STTx, potential_spend: XrpAmount) -> Self {
        let fee = tx.get_amount(&SField::sf_fee());
        TxConsequences {
            inner: rippled_bridge::tx_consequences::TxConsequences::new(
                false,
                if !fee.negative() { fee.xrp().into() } else { XRPAmount::zero() },
                potential_spend.into(),
                tx.seq_proxy(),
                1
            )
        }
    }
}

impl From<TxConsequences> for rippled_bridge::tx_consequences::TxConsequences {
    fn from(value: TxConsequences) -> Self {
        value.inner
    }
}

impl From<NotTEC> for TxConsequences {
    fn from(value: NotTEC) -> Self {
        if value == tesSUCCESS {
            panic!("Preflight result must not be tesSUCCESS");
        }
        TxConsequences {
            inner: rippled_bridge::tx_consequences::TxConsequences::new(
                false,
                XRPAmount::zero(),
                XRPAmount::zero(),
                SeqProxy::sequence(0),
                0
            )
        }
    }
}

pub fn preflight1(ctx: &PreflightContext) -> rippled_bridge::NotTEC {
    rippled_bridge::rippled::preflight1(ctx.instance)
}

pub fn preflight2(ctx: &PreflightContext) -> rippled_bridge::NotTEC {
    rippled_bridge::rippled::preflight2(ctx.instance)
}

pub const TF_FULLY_CANONICAL_SIG: u32 = 0x80000000;
pub const TF_UNIVERSAL: u32 = TF_FULLY_CANONICAL_SIG;
pub const TF_UNIVERSAL_MASK: u32 = !TF_UNIVERSAL;
pub const TF_PAYMENT_MASK: u32 = !TF_UNIVERSAL;

pub fn minimum_fee(app: &mut Application, base_fee: XrpAmount, fees: &Fees, flags: ApplyFlags) -> XrpAmount {
    rippled_bridge::rippled::minimumFee(app.instance.as_mut(), base_fee.into(), fees.instance, flags).into()
}