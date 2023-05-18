extern crate core;

pub mod transactor;

use core::slice;
use std::ops::Deref;
use std::pin::Pin;
use cxx::SharedPtr;
pub use transactor::Transactor;

use xrpl_rust_sdk_core::core::types::{AccountId, XrpAmount};
use rippled_bridge::{ApplyFlags, Keylet, LedgerSpecificFlags};
use rippled_bridge::rippled::setFlag;

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
}

impl PreclaimContext<'_> {
    pub fn new(instance: &rippled_bridge::rippled::PreclaimContext) -> PreclaimContext {
        PreclaimContext { instance }
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
        rippled_bridge::rippled::upcast(self.instance).getAccountID(field.instance).into()
    }

    pub fn get_plugin_type(&self, field: &SField) -> STPluginType {
        STPluginType::new(self.as_st_object().getPluginType(field.instance))
    }

    pub fn is_field_present(&self, field: &SField) -> bool {
        self.as_st_object().isFieldPresent(field.instance)
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

    pub fn get_plugin_field(type_id: i32, field_id: i32) -> Self {
        SField {
            instance: rippled_bridge::rippled::getSField(type_id, field_id)
        }
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
}

pub struct Fees<'a> {
    instance: &'a rippled_bridge::rippled::Fees,
}

impl Fees<'_> {
    pub fn new(instance: &rippled_bridge::rippled::Fees) -> Fees {
        Fees { instance }
    }
}

pub struct SLE {
    instance: SharedPtr<rippled_bridge::rippled::SLE>,
}

impl SLE {
    pub fn new(instance: SharedPtr<rippled_bridge::rippled::SLE>) -> Self {
        SLE { instance }
    }

    pub fn set_plugin_type(&self, field: &SField, value: &STPluginType) {
        rippled_bridge::rippled::setPluginType(&self.instance, field.instance, value.instance);
    }

    pub fn make_field_absent(&self, field: &SField) {
        rippled_bridge::rippled::makeFieldAbsent(&self.instance, field.instance);
    }

    pub fn set_flag(&self, flag: LedgerSpecificFlags) {
        setFlag(&self.instance, flag.into());
    }

    pub fn is_flag(&self, flag: LedgerSpecificFlags) -> bool {
        self.instance.deref().isFlag(flag.into())
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

    pub fn peek(&mut self, keylet: &Keylet) -> Option<SLE> {
        let maybe_sle = self.instance.as_mut().peek(keylet);
        if maybe_sle.is_null() {
            None
        } else {
            Some(SLE::new(maybe_sle))
        }
    }

    pub fn fees(&self) -> &Fees {
        &self.fees
    }

    pub fn flags(&self) -> ApplyFlags {
        self.flags
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
}

impl<'a> ApplyContext<'a> {
    pub fn new(instance: &'a mut Pin<&'a mut rippled_bridge::rippled::ApplyContext>) -> ApplyContext<'a> {
        let tx = instance.getTx();
        let view = instance.as_mut().view();
        let app = instance.as_mut().getApp();
        let base_fee = instance.as_mut().getBaseFee();
        ApplyContext {
            instance,
            tx: STTx::new(tx),
            view: ApplyView::new(view),
            app: Application::new(app),
            base_fee: base_fee.into(),
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

pub fn minimum_fee(app: &mut Application, base_fee: XrpAmount, fees: &Fees, flags: ApplyFlags) -> XrpAmount {
    rippled_bridge::rippled::minimumFee(app.instance.as_mut(), base_fee.into(), fees.instance, flags).into()
}