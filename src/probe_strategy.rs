// src/probe_strategy.rs

use crate::simd_utils::CHUNK_SIZE;

/// 混合探测策略 - 组内线性，组间二次哈希
pub struct HybridProbeStrategy {
    group_index: usize,      // 当前组索引（0..group_count-1）
    group_count: usize,      // 总分组数
    step: usize,            // 组间步长（必须是奇数）
    group_probe_count: usize, // 当前组内探测次数
    global_probe_count: usize, // 全局探测计数
}

impl HybridProbeStrategy {
    const IN_GROUP_LIMIT: usize = 4; // 组内最大探测次数
    
    pub fn new(initial_group: usize, group_count: usize, step: usize) -> Self {
        // 将绝对索引转换为组索引
        let group_index = initial_group / CHUNK_SIZE;
        
        Self {
            group_index,
            group_count,
            step,
            group_probe_count: 0,
            global_probe_count: 0,
        }
    }
    
    /// 获取下一个探测位置
    pub fn next(&mut self) -> Option<usize> {
        if self.global_probe_count >= self.group_count * 2 {
            return None;
        }
        
        // 组内线性探测
        if self.group_probe_count < Self::IN_GROUP_LIMIT {
            let slot_in_group = self.group_probe_count;
            self.group_probe_count += 1;
            
            // 计算绝对索引 = 组索引 * CHUNK_SIZE + 组内槽位
            let absolute_index = self.group_index * CHUNK_SIZE + slot_in_group;
            return Some(absolute_index);
        }
        
        // 组间二次哈希探测
        self.global_probe_count += 1;
        self.group_index = (self.group_index + self.step) % self.group_count;
        self.group_probe_count = 0;
        self.next()  // 递归调用处理新组
    }
}