//! F14VectorMap 迭代器实现

use super::f14_map::{F14VectorMap, SlotState};
use std::marker::PhantomData;

/// 不可变迭代器
pub struct Iter<'a, K, V, S> {
    map: &'a F14VectorMap<K, V, S>,
    current: usize,
}

impl<'a, K, V, S> Iter<'a, K, V, S> {
    pub(crate) fn new(map: &'a F14VectorMap<K, V, S>) -> Self {
        Self { map, current: 0 }
    }
}

impl<'a, K, V, S> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.map.capacity() {
            let index = self.current;
            self.current += 1;
            
            // 只处理FULL状态的槽位
            if let SlotState::Full = self.map.slot_state(index) {
                unsafe {
                    // 修复：直接访问KeyValuePair字段
                    let entry = self.map.get_entry(index);
                    let key = &*entry.key.as_ptr();
                    let value = &*entry.value.as_ptr();
                    return Some((key, value));
                }
            }
        }
        
        None
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}

/// 可变迭代器
pub struct IterMut<'a, K, V, S> {
    map: *mut F14VectorMap<K, V, S>,
    current: usize,
    _marker: PhantomData<&'a mut F14VectorMap<K, V, S>>,
}

impl<'a, K, V, S> IterMut<'a, K, V, S> {
    pub(crate) fn new(map: &'a mut F14VectorMap<K, V, S>) -> Self {
        Self {
            map: map as *mut F14VectorMap<K, V, S>,
            current: 0,
            _marker: PhantomData,
        }
    }
}

impl<'a, K, V, S> Iterator for IterMut<'a, K, V, S> {
    type Item = (&'a mut K, &'a mut V);
    
    fn next(&mut self) -> Option<Self::Item> {
        // 安全：我们保证每次调用next时只提供一个可变引用
        unsafe {
            let map = &mut *self.map;
            
            while self.current < map.capacity() {
                let index = self.current;
                self.current += 1;
                
                // 只返回FULL状态的槽位
                if let SlotState::Full = map.slot_state(index) {
                    // 修复：直接访问KeyValuePair字段
                    let entry = map.get_entry_mut(index);
                    let key = &mut *entry.key.as_mut_ptr();
                    let value = &mut *entry.value.as_mut_ptr();
                    return Some((key, value));
                }
            }
            
            None
        }
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        unsafe {
            let map = &*self.map;
            (map.len(), Some(map.len()))
        }
    }
}

/// 消耗迭代器
pub struct IntoIter<K, V, S> {
    map: F14VectorMap<K, V, S>,
    current: usize,
}

impl<K, V, S> IntoIter<K, V, S> {
    pub(crate) fn new(map: F14VectorMap<K, V, S>) -> Self {
        Self { map, current: 0 }
    }
}

impl<K, V, S> Iterator for IntoIter<K, V, S> {
    type Item = (K, V);
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < self.map.capacity() {
            let index = self.current;
            self.current += 1;
            
            // 只处理FULL状态的槽位
            if let SlotState::Full = self.map.slot_state(index) {
                unsafe {
                    // 获取元素后标记为空
                    let (key, value) = self.map.replace_slot_state(index, SlotState::Empty);
                    // 不再需要手动减少长度，因为 replace_slot_state 已经处理了
                    return Some((key, value));
                }
            }
        }
        
        None
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.map.len(), Some(self.map.len()))
    }
}

impl<K, V, S> ExactSizeIterator for IntoIter<K, V, S> {
    fn len(&self) -> usize {
        self.map.len()
    }
}