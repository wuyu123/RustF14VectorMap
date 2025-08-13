//! F14VectorMap 基准测试

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use f14vectormap::{traits::HasherExt, F14VectorMap};
use std::collections::HashMap;

const SIZE: usize = 1000;

fn bench_f14_insert(c: &mut Criterion) {
    c.bench_function("f14_insert", |b| {

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
     
        b.iter(|| {
            let mut map: HashMap<usize, usize, FixedHasher> = 
                HashMap::with_hasher(FixedHasher);
            for i in 0..SIZE {
                map.insert(i, i).unwrap();
            }
        })
    });
}

fn bench_std_insert(c: &mut Criterion) {
    c.bench_function("std_insert", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for i in 0..SIZE {
                map.insert(i, i);
            }
        })
    });
}

fn bench_f14_get(c: &mut Criterion) {
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
     let mut map: HashMap<usize, usize, FixedHasher> = 
                HashMap::with_hasher(FixedHasher);
    for i in 0..SIZE {
        map.insert(i, i).unwrap();
    }
    
    c.bench_function("f14_get", |b| {
        b.iter(|| {
            for i in 0..SIZE {
                black_box(map.get(&i));
            }
        })
    });
}

fn bench_std_get(c: &mut Criterion) {
    let mut map = HashMap::new();
    for i in 0..SIZE {
        map.insert(i, i);
    }
    
    c.bench_function("std_get", |b| {
        b.iter(|| {
            for i in 0..SIZE {
                black_box(map.get(&i));
            }
        })
    });
}

fn bench_f14_remove(c: &mut Criterion) {
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
     let mut map: HashMap<usize, usize, FixedHasher> = 
                HashMap::with_hasher(FixedHasher);
    for i in 0..SIZE {
        map.insert(i, i).unwrap();
    }
    
    c.bench_function("f14_remove", |b| {
        b.iter(|| {
            for i in 0..SIZE {
                black_box(map.remove(&i));
            }
        })
    });
}

fn bench_std_remove(c: &mut Criterion) {
    let mut map = HashMap::new();
    for i in 0..SIZE {
        map.insert(i, i);
    }
    
    c.bench_function("std_remove", |b| {
        b.iter(|| {
            for i in 0..SIZE {
                black_box(map.remove(&i));
            }
        })
    });
}

fn bench_f14_iter(c: &mut Criterion) {
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
     let mut map: HashMap<usize, usize, FixedHasher> = 
                HashMap::with_hasher(FixedHasher);
    for i in 0..SIZE {
        map.insert(i, i).unwrap();
    }
    
    c.bench_function("f14_iter", |b| {
        b.iter(|| {
            for (k, v) in map.iter() {
                black_box((k, v));
            }
        })
    });
}

fn bench_std_iter(c: &mut Criterion) {
    let mut map = HashMap::new();
    for i in 0..SIZE {
        map.insert(i, i);
    }
    
    c.bench_function("std_iter", |b| {
        b.iter(|| {
            for (k, v) in map.iter() {
                black_box((k, v));
            }
        })
    });
}

fn bench_f14_high_collision(c: &mut Criterion) {
    // 使用固定哈希器创建高冲突场景
    #[derive(Clone)]
    struct FixedHasher;
    
    impl std::hash::BuildHasher for FixedHasher {
        type Hasher = FixedHasherImpl;
        
        fn build_hasher(&self) -> Self::Hasher {
            FixedHasherImpl(0)
        }
    }
    
    #[derive(Default)]
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
    
    c.bench_function("f14_high_collision", |b| {
        b.iter(|| {
            // 显式指定类型
            let mut map: F14VectorMap<usize, usize, FixedHasher> = 
                F14VectorMap::with_hasher(FixedHasher).unwrap();
            
            for i in 0..SIZE {
                map.insert(i, i).unwrap();
            }
            
            for i in 0..SIZE {
                black_box(map.get(&i));
            }
        })
    });
}

fn bench_std_high_collision(c: &mut Criterion) {
    // 使用固定哈希器创建高冲突场景
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
    
    c.bench_function("std_high_collision", |b| {
        b.iter(|| {
            // 显式指定类型
            let mut map: F14VectorMap<usize, usize, FixedHasher> = 
                F14VectorMap::with_hasher(FixedHasher).unwrap();
            
            for i in 0..SIZE {
                map.insert(i, i);
            }
            
            for i in 0..SIZE {
                black_box(map.get(&i));
            }
        })
    });
}

fn bench_f14_rebuild(c: &mut Criterion) {
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
    let mut map: F14VectorMap<usize, usize, FixedHasher> = 
                F14VectorMap::with_hasher(FixedHasher).unwrap();
    for i in 0..SIZE {
        map.insert(i, i).unwrap();
    }
    
    // 删除一半元素创建墓碑
    for i in 0..SIZE/2 {
        map.remove(&i).unwrap();
    }
    
    c.bench_function("f14_rebuild", |b| {
        b.iter(|| {
            map.rebuild().unwrap();
        })
    });

     // 验证重建后墓碑数量为0
    assert_eq!(map.deleted_count(), 0);
    // 验证元素数量减半
    assert_eq!(map.len(), SIZE/2);
}

fn bench_f14_clear(c: &mut Criterion) {
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
     let mut map: HashMap<usize, usize, FixedHasher> = 
                HashMap::with_hasher(FixedHasher);
    for i in 0..SIZE {
        map.insert(i, i).unwrap();
    }
    
    c.bench_function("f14_clear", |b| {
        b.iter(|| {
            map.clear();
        })
    });
}

criterion_group!(
    benches,
    bench_f14_insert,
    bench_std_insert,
    bench_f14_get,
    bench_std_get,
    bench_f14_remove,
    bench_std_remove,
    bench_f14_iter,
    bench_std_iter,
    bench_f14_high_collision,
    bench_std_high_collision,
    bench_f14_rebuild,
    bench_f14_clear
);
criterion_main!(benches);