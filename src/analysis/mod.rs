use crate::mir::ProjectMIR;
use std::collections::HashMap;
mod liveness;
pub use liveness::{LivenessResult, run_liveness_analysis};

/// 单个函数的所有分析结果
pub struct PerFunctionResults {
    pub liveness: LivenessResult,
    // 未来可扩展更多分析结果字段
}

/// 项目级分析结果：函数名 -> 分析结果
pub struct ProjectAnalysisResults {
    pub results: HashMap<String, PerFunctionResults>,
}

/// 分析管理器，调度所有分析
pub struct AnalysisManager<'a> {
    pub project_mir: &'a ProjectMIR,
}

impl<'a> AnalysisManager<'a> {
    pub fn new(project_mir: &'a ProjectMIR) -> Self {
        Self { project_mir }
    }

    /// 对所有函数运行所有分析
    pub fn run_all_passes(&self) -> ProjectAnalysisResults {
        let mut results = HashMap::new();
        for (name, func) in &self.project_mir.functions {
            // 1. 活跃变量分析
            let liveness = run_liveness_analysis(func);
            // 2. 未来可扩展更多分析
            let per_func = PerFunctionResults { liveness };
            results.insert(name.clone(), per_func);
        }
        ProjectAnalysisResults { results }
    }
}
