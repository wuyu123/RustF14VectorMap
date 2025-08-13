//! 自定义哈希trait

use std::hash::{ BuildHasher};

/// 增强版哈希构建器trait
pub trait BuildHasherExt: BuildHasher {
    /// 扩展的哈希器类型
    type ExtHasher: HasherExt;  // 重命名为 ExtHasher
    
    /// 创建扩展的哈希器
    fn build_hasher_ext(&self) -> Self::ExtHasher;
}

/// 扩展的哈希器trait
pub trait HasherExt: std::hash::Hasher {
    /// 完成哈希计算，返回完整的哈希值和片段
    fn finish_ext(&self) -> (u64, u8);
}

/// 为所有实现了BuildHasher的类型提供默认实现
impl<T: BuildHasher> BuildHasherExt for T {
    type ExtHasher = DefaultHasherWrapper<<T as BuildHasher>::Hasher>;  // 使用新名称
    
    fn build_hasher_ext(&self) -> Self::ExtHasher {
        DefaultHasherWrapper(self.build_hasher())
    }
}

/// 默认的哈希器包装器
pub struct DefaultHasherWrapper<H>(H);

impl<H: std::hash::Hasher> std::hash::Hasher for DefaultHasherWrapper<H> {
    fn write(&mut self, bytes: &[u8]) {
        self.0.write(bytes)
    }
    
    fn finish(&self) -> u64 {
        self.0.finish()
    }
}

impl<H: std::hash::Hasher> HasherExt for DefaultHasherWrapper<H> {
    fn finish_ext(&self) -> (u64, u8) {
        let full_hash = self.0.finish();
        // 取高7位作为片段
        let fragment = (full_hash >> (64 - 7)) as u8;
        (full_hash, fragment)
    }
}