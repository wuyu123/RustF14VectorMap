//! F14VectorMap 集成测试


use f14vectormap::{simd_utils, traits::HasherExt, F14VectorMap, MapError};
use std::{hash::RandomState, println as info}; // 使用 info! 宏替代 println!
// 定义通用的 FixedHasher 和 FixedHasherImpl
#[derive(Clone)]
struct FixedHasher;

impl std::hash::BuildHasher for FixedHasher {
    type Hasher = FixedHasherImpl;
    
    fn build_hasher(&self) -> Self::Hasher {
        FixedHasherImpl(0)
    }
}

struct FixedHasherImpl(u64);
impl std::hash::Hasher for FixedHasherImpl {
    fn write(&mut self, _: &[u8]) {}
    fn finish(&self) -> u64 { self.0 }
}



impl HasherExt for FixedHasherImpl {
    fn finish_ext(&self) -> (u64, u8) {
        (self.0, 0)
    }
}

#[test]
fn test_basic_operations() {
    // 使用 new() 方法创建实例，而不是 with_hasher()
    // let mut map: F14VectorMap<&'static str, i32, FixedHasher> = F14VectorMap::with_hasher(FixedHasher).unwrap();
    let mut map = F14VectorMap::<&str, i32, RandomState>::new().unwrap();
    assert!(map.is_empty());
    
    // 插入
    assert_eq!(map.insert("key1", 100), Ok(None));
    assert_eq!(map.len(), 1);
    assert!(!map.is_empty());
    
    // 获取
    assert_eq!(map.get("key1"), Some(&100));
    assert_eq!(map.get("key2"), None);
    
    // 更新
    assert_eq!(map.insert("key1", 200), Ok(Some(100))); // 返回旧值 100
    assert_eq!(map.get("key1"), Some(&200)); // 现在值是 200
    
    // 移除
    assert_eq!(map.remove("key1"), Some(200)); // 返回当前值 200
    assert_eq!(map.len(), 0);
}

#[test]
fn test_capacity_growth() {
    let mut map = F14VectorMap::<usize, usize, RandomState>::new().unwrap();
    assert_eq!(map.capacity(), 0); // 初始容量为0
    
    info!("开始插入元素...");
    // 插入元素触发扩容
    for i in 0..20 {
        let result = map.insert(i, i);
        assert!(result.is_ok(), "插入失败 i={}: {:?}", i, result.err());
        assert_eq!(map.len(), i + 1, "插入后长度不匹配 i={}", i);
        info!("插入 {}: len={}, capacity={}", i, map.len(), map.capacity());
    }
    
    info!("插入完成: len={}, capacity={}", map.len(), map.capacity());
    assert_eq!(map.len(), 20, "最终长度应为20");
    assert!(map.capacity() >= 32, "容量应至少为32，实际为 {}", map.capacity());
}

#[test]
fn test_iterators() {
    let mut map = F14VectorMap::<i32, &str, RandomState>::new().unwrap();
    map.insert(1, "a").unwrap();
    map.insert(2, "b").unwrap();
    map.insert(3, "c").unwrap();
    
    // 不可变迭代
    let mut iter = map.iter();
    assert_eq!(iter.next(), Some((&1, &"a")));
    assert_eq!(iter.next(), Some((&2, &"b")));
    assert_eq!(iter.next(), Some((&3, &"c")));
    assert_eq!(iter.next(), None);
    
    // 可变迭代
    for (_, v) in map.iter_mut() {
        *v = "x";
    }
    assert_eq!(map.get(&1), Some(&"x"));
    
    // 消耗迭代
    let mut into_iter = map.into_iter();
    assert_eq!(into_iter.next(), Some((1, "x")));
    assert_eq!(into_iter.next(), Some((2, "x")));
    assert_eq!(into_iter.next(), Some((3, "x")));
    assert_eq!(into_iter.next(), None);
}

#[test]
fn test_clear() {
     let mut map = F14VectorMap::<i32, i32, RandomState>::new().unwrap();
    map.insert(1, 1).unwrap();
    map.insert(2, 2).unwrap();
    map.clear();
    assert!(map.is_empty());
    assert_eq!(map.get(&1), None);
}

#[test]
fn test_find_match() {
    // 创建控制字节数组
    let mut ctrls = [0u8; 16];
    ctrls[0] = 10;
    ctrls[1] = 20;
    ctrls[2] = 30;
    ctrls[3] = 20; // 第二个匹配
    
    // 查找片段20
    let result = unsafe { simd_utils::simd_find_match(ctrls.as_ptr(), 20) };
    assert_eq!(result, Some(1)); // 应该返回第一个匹配位置
    
    // 查找片段30
    let result = unsafe { simd_utils::simd_find_match(ctrls.as_ptr(), 30) };
    assert_eq!(result, Some(2));
    
    // 查找不存在的片段
    let result = unsafe { simd_utils::simd_find_match(ctrls.as_ptr(), 40) };
    assert_eq!(result, None);
}

#[test]
fn test_insert_full() {
    let mut map = F14VectorMap::<i32, i32, RandomState>::new().unwrap();
    
    // 插入足够多的元素使表满载
    for i in 0..20 {
        map.insert(i, i).unwrap();
        // 验证所有已插入元素
        for j in 0..=i {
            assert_eq!(map.get(&j), Some(&j), "插入后键 {} 未找到", j);
        }
    }
    
    // 验证所有元素存在
    for i in 0..20 {
        assert_eq!(map.get(&i), Some(&i), "键 {} 未找到", i);
    }
}

#[test]
fn test_remove_boundary() {
     let mut map = F14VectorMap::<i32, i32, RandomState>::new().unwrap();
    
    // 插入元素
    for i in 0..100 {
        map.insert(i, i).unwrap();
    }
   
    // 删除前50个元素
    for i in 0..50 {
        map.remove(&i).unwrap();
    }
    
    // 验证边界值
    assert_eq!(map.get(&49), None); // 已删除
    assert_eq!(map.get(&50), Some(&50)); // 存在
    assert_eq!(map.get(&51), Some(&51)); // 存在
}
#[test]
fn test_rebuild() {
     let mut map = F14VectorMap::<i32, i32, RandomState>::new().unwrap();
    
    // 插入并删除元素创建墓碑
    for i in 0..100 {
        map.insert(i, i).unwrap();
    }
    
    // 删除一半元素
    for i in 0..50 {
        map.remove(&i).unwrap();
    }
    // 验证删除后，剩余元素存在
    for i in 50..100 {
        assert_eq!(map.get(&i), Some(&i), "删除后，键 {} 应该存在", i);
    }
    // 重建表
    map.rebuild().unwrap();
   
    // 验证所有剩余元素
    for i in 50..100 {
        assert_eq!(map.get(&i), Some(&i), "键 {} 未找到", i);
    }
    
    // 特别验证边界值
    assert_eq!(map.get(&50), Some(&50));
    assert_eq!(map.get(&51), Some(&51));
    assert_eq!(map.get(&99), Some(&99));
}

#[test]
fn test_high_collision() {
    // 使用固定哈希器创建高冲突场景
    let mut map = F14VectorMap::with_hasher(FixedHasher).unwrap(); // 使用 unwrap 获取 map
    
    // 插入100个元素（全部哈希冲突）
    for i in 0..100 {
        map.insert(i, i).unwrap();
    }
    
    // 验证所有元素存在
    for i in 0..100 {
        assert_eq!(map.get(&i), Some(&i));
    }
}

#[test]
fn test_error_handling() {
    // 测试超大容量错误
    let huge_capacity = usize::MAX / 16 + 1;
    let result = F14VectorMap::<i32, i32>::with_capacity(huge_capacity);
    
    // 验证返回错误
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), MapError::CapacityExceeded);
}

#[test]
fn test_debug_format() {
    let mut map = F14VectorMap::with_hasher(FixedHasher).unwrap();
    map.insert("a", 1).unwrap();
    map.insert("b", 2).unwrap();
    
    let debug_str = format!("{:?}", map);
    assert!(debug_str.contains("\"a\": 1"));
    assert!(debug_str.contains("\"b\": 2"));
}

#[test]
fn test_get_performance() {
     let mut map = F14VectorMap::<i32, i32, RandomState>::new().unwrap();
    
    // 插入1000个元素
    for i in 0..1000 {
        map.insert(i, i).unwrap();
    }
    
    // 测量查找性能
    let start = std::time::Instant::now();
    for i in 0..1000 {
        assert_eq!(map.get(&i), Some(&i));
    }
    let duration = start.elapsed();
    
    println!("查找1000个元素耗时: {:?}", duration);
    assert!(duration < std::time::Duration::from_millis(1));
}

#[test]
fn test_remove_performance() {
     let mut map = F14VectorMap::<i32, i32, RandomState>::new().unwrap();
    
    // 插入1000个元素
    for i in 0..1000 {
        map.insert(i, i).unwrap();
    }
    
    // 测量移除性能
    let start = std::time::Instant::now();
    for i in 0..1000 {
        assert_eq!(map.remove(&i), Some(i));
    }
    let duration = start.elapsed();
    
    println!("移除1000个元素耗时: {:?}", duration);
    assert!(duration < std::time::Duration::from_millis(1));
}