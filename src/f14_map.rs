//! F14VectorMap 核心实现

use crate::{dispatch_simd, traits::HasherExt};
use std::println as info; // 使用 info! 宏替代 println!
use super::{
    simd_utils::{self, CHUNK_SIZE, EMPTY, DELETED, FULL_MASK},
    error::MapError,
    traits::BuildHasherExt,
    iterators::{Iter, IterMut, IntoIter},
    allocator::AlignedAllocator,
};
use std::{
    borrow::Borrow, hash::{ Hash}, marker::PhantomData, mem::{self, MaybeUninit}, ptr::{self, NonNull}
};
const MAX_CAPACITY: usize = usize::MAX / (CHUNK_SIZE * 2);
/// 槽位状态
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SlotState {
    Empty,
    Deleted,
    Full,
}

/// 键值对存储单元
#[repr(C)]
pub(crate) struct KeyValuePair<K, V> {
   pub key: MaybeUninit<K>,
   pub  value: MaybeUninit<V>,
}

/// F14VectorMap 主结构
pub struct F14VectorMap<K, V, S = std::collections::hash_map::RandomState> 
where
    K: Sized,  // 在结构体级别添加约束
    V: Sized,  
{
    // 控制字节数组
    ctrls: NonNull<u8>,
    // 键值对数组
    entries: NonNull<KeyValuePair<K, V>>,
    // 容量（总槽位数）
    capacity: usize,
    // 分组数
    group_count: usize,
    // 有效元素数量
    len: usize,
    // 删除标记数量
    deleted: usize,
    // 哈希构建器
    hasher_builder: S,
    // 标记类型关系
    phantom: PhantomData<(K, V)>,
}

impl<K, V, S> F14VectorMap<K, V, S> {
    /// 获取每组的槽位数
    #[inline]
    pub fn chunk_size(&self) -> usize {
        CHUNK_SIZE
    }
    
    /// 获取分组数量
    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_count
    }
    
    /// 获取容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// 获取元素数量
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
     /// 获取删除标记数量
    pub fn deleted_count(&self) -> usize {
        self.deleted
    }
    /// 检查是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    /// 获取控制字节指针
    #[inline]
    fn ctrls_ptr(&self) -> *mut u8 {
        self.ctrls.as_ptr()
    }
    
    /// 获取键值对指针
    #[inline]
   fn entries_ptr(&self) -> *mut KeyValuePair<K, V> {
        self.entries.as_ptr()
    }
    /// 获取指定索引的键值对引用 (内部使用)
    #[inline]
    pub(crate)  fn get_entry(&self, index: usize) -> &KeyValuePair<K, V> {
        unsafe { &*self.entries_ptr().add(index) }
    }
    
    /// 获取指定索引的键值对可变引用 (内部使用)
    #[inline]
    pub(crate)   fn get_entry_mut(&mut self, index: usize) -> &mut KeyValuePair<K, V> {
        unsafe { &mut *self.entries_ptr().add(index) }
    }
    /// 获取指定索引的控制字节
    #[inline]
    fn get_ctrl(&self, index: usize) -> u8 {
        unsafe { *self.ctrls_ptr().add(index) }
    }
    
    /// 设置指定索引的控制字节
    #[inline]
    fn set_ctrl(&mut self, index: usize, ctrl: u8) {
        unsafe { *self.ctrls_ptr().add(index) = ctrl }
    }
    
    /// 获取槽位状态
    pub fn slot_state(&self, index: usize) -> SlotState {
        let ctrl = self.get_ctrl(index);
        match ctrl {
            EMPTY => SlotState::Empty,
            DELETED => SlotState::Deleted,
            _ if ctrl & FULL_MASK == ctrl => SlotState::Full,
            _ => SlotState::Deleted, // 无效状态处理为Deleted
        }
    }
    
     /// 减少长度（内部使用）
   pub fn decrement_len(&mut self) {
        self.len -= 1;
    }
    
    /// 替换槽位状态并返回旧数据（内部使用）
    pub unsafe fn replace_slot_state(&mut self, index: usize, new_state: SlotState) -> (K, V) {
        // 保存旧状态
        let old_ctrl = self.get_ctrl(index);
        let old_state = match old_ctrl {
            EMPTY => SlotState::Empty,
            DELETED => SlotState::Deleted,
            _ => SlotState::Full,
        };
        
        // 设置新控制字节
        let new_ctrl = match new_state {
            SlotState::Empty => EMPTY,
            SlotState::Deleted => DELETED,
            SlotState::Full => {0},
        };
        self.set_ctrl(index, new_ctrl);
        
        // 获取槽位数据
        unsafe {
        //let entry = self.get_entry_mut(index);
        //let key = ptr::read(entry.key.as_ptr());
        //let value = ptr::read(entry.value.as_ptr());
         // 获取槽位数据
    let entry_ptr = self.entries_ptr().add(index);
    let key = ptr::read(&(*entry_ptr).key as *const _ as *const K);
    let value = ptr::read(&(*entry_ptr).value as *const _ as *const V);
        // 如果从FULL状态变为非FULL状态，减少长度
        if matches!(old_state, SlotState::Full) && !matches!(new_state, SlotState::Full) {
            self.len -= 1;
        }
        
        (key, value)
        }
        
    }

     /// 计算内存布局
    /// 计算内存布局
    fn calculate_layout(capacity: usize) -> Result<usize, MapError> {
        if capacity == 0 {
            return Ok(0);
        }
        
        // 计算控制字节大小
        let ctrls_size = capacity * mem::size_of::<u8>();
        // 计算键值对大小
        let entries_size = capacity * mem::size_of::<KeyValuePair<K, V>>();
        
        // 总大小
        let total_size = ctrls_size + entries_size;
        
        Ok(total_size)
    }
    
    /// 分配内存
    unsafe fn allocate(capacity: usize) -> Result<(NonNull<u8>, NonNull<KeyValuePair<K, V>>), MapError> {
        if capacity == 0 {
            return Ok((
                NonNull::dangling(),
                NonNull::dangling(),
            ));
        }
        
        // 计算布局
        let total_size = Self::calculate_layout(capacity)?;
        
        // 分配内存
        let ptr = unsafe { AlignedAllocator::alloc_aligned(total_size) }?;
        
        // 初始化控制字节为EMPTY
        dispatch_simd!(
            fill_ctrls, 
            ptr.as_ptr(),
            EMPTY,
            capacity
        );
        
        // 设置键值对指针（控制字节之后）
        let entries_ptr = unsafe { ptr.as_ptr().add(capacity) } as *mut KeyValuePair<K, V>;
        let entries = NonNull::new(entries_ptr).ok_or(MapError::CapacityExceeded)?;
        
        Ok((ptr, entries))
    }
    
    /// 释放内存
    unsafe fn deallocate(&self) {
        if self.capacity == 0 {
            return;
        }
        
        let total_size = Self::calculate_layout(self.capacity)
            .expect("Invalid layout calculation");
        unsafe { AlignedAllocator::dealloc_aligned(self.ctrls.as_ptr(), total_size) };
    }
    
    
   

}
    
   

impl<K, V, S> F14VectorMap<K, V, S>
where
    K: Sized,  // 添加必要的约束
    V: Sized, 
    S: BuildHasherExt + Default,
{
    /// 创建一个新的 F14VectorMap
    pub fn new() ->  Result<Self, MapError> {
        Self::with_capacity_and_hasher(0, S::default())
    }
    
    /// 创建具有指定容量的 F14VectorMap
    pub fn with_capacity(capacity: usize) ->  Result<Self, MapError> {
        Self::with_capacity_and_hasher(capacity, S::default())
    }
}

impl<K, V, S> F14VectorMap<K, V, S>
where
    K: Sized,  // 添加必要的约束
    V: Sized, 
    
    S: BuildHasherExt,
{
    /// 使用指定的哈希构建器创建 F14VectorMap
    pub fn with_hasher(hasher: S) ->  Result<Self, MapError> {
        Self::with_capacity_and_hasher(0, hasher)
    }
    
    /// 使用指定容量和哈希构建器创建 F14VectorMap
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) ->  Result<Self, MapError> {
        // 确保容量是CHUNK_SIZE的倍数
        let group_count = if capacity > 0 {
            (capacity + CHUNK_SIZE - 1) / CHUNK_SIZE
        } else {
            0
        };
        
        let capacity = group_count * CHUNK_SIZE;
        // 检查容量是否过大（在分配内存前）
        if capacity > MAX_CAPACITY {
            return Err(MapError::CapacityExceeded);
        }
        // 分配内存
        let (ctrls, entries) = if capacity > 0 {
            unsafe { Self::allocate(capacity).expect("Allocation failed") }
        } else {
            (
                NonNull::dangling(),
                NonNull::dangling(),
            )
        };
        
         Ok(F14VectorMap {
            ctrls,
            entries,
            capacity,
            group_count,
            len: 0,
            deleted: 0,
            hasher_builder: hasher,
            phantom: PhantomData,
        })
    }
    
    /// 计算键的哈希和片段
    fn hash_key<Q>(&self, key: &Q) -> (u64, u8)
    where
        Q: Hash + ?Sized,
    {
        let mut hasher = self.hasher_builder.build_hasher_ext();
        key.hash(&mut hasher);
        let (full_hash, fragment) = hasher.finish_ext();
    
    // 确保片段在有效范围内
    let fragment = fragment & 0x7F; // 只保留低7位
    //info!("Hashing key: {}, full_hash: {}, fragment: {}", std::any::type_name::<Q>(), full_hash, fragment);
    (full_hash, fragment)
    }
    
    /// 获取组起始索引
    #[inline]
    fn group_start(&self, full_hash: u64) -> usize {
        if self.group_count == 0 {
            return 0; // 容量为0时返回0
        }
        
        // 计算组索引 = 哈希值 % 分组数
        let group_index = full_hash as usize % self.group_count;
        
        // 计算组起始索引 = 组索引 * 组大小
        let start =group_index * CHUNK_SIZE;
      //  info!("group_start: full_hash={}, group_count={}, group_index={}, start={}", full_hash, self.group_count, group_index, start);
        start
    }
    
    /// 重建表以减少墓碑
   pub  fn rebuild(&mut self) -> Result<(), MapError>
    where
        K: Eq + Hash,
     S: Clone,
    {
         // 如果没有墓碑，直接返回
    if self.deleted == 0 {
        return Ok(());
    }
    info!("开始重建表: len={}, deleted={}", self.len, self.deleted);
        // 创建新表，容量相同但删除墓碑
        let mut new_table = F14VectorMap::with_capacity_and_hasher(
            self.capacity(),
            self.hasher_builder.clone(),
        )?;
        
        // 迁移所有元素
    let mut migrated = 0;
    for index in 0..self.capacity {
        let ctrl = self.get_ctrl(index);
        
        // 只处理FULL状态的槽位（高位为0）
        if ctrl < 128 {
            unsafe {
                // 获取键值对
                let entry_ptr = self.entries_ptr().add(index);
                let key_ptr = &(*entry_ptr).key as *const _ as *const K;
                let value_ptr = &(*entry_ptr).value as *const _ as *const V;
                
                let key = ptr::read(key_ptr);
                let value = ptr::read(value_ptr);
                
                // 插入新表
                new_table.insert(key, value)?;
                
                // 标记原槽位为EMPTY
                self.set_ctrl(index, EMPTY);
                
                migrated += 1;
            }
        }
    }
    
    info!("迁移完成: 迁移了 {} 个元素", migrated);
    
    // 交换表
    *self = new_table;
    info!("重建完成: 新 len={}, deleted={}", self.len, self.deleted);
    Ok(())
    }
    
    /// 扩容表
    fn resize(&mut self) -> Result<(), MapError>
    where
        K: Eq + Hash,
        S: Clone,
    {
        info!("开始扩容: 当前 len={}, capacity={}", self.len, self.capacity);
        // 计算新容量（翻倍或初始大小）
        let new_capacity = if self.capacity == 0 {
            CHUNK_SIZE
        } else {
            self.capacity * 2
        };
        info!("新容量: {}", new_capacity);
        // 创建新表
        let mut new_table = F14VectorMap::with_capacity_and_hasher(
            new_capacity,
            self.hasher_builder.clone(),
        )?;
        
         let mut migrated = 0;
        // 迁移所有元素
        for index in 0..self.capacity {
            if self.get_ctrl(index) < 128 { // 高位为0表示FULL
            unsafe {
                let (key, value) = self.replace_slot_state(index, SlotState::Empty);
                new_table.insert(key, value)?;
                migrated += 1;
            }
        }
        }
        info!("迁移完成: 迁移了 {} 个元素", migrated);
        // 交换表
        *self = new_table;
        info!("扩容完成: 新 len={}, capacity={}", self.len, self.capacity);
        Ok(())
    }
    
    /// 插入键值对
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, MapError>
    where
        K: Eq + Hash,
        S: Clone,
    {
        // 如果容量为0，直接扩容
        if self.capacity == 0 {
            self.resize()?;
        }
        
        // 重建检查
        if self.deleted > self.len / 2 {
            self.rebuild()?;
        }
        
        // 扩容检查
        if self.len >= self.capacity * 7 / 10 {
            self.resize()?;
        }
        
        let (full_hash, fragment) = self.hash_key(&key);
      //  info!("insert 键 {} 的哈希: full_hash={}, fragment={}", std::any::type_name::<K>(), full_hash, fragment);
        let group_start = self.group_start(full_hash);
      //  info!("insert 分组数: {}, group_start={}", self.group_count, group_start);
        let fragment = simd_utils::make_ctrl_byte(fragment);
        
                
        // 2. 检查键是否已存在
        if let Some(index) = self.find_in_group(group_start, &key, fragment) {
            return self.replace_value(index, value);
        }
        // 1. 在初始组内查找空闲位置
        if let Some(index) = self.find_empty_in_group(group_start) {
            return self.insert_at(index, key, value, fragment);
        }
        
        // 3. 二次哈希探测其他组
        let step = (full_hash as usize % self.group_count) | 1;
        let mut group_index = (group_start / CHUNK_SIZE + step) % self.group_count;
        let mut probe_count = 0;
        
        while probe_count < self.group_count * 2 {
            let group_start = group_index * CHUNK_SIZE;
            
            // 在组内查找空闲位置
            if let Some(index) = self.find_empty_in_group(group_start) {
                return self.insert_at(index, key, value, fragment);
            }
            
            // 检查键是否已存在
            if let Some(index) = self.find_in_group(group_start, &key, fragment) {
                return self.replace_value(index, value);
            }
            
            // 跳到下一个组
            group_index = (group_index + step) % self.group_count;
            probe_count += 1;
        }
        
        // 4. 如果探测失败，扩容后重试
        self.resize()?;
        self.insert(key, value)
    }
    
    /// 在组内查找空闲位置
    #[inline]
    fn find_empty_in_group(&self, group_start: usize) -> Option<usize> {
        dispatch_simd!(
            find_empty,
            unsafe {
               self.ctrls_ptr().add(group_start) 
            }
            
        ).map(|slot| group_start + slot)
    }
    
    /// 在组内查找键
    #[inline]
    fn find_in_group<Q>(
        &self,
        group_start: usize,
        key: &Q,
        fragment: u8,
    ) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
       
        // 确保索引在有效范围内
    if group_start >= self.capacity {
        return None;
    }
    // 使用 SIMD 查找匹配片段的位置
    // 使用 SIMD 查找所有匹配片段的位置
    let matches = unsafe {
        simd_utils::find_all_matches(
            self.ctrls_ptr().add(group_start),
            fragment
        )
    };
    // 检查所有匹配位置
    for &slot_in_group in matches.iter() {
        // 遇到 0xFF 表示结束
        if slot_in_group == 0xFF {
            break;
        }
        
        let index = group_start + slot_in_group as usize;
        
        // 确保索引在有效范围内
        if index >= self.capacity {
            continue;
        }
        
        // 验证键是否匹配
        unsafe {
            let entry_ptr = self.entries_ptr().add(index);
            let key_ptr = (*entry_ptr).key.as_ptr() as *const K;
            let candidate_key = &*key_ptr;
            
            if candidate_key.borrow().eq(key) {
                return Some(index);
            }
        }
    }
    
    None
}
    /// 在指定位置插入键值对
    // 修改insert_at函数
fn insert_at(&mut self, index: usize, key: K, value: V, fragment: u8) -> Result<Option<V>, MapError> {
    let state = self.slot_state(index);
    
    // 写入数据
    unsafe {
        let entry_ptr = self.entries_ptr().add(index);
        ptr::write(&mut (*entry_ptr).key, MaybeUninit::new(key));
        ptr::write(&mut (*entry_ptr).value, MaybeUninit::new(value));
    }
    
    // 更新控制字节
    self.set_ctrl(index, fragment);
    
    // 更新计数
    match state {
        SlotState::Full => panic!("Inserting into a full slot"),
        SlotState::Deleted => self.deleted -= 1,
        SlotState::Empty => {},
    }
    
    self.len += 1;
    info!("插入成功: index={}, len={}", index, self.len);
    Ok(None)
}

// 修改replace_value函数
fn replace_value(&mut self, index: usize, value: V) -> Result<Option<V>,  MapError> {
    let state = self.slot_state(index);
    if state != SlotState::Full {
        return Err(MapError::InvalidSlotState);
    }
    
    // 替换值并返回旧值
    unsafe {
        let entry_ptr = self.entries_ptr().add(index);
         // 保存旧值
        let old_value = ptr::read(&(*entry_ptr).value).assume_init();
        
        // 写入新值
        ptr::write(&mut (*entry_ptr).value, MaybeUninit::new(value));
        
        Ok(Some(old_value))
    }
}


    
    /// 查找键
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let (full_hash, fragment) = self.hash_key(key);
      //  info!("get 键 {} 的哈希: full_hash={}, fragment={}", std::any::type_name::<Q>(), full_hash, fragment);
        
        let group_start = self.group_start(full_hash);
      //  info!("get 分组数: {}, group_start={}", self.group_count, group_start);

        let fragment = simd_utils::make_ctrl_byte(fragment);
        // 1. 在初始组内查找
        if let Some(index) = self.find_in_group(group_start, key, fragment) {
            unsafe {
                let entry_ptr = self.entries_ptr().add(index);
                let value_ptr = (*entry_ptr).value.as_ptr() as *const V;
                return Some(&*value_ptr);
            }
        }
        
        // 2. 二次哈希探测其他组
        let step = (full_hash as usize % self.group_count) | 1;
        let mut group_index = (group_start / CHUNK_SIZE + step) % self.group_count;
        let mut probe_count = 0;
        
        while probe_count < self.group_count * 2 {
            let group_start = group_index * CHUNK_SIZE;
            
            // 在组内查找键
            if let Some(index) = self.find_in_group(group_start, key, fragment) {
                unsafe {
                    let entry_ptr = self.entries_ptr().add(index);
                let value_ptr = (*entry_ptr).value.as_ptr() as *const V;
                return Some(&*value_ptr);
                }
            }
            
            // 跳到下一个组
            group_index = (group_index + step) % self.group_count;
            probe_count += 1;
        }
        
        None
    }
    
    /// 移除键
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let (full_hash, fragment) = self.hash_key(key);
        let fragment = simd_utils::make_ctrl_byte(fragment);
        let group_start = self.group_start(full_hash);
        
        // 1. 在初始组内查找
        if let Some(index) = self.find_in_group(group_start, key, fragment) {
            return self.remove_at(index);
        }
        
        // 2. 二次哈希探测其他组
        let step = (full_hash as usize % self.group_count) | 1;
        let mut group_index = (group_start / CHUNK_SIZE + step) % self.group_count;
        let mut probe_count = 0;
        
        while probe_count < self.group_count * 2 {
            let group_start = group_index * CHUNK_SIZE;
            
            // 在组内查找键
            if let Some(index) = self.find_in_group(group_start, key, fragment) {
                return self.remove_at(index);
            }
            
            // 跳到下一个组
            group_index = (group_index + step) % self.group_count;
            probe_count += 1;
        }
        
        None
    }
    
    // 修改remove_at函数
fn remove_at(&mut self, index: usize) -> Option<V> {
    // 确保槽位状态为 FULL
        if self.get_ctrl(index) >= 128 {
            return None;
        }
        // 标记为删除
        self.set_ctrl(index, DELETED);
        self.deleted += 1;
        self.len -= 1;
        
        // 取出值
         // 取出值
        unsafe {
            let value_ptr = &(*self.entries_ptr().add(index)).value as *const _ as *const V;
            let value = ptr::read(value_ptr);
            Some(value)
        }
    }
    
    
    
    /// 获取迭代器
    pub fn iter(&self) -> Iter<'_, K, V, S> {
        Iter::new(self)
    }
    
    /// 获取可变迭代器
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, S> {
        IterMut::new(self)
    }

    /// 获取消耗迭代器
    pub fn into_iter(self) -> IntoIter<K, V, S> {
        IntoIter::new(self)
    }
}


impl<K, V, S> F14VectorMap<K, V, S>
where
    K: Sized,
    V: Sized,
{
    /// 内部清理方法
    fn internal_clear(&mut self) {
        for index in 0..self.capacity {
            match self.slot_state(index) {
                SlotState::Full => {
                    unsafe {
                        let key_ptr = &mut (*self.entries_ptr().add(index)).key as *mut _ as *mut K;
                        let val_ptr = &mut (*self.entries_ptr().add(index)).value as *mut _ as *mut V;
                        
                        ptr::drop_in_place(key_ptr);
                        ptr::drop_in_place(val_ptr);
                    }
                    self.set_ctrl(index, EMPTY);
                }
                SlotState::Deleted => {
                    self.set_ctrl(index, EMPTY);
                }
                SlotState::Empty => {}
            }
        }
        
        self.len = 0;
        self.deleted = 0;
    }
    
    /// 公共 clear 方法
    pub fn clear(&mut self) {
        self.internal_clear();
    }
}

impl<K, V, S> Drop for F14VectorMap<K, V, S>
where
    K: Sized,  // 添加必要的约束
    V: Sized,   // 确保类型可安全操作
 
{
    fn drop(&mut self) {
        // 释放所有元素
        self.clear();
        
        // 释放内存
        unsafe {
            self.deallocate();
        }
    }
}

impl<K, V, S> IntoIterator for F14VectorMap<K, V, S> 
where
    K: Sized,  // 添加必要的约束
    V: Sized, 
{
    type Item = (K, V);
    type IntoIter = IntoIter<K, V, S>;
    
    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

impl<K: std::fmt::Debug, V: std::fmt::Debug, S> std::fmt::Debug for F14VectorMap<K, V, S> 
where
    K: std::fmt::Debug,
    V: std::fmt::Debug,
S: BuildHasherExt,  // 添加 BuildHasherExt 约束
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(self.iter())  // 现在可以调用 iter()
            .finish()
    }
}

impl<K, V, S> Default for F14VectorMap<K, V, S>
where
    K: Sized,  // 添加必要的约束
    V: Sized, 
   
    S: BuildHasherExt + Default,
{
    fn default() ->  Self {
        F14VectorMap {
            ctrls: NonNull::dangling(),
            entries: NonNull::dangling(),
            capacity: 0,
            group_count: 0,
            len: 0,
            deleted: 0,
            hasher_builder: S::default(),
            phantom: PhantomData,
        }
    }
}