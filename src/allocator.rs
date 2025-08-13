//! 定制内存分配器

use std::alloc::{GlobalAlloc, Layout, System};
use std::ptr::NonNull;
use crate::simd_utils::SIMD_ALIGNMENT;

/// 对齐内存分配器
pub struct AlignedAllocator;

impl AlignedAllocator {
    /// 分配对齐内存
    pub unsafe fn alloc_aligned(size: usize) -> Result<NonNull<u8>, crate::error::MapError> {
        if size == 0 {
            return Ok(NonNull::dangling());
        }
        
        // 创建布局
        let layout = Layout::from_size_align(size, SIMD_ALIGNMENT)
            .map_err(|_| crate::error::MapError::CapacityExceeded)?;
        
        // 分配内存
        let ptr = unsafe { System.alloc(layout) };
        if ptr.is_null() {
            return Err(crate::error::MapError::CapacityExceeded);
        }
        
        Ok(NonNull::new(ptr).unwrap())
    }
    
    /// 释放对齐内存
    pub unsafe fn dealloc_aligned(ptr: *mut u8, size: usize) {
        if size == 0 {
            return;
        }
        
        let layout = Layout::from_size_align(size, SIMD_ALIGNMENT)
            .expect("Invalid layout for deallocation");
        unsafe { System.dealloc(ptr, layout) };
    }
}

//