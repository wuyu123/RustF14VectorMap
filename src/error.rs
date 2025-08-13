//! 映射错误类型定义

use std::fmt;

/// 映射操作可能发生的错误
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapError {
    /// 超出最大容量限制
    CapacityExceeded,
    /// SIMD 操作不支持
    UnsupportedSimd,
    /// 并发修改冲突
    ConcurrentModification,
    InvalidSlotState,
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MapError::CapacityExceeded => write!(f, "Map capacity exceeded"),
            MapError::UnsupportedSimd => write!(f, "SIMD not supported on this platform"),
            MapError::ConcurrentModification => write!(f, "Concurrent modification detected"),
             MapError::InvalidSlotState => write!(f, "Invalid Slot State"),
        }
    }
}

impl std::error::Error for MapError {}