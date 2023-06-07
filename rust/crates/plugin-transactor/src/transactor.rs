use std::pin::Pin;
use xrpl_rust_sdk_core::core::types::XrpAmount;
use rippled_bridge::{NotTEC, SOEStyle, TER};
use crate::{ApplyContext, ConstSLE, PreclaimContext, PreflightContext, ReadView, SLE, STObject, STTx, TxConsequences};

pub trait LedgerObject: From<SLE> {
    fn get_sle(&self) -> &SLE;
}

pub trait ConstLedgerObject: From<ConstSLE> {
    fn get_sle(&self) -> &ConstSLE;
}

pub trait AsSTObject {
    fn as_st_object(&self) -> &STObject;
}

pub trait Transactor {
    fn pre_flight(ctx: PreflightContext) -> NotTEC;
    fn pre_claim(ctx: PreclaimContext) -> TER;
    fn calculate_base_fee(view: ReadView, tx: STTx) -> XrpAmount {
        rippled_bridge::rippled::defaultCalculateBaseFee(view.instance(), tx.instance).into()
    }
    fn do_apply<'a>(ctx: &'a mut ApplyContext<'a>, m_prior_balance: XrpAmount, m_source_balance: XrpAmount) -> TER;
    fn tx_format() -> Vec<SOElement>;
}

pub trait MakeTxConsequences: Transactor {
    fn make_tx_consequences(ctx: PreflightContext) -> TxConsequences;
}

pub struct SOElement {
    pub field_code: i32,
    pub style: SOEStyle
}