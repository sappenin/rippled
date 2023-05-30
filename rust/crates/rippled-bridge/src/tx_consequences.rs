use cxx::kind::Trivial;
use cxx::type_id;
use crate::{NotTEC, XRPAmount};
use crate::rippled::STTx;
use crate::TEScodes::tesSUCCESS;

#[repr(C)]
pub struct TxConsequences {
    is_blocker: bool,
    fee: XRPAmount,
    potential_spend: XRPAmount,
    seq_proxy: SeqProxy,
    sequences_consumed: u32
}

impl TxConsequences {
    pub fn new(
        is_blocker: bool,
        fee: XRPAmount,
        potential_spend: XRPAmount,
        seq_proxy: SeqProxy,
        sequences_consumed: u32
    ) -> Self {
        TxConsequences {
            is_blocker,
            fee,
            potential_spend,
            seq_proxy,
            sequences_consumed
        }
    }
}

unsafe impl cxx::ExternType for TxConsequences {
    type Id = type_id!("ripple::TxConsequences");
    type Kind = Trivial;
}

#[repr(C)]
pub struct SeqProxy {
    value: u32,
    seq_type: SeqType
}

impl SeqProxy {
    pub fn sequence(value: u32) -> Self {
        SeqProxy {
            value,
            seq_type: SeqType::Seq
        }
    }
}

unsafe impl cxx::ExternType for SeqProxy {
    type Id = type_id!("ripple::SeqProxy");
    type Kind = Trivial;
}

#[repr(u8)]
pub enum SeqType {
    Seq = 0,
    Ticket
}

unsafe impl cxx::ExternType for SeqType {
    type Id = type_id!("ripple::SeqType");
    type Kind = Trivial;
}