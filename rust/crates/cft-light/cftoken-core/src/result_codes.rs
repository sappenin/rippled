#[repr(i32)]
#[derive(Clone, Copy)]
pub enum TECCodes {
    tecNO_SUITABLE_CFTOKEN_PAGE = 162
}

impl From<TECCodes> for i32 {
    fn from(value: TECCodes) -> Self {
        value as i32
    }
}