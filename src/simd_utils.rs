//! SIMD 工具函数

/// SIMD 支持常量定义
pub const CHUNK_SIZE: usize = 16;
pub const SIMD_ALIGNMENT: usize = 64; // 适配AVX512

/// 控制字节常量
pub const EMPTY: u8 = 0b1000_0000;     // -128
pub const DELETED: u8 = 0b1000_0001;   // -127
pub const FULL_MASK: u8 = 0b0111_1111; // 所有设置位的掩码

/// SIMD策略trait
pub trait SimdStrategy {
    fn find_match(ctrls: *const u8, fragment: u8) -> Option<usize>;
    fn find_empty(ctrls: *const u8) -> Option<usize>;
    fn fill_ctrls(ctrls: *mut u8, value: u8, count: usize);
}

/// 标量降级实现
pub struct Scalar;
impl SimdStrategy for Scalar {
    #[inline]
    fn find_match(ctrls: *const u8, fragment: u8) -> Option<usize> {
        for i in 0..CHUNK_SIZE {
            if unsafe { *ctrls.add(i) } == fragment {
                return Some(i);
            }
        }
        None
    }
    
    #[inline]
    fn find_empty(ctrls: *const u8) -> Option<usize> {
        for i in 0..CHUNK_SIZE {
            let c = unsafe { *ctrls.add(i) };
            if c == EMPTY || c == DELETED {
                return Some(i);
            }
        }
        None
    }
    
    #[inline]
    fn fill_ctrls(ctrls: *mut u8, value: u8, count: usize) {
        for i in 0..count {
            unsafe { *ctrls.add(i) = value; }
        }
    }
}

// SSE2 实现
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub struct Sse2;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl SimdStrategy for Sse2 {
    #[inline]
    fn find_match(ctrls: *const u8, fragment: u8) -> Option<usize> {
        unsafe {
            use std::arch::x86_64::*;
            
            let ctrl_vec = _mm_loadu_si128(ctrls as *const __m128i);
            let frag_vec = _mm_set1_epi8(fragment as i8);
            let match_vec = _mm_cmpeq_epi8(ctrl_vec, frag_vec);
            let mask = _mm_movemask_epi8(match_vec) as u32;
            
            if mask != 0 { Some(mask.trailing_zeros() as usize) } else { None }
        }
    }
    
    #[inline]
    fn find_empty(ctrls: *const u8) -> Option<usize> {
        unsafe {
            use std::arch::x86_64::*;
            
            let empty_vec = _mm_set1_epi8(EMPTY as i8);
            let deleted_vec = _mm_set1_epi8(DELETED as i8);
            
            let ctrl_vec = _mm_loadu_si128(ctrls as *const __m128i);
            let empty_match = _mm_cmpeq_epi8(ctrl_vec, empty_vec);
            let deleted_match = _mm_cmpeq_epi8(ctrl_vec, deleted_vec);
            let combined = _mm_or_si128(empty_match, deleted_match);
            
            let mask = _mm_movemask_epi8(combined) as u32;
            if mask != 0 { Some(mask.trailing_zeros() as usize) } else { None }
        }
    }
    
    #[inline]
    fn fill_ctrls(ctrls: *mut u8, value: u8, count: usize) {
        unsafe {
            use std::arch::x86_64::*;
            
            let fill_vec = _mm_set1_epi8(value as i8);
            let chunks = count / 16;
            let remainder = count % 16;
            
            let mut ptr = ctrls;
            for _ in 0..chunks {
                _mm_storeu_si128(ptr as *mut __m128i, fill_vec);
                ptr = ptr.add(16);
            }
            
            for i in 0..remainder {
                *ptr.add(i) = value;
            }
        }
    }

    
}

// AVX2 实现
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub struct Avx2;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
impl SimdStrategy for Avx2 {
    #[inline]
    fn find_match(ctrls: *const u8, fragment: u8) -> Option<usize> {
        unsafe {
            use std::arch::x86_64::*;
            
            let ctrl_vec = _mm256_loadu_si256(ctrls as *const __m256i);
            let frag_vec = _mm256_set1_epi8(fragment as i8);
            let match_vec = _mm256_cmpeq_epi8(ctrl_vec, frag_vec);
            let mask = _mm256_movemask_epi8(match_vec) as u32;
            
            if mask != 0 { Some(mask.trailing_zeros() as usize) } else { None }
        }
    }
    
    #[inline]
    fn find_empty(ctrls: *const u8) -> Option<usize> {
        unsafe {
            use std::arch::x86_64::*;
            
            let empty_vec = _mm256_set1_epi8(EMPTY as i8);
            let deleted_vec = _mm256_set1_epi8(DELETED as i8);
            
            let ctrl_vec = _mm256_loadu_si256(ctrls as *const __m256i);
            let empty_match = _mm256_cmpeq_epi8(ctrl_vec, empty_vec);
            let deleted_match = _mm256_cmpeq_epi8(ctrl_vec, deleted_vec);
            let combined = _mm256_or_si256(empty_match, deleted_match);
            
            let mask = _mm256_movemask_epi8(combined) as u32;
            if mask != 0 { Some(mask.trailing_zeros() as usize) } else { None }
        }
    }
    
    #[inline]
    fn fill_ctrls(ctrls: *mut u8, value: u8, count: usize) {
        unsafe {
            use std::arch::x86_64::*;
            
            let fill_vec = _mm256_set1_epi8(value as i8);
            let chunks = count / 32;
            let remainder = count % 32;
            
            let mut ptr = ctrls;
            for _ in 0..chunks {
                _mm256_storeu_si256(ptr as *mut __m256i, fill_vec);
                ptr = ptr.add(32);
            }
            
            for i in 0..remainder {
                *ptr.add(i) = value;
            }
        }
    }
}



/// SIMD特性检测宏
#[macro_export]
macro_rules! dispatch_simd {
    ($method:ident, $($arg:expr),*) => {
        {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                 if is_x86_feature_detected!("avx2") {
                    <$crate::simd_utils::Avx2 as $crate::simd_utils::SimdStrategy>::$method($($arg),*)
                } else if is_x86_feature_detected!("sse2") {
                    <$crate::simd_utils::Sse2 as $crate::simd_utils::SimdStrategy>::$method($($arg),*)
                } else {
                    <$crate::simd_utils::Scalar as $crate::simd_utils::SimdStrategy>::$method($($arg),*)
                }
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                <$crate::simd_utils::Scalar as $crate::simd_utils::SimdStrategy>::$method($($arg),*)
            }
        }
    };
}

/// 安全地生成片段字节
pub fn make_ctrl_byte(hash_frag: u8) -> u8 {
    // 确保高位为0（表示FULL）
    hash_frag & FULL_MASK
}

#[inline]
pub unsafe fn simd_find_empty(ctrls: *const u8) -> Option<usize> {
    dispatch_simd!(
        find_empty,
        ctrls
    )
}

/// 查找匹配片段的位置
#[inline]
pub unsafe fn simd_find_match(ctrls: *const u8, fragment: u8) -> Option<usize> {
    dispatch_simd!(
        find_match,
        ctrls,
        fragment
    )
}


/// 查找所有匹配片段的位置
#[inline]
pub unsafe fn find_all_matches(ctrls: *const u8, fragment: u8) -> [u8; CHUNK_SIZE] {
    unsafe {
         // 平台特性检测
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
     {
        if is_x86_feature_detected!("avx2") {
            return avx2_find_all_matches(ctrls, fragment);
        } else if is_x86_feature_detected!("sse2") {
            return sse2_find_all_matches(ctrls, fragment);
        }
    }
    
    // 标量回退
    scalar_find_all_matches(ctrls, fragment)

    }
   
}

/// SSE2 实现 (x86/x86_64)
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse2")]
unsafe fn sse2_find_all_matches(ctrls: *const u8, fragment: u8) -> [u8; CHUNK_SIZE] {
    use std::arch::x86_64::*;
    
    let ctrl_vec = unsafe { _mm_loadu_si128(ctrls as *const __m128i) };
    let frag_vec = _mm_set1_epi8(fragment as i8);
    let match_vec = _mm_cmpeq_epi8(ctrl_vec, frag_vec);
    let mask = _mm_movemask_epi8(match_vec) as u16;
    
    let mut matches = [0xFF; CHUNK_SIZE]; // 初始化为无效值
    
    if mask != 0 {
        let mut bitmask = mask;
        let mut count = 0;
        
        // 使用位扫描指令优化
        while bitmask != 0 {
            let index = bitmask.trailing_zeros() as u8;
            matches[count] = index;
            count += 1;
            bitmask &= bitmask - 1; // 清除最低位的1
        }
    }
    
    matches
}

/// AVX2 实现 (x86/x86_64)
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn avx2_find_all_matches(ctrls: *const u8, fragment: u8) -> [u8; CHUNK_SIZE] {
    use std::arch::x86_64::*;
    
    let ctrl_vec = unsafe { _mm256_loadu_si256(ctrls as *const __m256i) };
    let frag_vec = _mm256_set1_epi8(fragment as i8);
    let match_vec = _mm256_cmpeq_epi8(ctrl_vec, frag_vec);
    let mask = _mm256_movemask_epi8(match_vec) as u32;
    
    let mut matches = [0xFF; CHUNK_SIZE]; // 初始化为无效值
    
    if mask != 0 {
        let mut bitmask = mask;
        let mut count = 0;
        
        while bitmask != 0 && count < CHUNK_SIZE {
            let index = bitmask.trailing_zeros() as u8;
            matches[count] = index;
            count += 1;
            bitmask &= bitmask - 1; // 清除最低位的1
        }
    }
    
    matches
}


/// 标量回退实现
unsafe fn scalar_find_all_matches(ctrls: *const u8, fragment: u8) -> [u8; CHUNK_SIZE] {
    let mut matches = [0xFF; CHUNK_SIZE]; // 初始化为无效值
    let mut count = 0;
    
    // 使用 while 循环避免固定次数迭代
    let mut i = 0;
    while i < CHUNK_SIZE {
        if unsafe { *ctrls.add(i) } == fragment {
            matches[count] = i as u8;
            count += 1;
        }
        i += 1;
    }
    
    matches
}