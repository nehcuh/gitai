// 依赖图模块 - 用于构建和分析代码间的依赖关系
use crate::tree_sitter::{ClassInfo, FunctionInfo, StructuralSummary};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// 依赖图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// 图中的所有节点
    pub nodes: HashMap<String, Node>,
    /// 图中的所有边
    pub edges: Vec<Edge>,
    /// 邻接表表示（用于快速查找）
    #[serde(skip)]
    adjacency_list: HashMap<String, Vec<String>>,
    /// 反向邻接表（用于查找依赖某节点的所有节点）
    #[serde(skip)]
    reverse_adjacency_list: HashMap<String, Vec<String>>,
}

/// 图节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// 节点唯一标识
    pub id: String,
    /// 节点类型
    pub node_type: NodeType,
    /// 节点元数据
    pub metadata: NodeMetadata,
    /// 节点的重要性分数
    pub importance_score: f32,
}

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// 函数节点
    Function(FunctionNode),
    /// 类/结构体节点
    Class(ClassNode),
    /// 模块节点
    Module(ModuleNode),
    /// 文件节点
    File(FileNode),
}

/// 函数节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    pub name: String,
    pub visibility: Option<Visibility>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub is_async: bool,
}

/// 类节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassNode {
    pub name: String,
    pub visibility: Option<Visibility>,
    pub is_abstract: bool,
    pub methods: Vec<String>,
    pub fields: Vec<String>,
}

/// 模块节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleNode {
    pub name: String,
    pub path: String,
    pub exports: Vec<String>,
}

/// 文件节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    pub path: String,
    pub language: String,
    pub size: usize,
}

/// 节点元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// 所在文件路径
    pub file_path: String,
    /// 起始行号
    pub start_line: usize,
    /// 结束行号
    pub end_line: usize,
    /// 复杂度
    pub complexity: u32,
    /// 创建时间戳
    pub created_at: i64,
}

/// 图的边（依赖关系）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    /// 起始节点ID
    pub from: String,
    /// 目标节点ID
    pub to: String,
    /// 边的类型
    pub edge_type: EdgeType,
    /// 边的权重（依赖强度）
    pub weight: f32,
    /// 边的元数据
    pub metadata: Option<EdgeMetadata>,
}

/// 边的类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    /// 函数调用
    Calls,
    /// 导入/使用
    Imports,
    /// 导出（文件/模块导出特定符号）
    Exports,
    /// 继承
    Inherits,
    /// 实现接口
    Implements,
    /// 使用类型
    Uses,
    /// 引用
    References,
    /// 包含（如文件包含函数）
    Contains,
    /// 依赖
    DependsOn,
}

/// 边的元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMetadata {
    /// 调用次数（对于Calls类型）
    pub call_count: Option<u32>,
    /// 是否是强依赖
    pub is_strong_dependency: bool,
    /// 额外信息
    pub notes: Option<String>,
}

/// 可见性
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Package,
    Internal,
    Unknown,
}

impl Visibility {
    fn from_str(s: &str) -> Visibility {
        match s.to_lowercase().as_str() {
            "public" => Visibility::Public,
            "private" => Visibility::Private,
            "protected" => Visibility::Protected,
            "package" | "default" => Visibility::Package,
            "internal" => Visibility::Internal,
            _ => Visibility::Unknown,
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
            adjacency_list: HashMap::new(),
            reverse_adjacency_list: HashMap::new(),
        }
    }

    /// 从结构化摘要构建依赖图
    pub fn from_structural_summary(summary: &StructuralSummary, file_path: &str) -> Self {
        let mut graph = Self::new();

        // 添加文件节点
        let file_id = format!("file:{file_path}");
        graph.add_file_node(&file_id, file_path);

        // 添加函数节点
        for func in &summary.functions {
            let func_id = format!("func:{}::{}", file_path, func.name);
            graph.add_function_node(&func_id, func, file_path);

            // 文件包含函数
            graph.add_edge(Edge {
                from: file_id.clone(),
                to: func_id.clone(),
                edge_type: EdgeType::Contains,
                weight: 1.0,
                metadata: None,
            });
        }

        // 添加类节点
        for class in &summary.classes {
            let class_id = format!("class:{}::{}", file_path, class.name);
            graph.add_class_node(&class_id, class, file_path);

            // 文件包含类
            graph.add_edge(Edge {
                from: file_id.clone(),
                to: class_id.clone(),
                edge_type: EdgeType::Contains,
                weight: 1.0,
                metadata: None,
            });
        }

        // 处理导入关系（为每个导入创建模块节点）
        for import in &summary.imports {
            let module_id = format!("module:{import}");
            graph.add_module_node(&module_id, import);
            graph.add_edge(Edge {
                from: file_id.clone(),
                to: module_id,
                edge_type: EdgeType::Imports,
                weight: 0.8,
                metadata: None,
            });
        }

        // 处理导出关系（将文件导出符号与本文件项目相连）
        for export in &summary.exports {
            // 函数优先
            let func_id = format!("func:{file_path}::{export}");
            if graph.nodes.contains_key(&func_id) {
                graph.add_edge(Edge {
                    from: file_id.clone(),
                    to: func_id,
                    edge_type: EdgeType::Exports,
                    weight: 0.9,
                    metadata: None,
                });
                continue;
            }
            // 类/结构体
            let class_id = format!("class:{file_path}::{export}");
            if graph.nodes.contains_key(&class_id) {
                graph.add_edge(Edge {
                    from: file_id.clone(),
                    to: class_id,
                    edge_type: EdgeType::Exports,
                    weight: 0.9,
                    metadata: None,
                });
            }
        }

        // 根据调用信息添加 Calls 边（仅限当前文件内可解析的函数）
        if !summary.calls.is_empty() {
            // 建立函数名 -> 节点ID 的映射（同文件）
            let mut name_to_id: HashMap<&str, String> = HashMap::new();
            for func in &summary.functions {
                let id = format!("func:{}::{}", file_path, func.name);
                name_to_id.insert(func.name.as_str(), id);
            }
            // 为快速定位 caller，缓存函数范围
            let func_ranges: Vec<(String, usize, usize)> = summary
                .functions
                .iter()
                .map(|f| {
                    (
                        format!("func:{}::{}", file_path, f.name),
                        f.line_start,
                        f.line_end,
                    )
                })
                .collect();
            for call in &summary.calls {
                // 确定 caller：调用所在的函数范围包含 call.line
                if let Some((caller_id, _, _)) = func_ranges
                    .iter()
                    .find(|(_, s, e)| call.line >= *s && call.line <= *e)
                    .cloned()
                {
                    // 仅在 callee 存在于同文件函数映射中时创建 Calls 边
                    if let Some(callee_id) = name_to_id.get(call.callee.as_str()) {
                        graph.add_edge(Edge {
                            from: caller_id,
                            to: callee_id.clone(),
                            edge_type: EdgeType::Calls,
                            weight: 1.0,
                            metadata: Some(EdgeMetadata {
                                call_count: Some(1),
                                is_strong_dependency: true,
                                notes: None,
                            }),
                        });
                    }
                }
            }
        }

        graph.rebuild_adjacency_lists();
        graph
    }

    /// 添加节点
    pub fn add_node(&mut self, node: Node) {
        self.nodes.insert(node.id.clone(), node);
    }

    /// 添加函数节点
    fn add_function_node(&mut self, id: &str, func: &FunctionInfo, file_path: &str) {
        let node = Node {
            id: id.to_string(),
            node_type: NodeType::Function(FunctionNode {
                name: func.name.clone(),
                visibility: func.visibility.as_deref().map(Visibility::from_str),
                parameters: func.parameters.clone(),
                return_type: func.return_type.clone(),
                is_async: func.is_async,
            }),
            metadata: NodeMetadata {
                file_path: file_path.to_string(),
                start_line: func.line_start,
                end_line: func.line_end,
                complexity: 1, // TODO: 可与复杂度分析整合
                created_at: Self::unix_ts_now(),
            },
            importance_score: Self::calculate_function_importance(func),
        };
        self.add_node(node);
    }

    /// 添加类节点
    fn add_class_node(&mut self, id: &str, class: &ClassInfo, file_path: &str) {
        let node = Node {
            id: id.to_string(),
            node_type: NodeType::Class(ClassNode {
                name: class.name.clone(),
                visibility: None, // 源信息暂无可见性
                is_abstract: class.is_abstract,
                methods: class.methods.clone(),
                fields: class.fields.clone(),
            }),
            metadata: NodeMetadata {
                file_path: file_path.to_string(),
                start_line: class.line_start,
                end_line: class.line_end,
                complexity: class.methods.len() as u32,
                created_at: Self::unix_ts_now(),
            },
            importance_score: Self::calculate_class_importance(class),
        };
        self.add_node(node);
    }

    /// 添加文件节点
    fn add_file_node(&mut self, id: &str, file_path: &str) {
        let size = std::fs::metadata(file_path)
            .map(|m| m.len() as usize)
            .unwrap_or(0);
        let node = Node {
            id: id.to_string(),
            node_type: NodeType::File(FileNode {
                path: file_path.to_string(),
                language: Self::detect_language(file_path),
                size,
            }),
            metadata: NodeMetadata {
                file_path: file_path.to_string(),
                start_line: 0,
                end_line: 0,
                complexity: 0,
                created_at: Self::unix_ts_now(),
            },
            importance_score: 0.5,
        };
        self.add_node(node);
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// 基于现有图节点尝试解析并添加跨文件调用边
    /// 规则：
    /// - 通过 caller_file_path 和 call_line 定位调用发生的函数（作为 caller）
    /// - 在全图中按函数名唯一匹配 callee（若有多个同名函数则跳过，以避免歧义）
    pub fn add_resolved_call(
        &mut self,
        caller_file_path: &str,
        call_line: usize,
        callee_name: &str,
    ) {
        // 定位 caller
        let caller_id_opt = self
            .nodes
            .iter()
            .filter_map(|(id, node)| match &node.node_type {
                NodeType::Function(_)
                    if node.metadata.file_path == caller_file_path
                        && node.metadata.start_line <= call_line
                        && node.metadata.end_line >= call_line =>
                {
                    Some(id.clone())
                }
                _ => None,
            })
            .next();

        if caller_id_opt.is_none() {
            return;
        }
        let caller_id = caller_id_opt.unwrap();

        // 唯一定位 callee（同名函数必须唯一）
        let mut matches: Vec<String> = self
            .nodes
            .iter()
            .filter_map(|(id, node)| match &node.node_type {
                NodeType::Function(f) if f.name == callee_name => Some(id.clone()),
                _ => None,
            })
            .collect();

        if matches.len() != 1 {
            return;
        }
        let callee_id = matches.pop().unwrap();

        self.add_edge(Edge {
            from: caller_id,
            to: callee_id,
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: Some(EdgeMetadata {
                call_count: Some(1),
                is_strong_dependency: true,
                notes: None,
            }),
        });
    }

    /// 添加模块节点（用于 imports 等场景）
    fn add_module_node(&mut self, id: &str, import_path: &str) {
        let name = import_path
            .rsplit(['.', '/', ':'])
            .next()
            .unwrap_or(import_path)
            .to_string();
        let node = Node {
            id: id.to_string(),
            node_type: NodeType::Module(ModuleNode {
                name,
                path: import_path.to_string(),
                exports: vec![],
            }),
            metadata: NodeMetadata {
                file_path: import_path.to_string(),
                start_line: 0,
                end_line: 0,
                complexity: 0,
                created_at: Self::unix_ts_now(),
            },
            importance_score: 0.4,
        };
        self.add_node(node);
    }

    fn unix_ts_now() -> i64 {
        match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            Ok(d) => d.as_secs() as i64,
            Err(_) => 0,
        }
    }

    /// 重建邻接表（在添加完所有节点和边后调用）
    pub fn rebuild_adjacency_lists(&mut self) {
        self.adjacency_list.clear();
        self.reverse_adjacency_list.clear();

        for edge in &self.edges {
            self.adjacency_list
                .entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());

            self.reverse_adjacency_list
                .entry(edge.to.clone())
                .or_default()
                .push(edge.from.clone());
        }
    }

    /// 获取节点的所有直接依赖
    pub fn get_dependencies(&self, node_id: &str) -> Vec<&String> {
        self.adjacency_list
            .get(node_id)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// 获取依赖某节点的所有节点
    pub fn get_dependents(&self, node_id: &str) -> Vec<&String> {
        self.reverse_adjacency_list
            .get(node_id)
            .map(|deps| deps.iter().collect())
            .unwrap_or_default()
    }

    /// 使用BFS计算从某个节点开始的影响范围
    pub fn calculate_impact_scope(
        &self,
        changed_node_id: &str,
        max_depth: usize,
    ) -> HashMap<String, usize> {
        let mut visited = HashMap::new();
        let mut queue = VecDeque::new();

        queue.push_back((changed_node_id.to_string(), 0));
        visited.insert(changed_node_id.to_string(), 0);

        while let Some((node_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            // 获取所有依赖当前节点的节点（反向传播）
            if let Some(dependents) = self.reverse_adjacency_list.get(&node_id) {
                for dependent in dependents {
                    if !visited.contains_key(dependent) {
                        visited.insert(dependent.clone(), depth + 1);
                        queue.push_back((dependent.clone(), depth + 1));
                    }
                }
            }
        }

        visited
    }
    
    /// 计算 PageRank 分数
    /// 使用经典的 PageRank 算法，考虑节点的入度和出度
    pub fn calculate_pagerank(
        &mut self,
        damping_factor: f32,
        max_iterations: usize,
        tolerance: f32,
    ) -> HashMap<String, f32> {
        let n = self.nodes.len() as f32;
        if n == 0.0 {
            return HashMap::new();
        }
        
        // 初始化 PageRank 分数
        let mut pagerank: HashMap<String, f32> = HashMap::new();
        for node_id in self.nodes.keys() {
            pagerank.insert(node_id.clone(), 1.0 / n);
        }
        
        // 预计算每个节点的出度
        let mut out_degree: HashMap<String, usize> = HashMap::new();
        for node_id in self.nodes.keys() {
            let degree = self.adjacency_list.get(node_id)
                .map(|neighbors| neighbors.len())
                .unwrap_or(0);
            out_degree.insert(node_id.clone(), degree);
        }
        
        // 迭代计算 PageRank
        for iteration in 0..max_iterations {
            let mut new_pagerank: HashMap<String, f32> = HashMap::new();
            let mut total_change = 0.0;
            
            for node_id in self.nodes.keys() {
                let mut rank = (1.0 - damping_factor) / n;
                
                // 从所有指向该节点的节点收集 PageRank 贡献
                if let Some(incoming) = self.reverse_adjacency_list.get(node_id) {
                    for source in incoming {
                        if let Some(&source_rank) = pagerank.get(source) {
                            if let Some(&out_deg) = out_degree.get(source) {
                                if out_deg > 0 {
                                    // 考虑边的权重
                                    let edge_weight = self.get_edge_weight(source, node_id).unwrap_or(1.0);
                                    rank += damping_factor * source_rank * edge_weight / out_deg as f32;
                                }
                            }
                        }
                    }
                }
                
                // 处理没有出边的节点（dangling nodes）
                for (source_id, &source_rank) in &pagerank {
                    if let Some(&out_deg) = out_degree.get(source_id) {
                        if out_deg == 0 {
                            rank += damping_factor * source_rank / n;
                        }
                    }
                }
                
                let old_rank = pagerank.get(node_id).copied().unwrap_or(0.0);
                total_change += (rank - old_rank).abs();
                new_pagerank.insert(node_id.clone(), rank);
            }
            
            pagerank = new_pagerank;
            
            // 检查收敛
            if iteration > 0 && total_change < tolerance {
                log::debug!("PageRank 在第 {} 次迭代后收敛", iteration + 1);
                break;
            }
        }
        
        // 更新节点的 importance_score
        for (node_id, &score) in &pagerank {
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.importance_score = score;
            }
        }
        
        pagerank
    }
    
    /// 获取边的权重
    fn get_edge_weight(&self, from: &str, to: &str) -> Option<f32> {
        self.edges.iter()
            .find(|e| e.from == from && e.to == to)
            .map(|e| e.weight)
    }
    
    /// 计算加权影响传播
    /// 使用 PageRank 分数和边权重来计算影响传播
    pub fn calculate_weighted_impact(
        &self,
        start_node: &str,
        impact_strength: f32,
        decay_factor: f32,
        min_impact: f32,
    ) -> HashMap<String, f32> {
        let mut impact_scores: HashMap<String, f32> = HashMap::new();
        let mut queue = VecDeque::new();
        
        // 初始节点的影响力
        queue.push_back((start_node.to_string(), impact_strength));
        impact_scores.insert(start_node.to_string(), impact_strength);
        
        while let Some((node_id, current_impact)) = queue.pop_front() {
            // 传播影响到依赖节点
            if let Some(neighbors) = self.reverse_adjacency_list.get(&node_id) {
                for neighbor in neighbors {
                    // 计算传播的影响力
                    let edge_weight = self.get_edge_weight(&node_id, neighbor).unwrap_or(1.0);
                    let neighbor_importance = self.nodes.get(neighbor)
                        .map(|n| n.importance_score)
                        .unwrap_or(0.5);
                    
                    // 影响力 = 当前影响 * 边权重 * 节点重要性 * 衰减因子
                    let propagated_impact = current_impact * edge_weight * neighbor_importance * decay_factor;
                    
                    if propagated_impact >= min_impact {
                        let existing_impact = impact_scores.get(neighbor).copied().unwrap_or(0.0);
                        if propagated_impact > existing_impact {
                            impact_scores.insert(neighbor.clone(), propagated_impact);
                            queue.push_back((neighbor.clone(), propagated_impact));
                        }
                    }
                }
            }
        }
        
        impact_scores
    }
    
    /// 识别关键路径（基于 PageRank 分数）
    pub fn find_critical_paths(&self, top_n: usize) -> Vec<Vec<String>> {
        let mut critical_paths = Vec::new();
        
        // 获取 PageRank 分数最高的节点
        let mut nodes_by_importance: Vec<_> = self.nodes.iter()
            .map(|(id, node)| (id.clone(), node.importance_score))
            .collect();
        nodes_by_importance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // 对每个重要节点，找到其关键依赖路径
        for (node_id, _) in nodes_by_importance.iter().take(top_n) {
            let paths = self.find_dependency_paths(node_id, 3);
            for path in paths {
                if path.len() > 1 {
                    critical_paths.push(path);
                }
            }
        }
        
        critical_paths
    }
    
    /// 查找从指定节点开始的依赖路径
    fn find_dependency_paths(&self, start: &str, max_depth: usize) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current_path = vec![start.to_string()];
        self.dfs_paths(start, &mut current_path, &mut paths, max_depth);
        paths
    }
    
    /// 深度优先搜索依赖路径
    fn dfs_paths(
        &self,
        node: &str,
        current_path: &mut Vec<String>,
        all_paths: &mut Vec<Vec<String>>,
        max_depth: usize,
    ) {
        if current_path.len() >= max_depth {
            all_paths.push(current_path.clone());
            return;
        }
        
        if let Some(neighbors) = self.adjacency_list.get(node) {
            if neighbors.is_empty() {
                all_paths.push(current_path.clone());
            } else {
                for neighbor in neighbors {
                    if !current_path.contains(neighbor) {
                        current_path.push(neighbor.clone());
                        self.dfs_paths(neighbor, current_path, all_paths, max_depth);
                        current_path.pop();
                    }
                }
            }
        } else {
            all_paths.push(current_path.clone());
        }
    }

    /// 检测环路
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node_id in self.nodes.keys() {
            if !visited.contains(node_id) {
                self.dfs_cycle_detection(
                    node_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    /// DFS环路检测辅助函数
    fn dfs_cycle_detection(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());
        path.push(node_id.to_string());

        if let Some(neighbors) = self.adjacency_list.get(node_id) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_cycle_detection(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // 找到环路
                    if let Some(start_idx) = path.iter().position(|n| n == neighbor) {
                        cycles.push(path[start_idx..].to_vec());
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(node_id);
    }

    /// 计算节点的中心性（重要性）
    pub fn calculate_centrality(&self, node_id: &str) -> f32 {
        let in_degree = self.get_dependents(node_id).len() as f32;
        let out_degree = self.get_dependencies(node_id).len() as f32;

        // 简单的度中心性计算
        (in_degree + out_degree) / (self.nodes.len() as f32 * 2.0)
    }

    /// 识别关键节点（高中心性节点）
    pub fn identify_critical_nodes(&self, threshold: f32) -> Vec<(&String, f32)> {
        let mut critical_nodes = Vec::new();

        for node_id in self.nodes.keys() {
            let centrality = self.calculate_centrality(node_id);
            if centrality > threshold {
                critical_nodes.push((node_id, centrality));
            }
        }

        critical_nodes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        critical_nodes
    }

    /// 计算函数的重要性分数
    fn calculate_function_importance(func: &FunctionInfo) -> f32 {
        let mut score = 0.5;

        // 公共函数更重要
        if matches!(func.visibility.as_deref(), Some("public")) {
            score += 0.2;
        }

        // 异步函数通常更重要
        if func.is_async {
            score += 0.1;
        }

        // 参数多的函数可能更复杂
        score += (func.parameters.len() as f32) * 0.02;

        score.min(1.0)
    }

    /// 计算类的重要性分数
    fn calculate_class_importance(class: &ClassInfo) -> f32 {
        let mut score = 0.5;

        // 抽象类通常更重要
        if class.is_abstract {
            score += 0.15;
        }

        // 继承关系增加重要性
        if class.extends.is_some() {
            score += 0.1;
        }

        // 实现接口增加重要性
        score += (class.implements.len() as f32) * 0.05;

        // 方法多的类更重要
        score += (class.methods.len() as f32) * 0.01;

        score.min(1.0)
    }

    /// 检测语言类型
    fn detect_language(file_path: &str) -> String {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension {
            "rs" => "rust",
            "java" => "java",
            "py" => "python",
            "js" => "javascript",
            "ts" => "typescript",
            "go" => "go",
            "c" => "c",
            "cpp" | "cc" => "cpp",
            _ => "unknown",
        }
        .to_string()
    }

    /// 获取图的统计信息
    pub fn get_statistics(&self) -> GraphStatistics {
        let internal_edge_count = self
            .edges
            .iter()
            .filter(|e| self.nodes.contains_key(&e.from) && self.nodes.contains_key(&e.to))
            .count();
        GraphStatistics {
            node_count: self.nodes.len(),
            edge_count: internal_edge_count,
            avg_degree: if self.nodes.is_empty() {
                0.0
            } else {
                (internal_edge_count * 2) as f32 / self.nodes.len() as f32
            },
            cycles_count: self.detect_cycles().len(),
            critical_nodes_count: self.identify_critical_nodes(0.1).len(),
        }
    }

    /// 以 Graphviz DOT 格式导出当前图
    pub fn to_dot(&self, options: Option<&DotOptions>) -> String {
        let include_weights = options.map(|o| o.include_weights).unwrap_or(false);
        let highlight_nodes: std::collections::HashSet<String> = options
            .map(|o| o.highlight_nodes.iter().cloned().collect())
            .unwrap_or_default();

        let mut node_ids: Vec<String> = self.nodes.keys().cloned().collect();
        node_ids.sort();
        let mut edges_sorted = self.edges.clone();
        edges_sorted.sort_by(|a, b| {
            let c = a.from.cmp(&b.from);
            if c == std::cmp::Ordering::Equal {
                a.to.cmp(&b.to)
            } else {
                c
            }
        });

        let mut s = String::new();
        s.push_str("digraph G {\n");
        s.push_str("  rankdir=LR;\n");
        s.push_str("  node [fontname=\"Helvetica\"];\n");
        s.push_str("  edge [fontname=\"Helvetica\"];\n");

        // 输出节点
        for id in node_ids {
            if let Some(node) = self.nodes.get(&id) {
                let (shape, fillcolor) = match &node.node_type {
                    NodeType::Function(_) => ("ellipse", "#8ecae6"),
                    NodeType::Class(_) => ("box", "#ffb703"),
                    NodeType::Module(_) => ("folder", "#219ebc"),
                    NodeType::File(_) => ("note", "#adb5bd"),
                };
                let label = match &node.node_type {
                    NodeType::Function(f) => format!("{}()", f.name),
                    NodeType::Class(c) => c.name.clone(),
                    NodeType::Module(m) => m.name.clone(),
                    NodeType::File(f) => f.path.clone(),
                };
                let safe_label = label.replace("\\", "\\\\").replace("\"", "\\\"");
                let mut attrs = format!(
                    "shape=\"{shape}\", style=\"filled\", fillcolor=\"{fillcolor}\", label=\"{safe_label}\"",
                );
                if highlight_nodes.contains(&id) {
                    attrs.push_str(", color=\"red\", penwidth=2");
                }
                s.push_str(&format!("  \"{id}\" [{attrs}];\n"));
            }
        }

        // 输出边
        for e in edges_sorted.iter() {
            let label_type = match e.edge_type {
                EdgeType::Calls => "Calls",
                EdgeType::Imports => "Imports",
                EdgeType::Exports => "Exports",
                EdgeType::Inherits => "Inherits",
                EdgeType::Implements => "Implements",
                EdgeType::Uses => "Uses",
                EdgeType::References => "References",
                EdgeType::Contains => "Contains",
                EdgeType::DependsOn => "DependsOn",
            };
            let color = match e.edge_type {
                EdgeType::Calls => "#1b4332",
                EdgeType::Imports => "#6c757d",
                EdgeType::Exports => "#0077b6",
                EdgeType::Inherits => "#4a4e69",
                EdgeType::Implements => "#2a9d8f",
                EdgeType::Uses => "#264653",
                EdgeType::References => "#8d99ae",
                EdgeType::Contains => "#023047",
                EdgeType::DependsOn => "#e76f51",
            };
            let mut label = label_type.to_string();
            if include_weights {
                label.push_str(&format!(" (w={:.2})", e.weight));
            }
            let safe_label = label.replace("\\", "\\\\").replace("\"", "\\\"");
            s.push_str(&format!(
                "  \"{}\" -> \"{}\" [color=\"{}\", label=\"{}\"];\n",
                e.from, e.to, color, safe_label
            ));
        }

        s.push_str("}\n");
        s
    }

    /// 将 DOT 输出写入文件
    pub fn write_dot_file(&self, path: &str, options: Option<&DotOptions>) -> std::io::Result<()> {
        std::fs::write(path, self.to_dot(options))
    }
}

/// DOT 输出选项
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DotOptions {
    /// 是否在边上包含权重
    pub include_weights: bool,
    /// 高亮的节点ID列表
    pub highlight_nodes: Vec<String>,
}

/// 图的统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub node_count: usize,
    pub edge_count: usize,
    pub avg_degree: f32,
    pub cycles_count: usize,
    pub critical_nodes_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_construction() {
        let mut graph = DependencyGraph::new();

        // 添加节点
        let node1 = Node {
            id: "func1".to_string(),
            node_type: NodeType::Function(FunctionNode {
                name: "test_func".to_string(),
                visibility: Some(Visibility::Public),
                parameters: vec![],
                return_type: None,
                is_async: false,
            }),
            metadata: NodeMetadata {
                file_path: "test.rs".to_string(),
                start_line: 1,
                end_line: 10,
                complexity: 5,
                created_at: 0,
            },
            importance_score: 0.7,
        };

        graph.add_node(node1);

        // 添加边
        graph.add_edge(Edge {
            from: "func1".to_string(),
            to: "func2".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_impact_scope_calculation() {
        let mut graph = DependencyGraph::new();

        // 构建简单的依赖链: A -> B -> C -> D
        for id in ["A", "B", "C", "D"] {
            graph.add_node(Node {
                id: id.to_string(),
                node_type: NodeType::Function(FunctionNode {
                    name: id.to_string(),
                    visibility: Some(Visibility::Public),
                    parameters: vec![],
                    return_type: None,
                    is_async: false,
                }),
                metadata: NodeMetadata {
                    file_path: "test.rs".to_string(),
                    start_line: 1,
                    end_line: 10,
                    complexity: 1,
                    created_at: 0,
                },
                importance_score: 0.5,
            });
        }

        graph.add_edge(Edge {
            from: "B".to_string(),
            to: "A".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "C".to_string(),
            to: "B".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "D".to_string(),
            to: "C".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.rebuild_adjacency_lists();

        // 测试从A开始的影响范围
        let impact = graph.calculate_impact_scope("A", 3);

        assert_eq!(impact.len(), 4); // A, B, C, D都受影响
        assert_eq!(impact.get("A"), Some(&0));
        assert_eq!(impact.get("B"), Some(&1));
        assert_eq!(impact.get("C"), Some(&2));
        assert_eq!(impact.get("D"), Some(&3));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();

        // 构建有环的图: A -> B -> C -> A
        for id in ["A", "B", "C"] {
            graph.add_node(Node {
                id: id.to_string(),
                node_type: NodeType::Function(FunctionNode {
                    name: id.to_string(),
                    visibility: Some(Visibility::Public),
                    parameters: vec![],
                    return_type: None,
                    is_async: false,
                }),
                metadata: NodeMetadata {
                    file_path: "test.rs".to_string(),
                    start_line: 1,
                    end_line: 10,
                    complexity: 1,
                    created_at: 0,
                },
                importance_score: 0.5,
            });
        }

        graph.add_edge(Edge {
            from: "A".to_string(),
            to: "B".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "B".to_string(),
            to: "C".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.add_edge(Edge {
            from: "C".to_string(),
            to: "A".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });

        graph.rebuild_adjacency_lists();

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_dot_export_basic() {
        let mut graph = DependencyGraph::new();
        // 节点
        graph.add_node(Node {
            id: "func:A".to_string(),
            node_type: NodeType::Function(FunctionNode {
                name: "A".to_string(),
                visibility: Some(Visibility::Public),
                parameters: vec![],
                return_type: None,
                is_async: false,
            }),
            metadata: NodeMetadata {
                file_path: "a.rs".to_string(),
                start_line: 1,
                end_line: 3,
                complexity: 1,
                created_at: 0,
            },
            importance_score: 0.5,
        });
        graph.add_node(Node {
            id: "func:B".to_string(),
            node_type: NodeType::Function(FunctionNode {
                name: "B".to_string(),
                visibility: Some(Visibility::Public),
                parameters: vec![],
                return_type: None,
                is_async: false,
            }),
            metadata: NodeMetadata {
                file_path: "b.rs".to_string(),
                start_line: 1,
                end_line: 3,
                complexity: 1,
                created_at: 0,
            },
            importance_score: 0.5,
        });
        // 边
        graph.add_edge(Edge {
            from: "func:A".to_string(),
            to: "func:B".to_string(),
            edge_type: EdgeType::Calls,
            weight: 1.0,
            metadata: None,
        });
        graph.rebuild_adjacency_lists();

        let dot = graph.to_dot(None);
        assert!(dot.contains("digraph G"));
        assert!(dot.contains("\"func:A\""));
        assert!(dot.contains("->"));
        assert!(dot.contains("Calls"));
    }

    #[test]
    fn test_import_nodes_from_summary() {
        let summary = crate::tree_sitter::StructuralSummary {
            language: "rust".to_string(),
            imports: vec!["std::fmt".to_string(), "serde::Serialize".to_string()],
            ..Default::default()
        };

        let graph = DependencyGraph::from_structural_summary(&summary, "src/lib.rs");

        // 应存在文件节点和两个模块节点
        assert!(graph.nodes.contains_key("file:src/lib.rs"));
        assert!(graph.nodes.contains_key("module:std::fmt"));
        assert!(graph.nodes.contains_key("module:serde::Serialize"));

        // 应存在从文件到模块的 Imports 边
        let mut imports_edges = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Imports);
        assert!(imports_edges.any(|e| e.from == "file:src/lib.rs" && e.to == "module:std::fmt"));
        let mut imports_edges2 = graph
            .edges
            .iter()
            .filter(|e| e.edge_type == EdgeType::Imports);
        assert!(imports_edges2
            .any(|e| e.from == "file:src/lib.rs" && e.to == "module:serde::Serialize"));
    }

    #[test]
    fn test_exports_edges_from_summary() {
        use crate::tree_sitter::{FunctionInfo, StructuralSummary};
        let file_path = "src/sample.rs";

        let func = FunctionInfo {
            name: "foo".to_string(),
            parameters: vec![],
            return_type: None,
            line_start: 1,
            line_end: 5,
            is_async: false,
            visibility: None,
        };
        let summary = StructuralSummary {
            functions: vec![func],
            exports: vec!["foo".to_string()],
            ..Default::default()
        };

        let graph = DependencyGraph::from_structural_summary(&summary, file_path);
        let file_id = format!("file:{file_path}");
        let func_name = "foo";
        let func_id = format!("func:{file_path}::{func_name}");

        // 存在 Exports 边
        assert!(graph
            .edges
            .iter()
            .any(|e| e.edge_type == EdgeType::Exports && e.from == file_id && e.to == func_id));
    }
}
