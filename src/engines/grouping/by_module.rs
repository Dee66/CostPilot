// Group resources by Terraform module path

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A group of resources organized by their Terraform module path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleGroup {
    /// Module path (e.g., "root", "root.vpc", "root.vpc.subnets")
    pub module_path: String,
    /// Resource addresses in this module
    pub resources: Vec<String>,
    /// Total monthly cost for this module
    pub monthly_cost: f64,
    /// Number of resources
    pub resource_count: usize,
    /// Cost breakdown by resource type
    pub cost_by_type: HashMap<String, f64>,
}

impl ModuleGroup {
    pub fn new(module_path: String) -> Self {
        Self {
            module_path,
            resources: Vec::new(),
            monthly_cost: 0.0,
            resource_count: 0,
            cost_by_type: HashMap::new(),
        }
    }

    pub fn add_resource(&mut self, address: String, resource_type: String, cost: f64) {
        self.resources.push(address);
        self.monthly_cost += cost;
        self.resource_count += 1;
        *self.cost_by_type.entry(resource_type).or_insert(0.0) += cost;
    }

    pub fn average_cost_per_resource(&self) -> f64 {
        if self.resource_count == 0 {
            0.0
        } else {
            self.monthly_cost / self.resource_count as f64
        }
    }

    /// Get the module depth (e.g., "root" = 0, "root.vpc" = 1, "root.vpc.subnets" = 2)
    pub fn depth(&self) -> usize {
        if self.module_path == "root" {
            0
        } else {
            self.module_path.matches('.').count()
        }
    }

    /// Get parent module path (e.g., "root.vpc.subnets" -> "root.vpc")
    pub fn parent_path(&self) -> Option<String> {
        if self.module_path == "root" {
            None
        } else if let Some(pos) = self.module_path.rfind('.') {
            Some(self.module_path[..pos].to_string())
        } else {
            Some("root".to_string())
        }
    }

    /// Get child module paths from a list of all module paths
    pub fn child_paths(&self, all_paths: &[String]) -> Vec<String> {
        let prefix = format!("{}.", self.module_path);
        all_paths
            .iter()
            .filter(|path| path.starts_with(&prefix) && path.matches('.').count() == self.depth() + 1)
            .cloned()
            .collect()
    }
}

/// Group resources by their Terraform module paths
pub fn group_by_module(
    resources: &[(String, String, f64)], // (address, type, cost)
) -> Vec<ModuleGroup> {
    let mut groups: HashMap<String, ModuleGroup> = HashMap::new();

    for (address, resource_type, cost) in resources {
        let module_path = extract_module_path(address);
        let group = groups.entry(module_path.clone()).or_insert_with(|| ModuleGroup::new(module_path));
        group.add_resource(address.clone(), resource_type.clone(), *cost);
    }

    let mut result: Vec<ModuleGroup> = groups.into_values().collect();
    result.sort_by(|a, b| b.monthly_cost.partial_cmp(&a.monthly_cost).unwrap());
    result
}

/// Extract module path from resource address
/// Examples:
/// - "aws_instance.web" -> "root"
/// - "module.vpc.aws_vpc.main" -> "root.vpc"
/// - "module.vpc.module.subnets.aws_subnet.private" -> "root.vpc.subnets"
fn extract_module_path(address: &str) -> String {
    let parts: Vec<&str> = address.split('.').collect();
    let mut path = Vec::new();

    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "module" && i + 1 < parts.len() {
            path.push(parts[i + 1]);
            i += 2;
        } else {
            break;
        }
    }

    if path.is_empty() {
        "root".to_string()
    } else {
        format!("root.{}", path.join("."))
    }
}

/// Aggregate child module costs to parent modules (hierarchical rollup)
pub fn aggregate_module_hierarchy(groups: &[ModuleGroup]) -> Vec<ModuleGroup> {
    let mut aggregated: HashMap<String, ModuleGroup> = HashMap::new();

    // Initialize with existing groups
    for group in groups {
        aggregated.insert(group.module_path.clone(), group.clone());
    }

    // Build hierarchy and aggregate upwards
    let all_paths: Vec<String> = groups.iter().map(|g| g.module_path.clone()).collect();
    
    for group in groups {
        let mut current_path = group.module_path.clone();
        let mut accumulated_cost = group.monthly_cost;

        // Walk up the hierarchy
        while let Some(parent) = extract_parent_path(&current_path) {
            if parent == current_path {
                break;
            }
            
            let parent_group = aggregated.entry(parent.clone()).or_insert_with(|| {
                let mut g = ModuleGroup::new(parent.clone());
                g
            });
            
            // Only add cost if this isn't already counted (avoid double-counting)
            if parent != group.module_path {
                parent_group.monthly_cost += accumulated_cost;
                accumulated_cost = 0.0; // Don't double count on further parents
            }
            
            current_path = parent;
        }
    }

    let mut result: Vec<ModuleGroup> = aggregated.into_values().collect();
    result.sort_by(|a, b| b.monthly_cost.partial_cmp(&a.monthly_cost).unwrap());
    result
}

fn extract_parent_path(path: &str) -> Option<String> {
    if path == "root" {
        None
    } else if let Some(pos) = path.rfind('.') {
        Some(path[..pos].to_string())
    } else {
        Some("root".to_string())
    }
}

/// Generate a module hierarchy tree for display
pub fn generate_module_tree(groups: &[ModuleGroup]) -> String {
    let mut tree = String::new();
    let mut sorted = groups.to_vec();
    sorted.sort_by(|a, b| {
        let depth_cmp = a.depth().cmp(&b.depth());
        if depth_cmp == std::cmp::Ordering::Equal {
            a.module_path.cmp(&b.module_path)
        } else {
            depth_cmp
        }
    });

    for group in sorted {
        let indent = "  ".repeat(group.depth());
        let name = if group.depth() == 0 {
            group.module_path.clone()
        } else {
            group.module_path.split('.').last().unwrap().to_string()
        };
        
        tree.push_str(&format!(
            "{}├─ {} (${:.2}/mo, {} resources)\n",
            indent, name, group.monthly_cost, group.resource_count
        ));
    }

    tree
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_module_path() {
        assert_eq!(extract_module_path("aws_instance.web"), "root");
        assert_eq!(extract_module_path("module.vpc.aws_vpc.main"), "root.vpc");
        assert_eq!(
            extract_module_path("module.vpc.module.subnets.aws_subnet.private"),
            "root.vpc.subnets"
        );
    }

    #[test]
    fn test_group_by_module() {
        let resources = vec![
            ("aws_instance.web".to_string(), "aws_instance".to_string(), 100.0),
            ("module.vpc.aws_vpc.main".to_string(), "aws_vpc".to_string(), 50.0),
            ("module.vpc.aws_subnet.private".to_string(), "aws_subnet".to_string(), 20.0),
        ];

        let groups = group_by_module(&resources);
        assert_eq!(groups.len(), 2);
        
        // Root module has highest cost
        assert_eq!(groups[0].module_path, "root");
        assert_eq!(groups[0].monthly_cost, 100.0);
        
        // VPC module
        assert_eq!(groups[1].module_path, "root.vpc");
        assert_eq!(groups[1].monthly_cost, 70.0);
    }

    #[test]
    fn test_module_depth() {
        let mut group = ModuleGroup::new("root".to_string());
        assert_eq!(group.depth(), 0);
        
        group.module_path = "root.vpc".to_string();
        assert_eq!(group.depth(), 1);
        
        group.module_path = "root.vpc.subnets".to_string();
        assert_eq!(group.depth(), 2);
    }

    #[test]
    fn test_parent_path() {
        let group = ModuleGroup::new("root.vpc.subnets".to_string());
        assert_eq!(group.parent_path(), Some("root.vpc".to_string()));
        
        let group = ModuleGroup::new("root.vpc".to_string());
        assert_eq!(group.parent_path(), Some("root".to_string()));
        
        let group = ModuleGroup::new("root".to_string());
        assert_eq!(group.parent_path(), None);
    }
}
