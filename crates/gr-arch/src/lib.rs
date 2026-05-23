pub mod arch;
pub mod error;

#[cfg(feature = "arm")]
pub mod arm;
#[cfg(feature = "x86")]
pub mod x86;

pub use arch::{
    Architecture, CallingConvention, DecodedInstruction, FlowType, ParamLocation, RegisterInfo,
};
pub use error::DisasmError;
