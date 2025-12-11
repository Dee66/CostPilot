use crate::engines::policy::policy_metadata::*;
use std::collections::HashMap;

/// Repository for managing policies with metadata
pub struct PolicyRepository<T> {
    /// All policies indexed by ID
    policies: HashMap<String, PolicyWithMetadata<T>>,
}

impl<T> PolicyRepository<T>
where
    T: Clone,
{
    /// Create a new empty policy repository
    pub fn new() -> Self {
        Self {
            policies: HashMap::new(),
        }
    }

    /// Add a policy to the repository
    pub fn add(&mut self, policy: PolicyWithMetadata<T>) -> Result<(), String> {
        let id = policy.metadata.id.clone();

        if self.policies.contains_key(&id) {
            return Err(format!("Policy with ID '{}' already exists", id));
        }

        self.policies.insert(id, policy);
        Ok(())
    }

    /// Get a policy by ID
    pub fn get(&self, id: &str) -> Option<&PolicyWithMetadata<T>> {
        self.policies.get(id)
    }

    /// Get a mutable reference to a policy
    pub fn get_mut(&mut self, id: &str) -> Option<&mut PolicyWithMetadata<T>> {
        self.policies.get_mut(id)
    }

    /// Remove a policy from the repository
    pub fn remove(&mut self, id: &str) -> Option<PolicyWithMetadata<T>> {
        self.policies.remove(id)
    }

    /// Update an existing policy
    pub fn update(&mut self, id: &str, policy: PolicyWithMetadata<T>) -> Result<(), String> {
        if !self.policies.contains_key(id) {
            return Err(format!("Policy with ID '{}' not found", id));
        }

        self.policies.insert(id.to_string(), policy);
        Ok(())
    }

    /// Get all policies
    pub fn all(&self) -> Vec<&PolicyWithMetadata<T>> {
        self.policies.values().collect()
    }

    /// Get all policy IDs
    pub fn ids(&self) -> Vec<String> {
        self.policies.keys().cloned().collect()
    }

    /// Count total policies
    pub fn count(&self) -> usize {
        self.policies.len()
    }

    /// Check if repository contains a policy
    pub fn contains(&self, id: &str) -> bool {
        self.policies.contains_key(id)
    }

    /// Clear all policies
    pub fn clear(&mut self) {
        self.policies.clear();
    }

    /// Delegate ownership of a policy to a new owner
    pub fn delegate_ownership(
        &mut self,
        policy_id: &str,
        new_owner: String,
        new_team: Option<String>,
    ) -> Result<(), String> {
        let policy = self
            .policies
            .get_mut(policy_id)
            .ok_or_else(|| format!("Policy not found: {}", policy_id))?;

        policy.metadata.ownership.owner = new_owner;
        if let Some(team) = new_team {
            policy.metadata.ownership.team = Some(team);
        }
        policy.metadata.lifecycle.updated_at = chrono::Utc::now();

        Ok(())
    }

    /// Transfer ownership to a different team
    pub fn transfer_to_team(
        &mut self,
        policy_id: &str,
        team: String,
        new_owner: String,
    ) -> Result<(), String> {
        self.delegate_ownership(policy_id, new_owner, Some(team))
    }
}

impl<T> Default for PolicyRepository<T>
where
    T: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Filtering and querying capabilities
impl<T> PolicyRepository<T>
where
    T: Clone,
{
    /// Get all active policies that should be enforced
    pub fn get_enforceable(&self) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| p.should_enforce())
            .collect()
    }

    /// Get policies by status
    pub fn get_by_status(&self, status: &PolicyStatus) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| &p.metadata.status == status)
            .collect()
    }

    /// Get policies by category
    pub fn get_by_category(&self, category: &PolicyCategory) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| &p.metadata.category == category)
            .collect()
    }

    /// Get policies by severity
    pub fn get_by_severity(&self, severity: &Severity) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| &p.metadata.severity == severity)
            .collect()
    }

    /// Get policies by minimum severity
    pub fn get_by_min_severity(&self, min_severity: &Severity) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| p.metadata.severity >= *min_severity)
            .collect()
    }

    /// Get policies by tag
    pub fn get_by_tag(&self, tag: &str) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| p.metadata.has_tag(tag))
            .collect()
    }

    /// Get policies by owner
    pub fn get_by_owner(&self, owner: &str) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| p.metadata.ownership.owner == owner)
            .collect()
    }

    /// Get policies by team
    pub fn get_by_team(&self, team: &str) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| {
                p.metadata
                    .ownership
                    .team
                    .as_ref()
                    .is_some_and(|t| t == team)
            })
            .collect()
    }

    /// Get blocking policies (error or critical severity)
    pub fn get_blocking(&self) -> Vec<&PolicyWithMetadata<T>> {
        self.policies.values().filter(|p| p.is_blocking()).collect()
    }

    /// Get deprecated policies
    pub fn get_deprecated(&self) -> Vec<&PolicyWithMetadata<T>> {
        self.get_by_status(&PolicyStatus::Deprecated)
    }

    /// Search policies by name or description
    pub fn search(&self, query: &str) -> Vec<&PolicyWithMetadata<T>> {
        let query_lower = query.to_lowercase();
        self.policies
            .values()
            .filter(|p| {
                p.metadata.name.to_lowercase().contains(&query_lower)
                    || p.metadata.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }
}

/// Statistics and reporting
impl<T> PolicyRepository<T>
where
    T: Clone,
{
    /// Get repository statistics
    pub fn statistics(&self) -> RepositoryStatistics {
        let mut stats = RepositoryStatistics {
            total_policies: self.policies.len(),
            by_status: HashMap::new(),
            by_category: HashMap::new(),
            by_severity: HashMap::new(),
            total_evaluations: 0,
            total_violations: 0,
            total_exemptions: 0,
        };

        for policy in self.policies.values() {
            // Count by status
            *stats
                .by_status
                .entry(policy.metadata.status.clone())
                .or_insert(0) += 1;

            // Count by category
            *stats
                .by_category
                .entry(policy.metadata.category.clone())
                .or_insert(0) += 1;

            // Count by severity
            *stats
                .by_severity
                .entry(policy.metadata.severity.clone())
                .or_insert(0) += 1;

            // Aggregate metrics
            stats.total_evaluations += policy.metadata.metrics.evaluation_count;
            stats.total_violations += policy.metadata.metrics.violation_count;
            stats.total_exemptions += policy.metadata.metrics.exemption_count;
        }

        stats
    }

    /// Get policies with high violation rates
    pub fn get_high_violation_policies(&self, threshold: f64) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| {
                p.metadata
                    .metrics
                    .violation_rate
                    .is_some_and(|rate| rate > threshold)
            })
            .collect()
    }

    /// Get policies never evaluated
    pub fn get_never_evaluated(&self) -> Vec<&PolicyWithMetadata<T>> {
        self.policies
            .values()
            .filter(|p| p.metadata.metrics.evaluation_count == 0)
            .collect()
    }
}

/// Bulk operations
impl<T> PolicyRepository<T>
where
    T: Clone,
{
    /// Activate multiple policies by ID
    pub fn activate_policies(&mut self, ids: &[String]) -> Result<usize, String> {
        let mut activated = 0;

        for id in ids {
            if let Some(policy) = self.policies.get_mut(id) {
                policy.metadata.activate();
                activated += 1;
            } else {
                return Err(format!("Policy '{}' not found", id));
            }
        }

        Ok(activated)
    }

    /// Disable multiple policies by ID
    pub fn disable_policies(&mut self, ids: &[String]) -> Result<usize, String> {
        let mut disabled = 0;

        for id in ids {
            if let Some(policy) = self.policies.get_mut(id) {
                policy.metadata.disable();
                disabled += 1;
            } else {
                return Err(format!("Policy '{}' not found", id));
            }
        }

        Ok(disabled)
    }

    /// Archive old deprecated policies
    pub fn archive_deprecated(&mut self, older_than_days: i64) -> usize {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(older_than_days);
        let mut archived = 0;

        for policy in self.policies.values_mut() {
            if policy.metadata.status == PolicyStatus::Deprecated {
                if let Some(deprecation) = &policy.metadata.lifecycle.deprecation {
                    if deprecation.deprecated_at < cutoff {
                        policy.metadata.status = PolicyStatus::Archived;
                        archived += 1;
                    }
                }
            }
        }

        archived
    }
}

/// Repository statistics
#[derive(Debug, Clone)]
pub struct RepositoryStatistics {
    pub total_policies: usize,
    pub by_status: HashMap<PolicyStatus, usize>,
    pub by_category: HashMap<PolicyCategory, usize>,
    pub by_severity: HashMap<Severity, usize>,
    pub total_evaluations: u64,
    pub total_violations: u64,
    pub total_exemptions: u64,
}

impl RepositoryStatistics {
    /// Get overall violation rate across all policies
    pub fn overall_violation_rate(&self) -> Option<f64> {
        if self.total_evaluations > 0 {
            Some(self.total_violations as f64 / self.total_evaluations as f64)
        } else {
            None
        }
    }

    /// Format statistics as human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Total Policies: {}\n", self.total_policies));
        output.push_str("\nBy Status:\n");
        for (status, count) in &self.by_status {
            output.push_str(&format!("  {:?}: {}\n", status, count));
        }

        output.push_str("\nBy Category:\n");
        for (category, count) in &self.by_category {
            output.push_str(&format!("  {:?}: {}\n", category, count));
        }

        output.push_str("\nBy Severity:\n");
        for (severity, count) in &self.by_severity {
            output.push_str(&format!("  {:?}: {}\n", severity, count));
        }

        output.push_str(&format!(
            "\nTotal Evaluations: {}\n",
            self.total_evaluations
        ));
        output.push_str(&format!("Total Violations: {}\n", self.total_violations));
        output.push_str(&format!("Total Exemptions: {}\n", self.total_exemptions));

        if let Some(rate) = self.overall_violation_rate() {
            output.push_str(&format!("Overall Violation Rate: {:.2}%\n", rate * 100.0));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_policy(
        id: &str,
        status: PolicyStatus,
        severity: Severity,
    ) -> PolicyWithMetadata<()> {
        let mut metadata = PolicyMetadata::new(
            id.to_string(),
            format!("Policy {}", id),
            "Test policy".to_string(),
            PolicyCategory::Budget,
            severity,
            "alice".to_string(),
            "bob".to_string(),
        );
        metadata.status = status;
        PolicyWithMetadata::new(metadata, ())
    }

    #[test]
    fn test_repository_new() {
        let repo: PolicyRepository<()> = PolicyRepository::new();
        assert_eq!(repo.count(), 0);
    }

    #[test]
    fn test_repository_add_get() {
        let mut repo = PolicyRepository::new();
        let policy = create_test_policy("test-1", PolicyStatus::Active, Severity::Error);

        repo.add(policy).unwrap();
        assert_eq!(repo.count(), 1);

        let retrieved = repo.get("test-1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.id, "test-1");
    }

    #[test]
    fn test_repository_add_duplicate() {
        let mut repo = PolicyRepository::new();
        let policy1 = create_test_policy("test-1", PolicyStatus::Active, Severity::Error);
        let policy2 = create_test_policy("test-1", PolicyStatus::Active, Severity::Error);

        repo.add(policy1).unwrap();
        let result = repo.add(policy2);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_repository_remove() {
        let mut repo = PolicyRepository::new();
        let policy = create_test_policy("test-1", PolicyStatus::Active, Severity::Error);

        repo.add(policy).unwrap();
        assert_eq!(repo.count(), 1);

        let removed = repo.remove("test-1");
        assert!(removed.is_some());
        assert_eq!(repo.count(), 0);
    }

    #[test]
    fn test_repository_update() {
        let mut repo = PolicyRepository::new();
        let mut policy = create_test_policy("test-1", PolicyStatus::Draft, Severity::Warning);

        repo.add(policy.clone()).unwrap();

        policy.metadata.activate();
        repo.update("test-1", policy).unwrap();

        let updated = repo.get("test-1").unwrap();
        assert_eq!(updated.metadata.status, PolicyStatus::Active);
    }

    #[test]
    fn test_repository_get_enforceable() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "draft",
            PolicyStatus::Draft,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "active",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "disabled",
            PolicyStatus::Disabled,
            Severity::Error,
        ))
        .unwrap();

        let enforceable = repo.get_enforceable();
        assert_eq!(enforceable.len(), 1);
        assert_eq!(enforceable[0].metadata.id, "active");
    }

    #[test]
    fn test_repository_get_by_status() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Active,
            Severity::Warning,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p3",
            PolicyStatus::Draft,
            Severity::Error,
        ))
        .unwrap();

        let active = repo.get_by_status(&PolicyStatus::Active);
        assert_eq!(active.len(), 2);

        let draft = repo.get_by_status(&PolicyStatus::Draft);
        assert_eq!(draft.len(), 1);
    }

    #[test]
    fn test_repository_get_by_severity() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Active,
            Severity::Warning,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p3",
            PolicyStatus::Active,
            Severity::Critical,
        ))
        .unwrap();

        let errors = repo.get_by_severity(&Severity::Error);
        assert_eq!(errors.len(), 1);

        let warnings = repo.get_by_severity(&Severity::Warning);
        assert_eq!(warnings.len(), 1);
    }

    #[test]
    fn test_repository_get_by_min_severity() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Active,
            Severity::Info,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Active,
            Severity::Warning,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p3",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p4",
            PolicyStatus::Active,
            Severity::Critical,
        ))
        .unwrap();

        let high_severity = repo.get_by_min_severity(&Severity::Error);
        assert_eq!(high_severity.len(), 2); // Error and Critical
    }

    #[test]
    fn test_repository_get_blocking() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Active,
            Severity::Info,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Active,
            Severity::Warning,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p3",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p4",
            PolicyStatus::Active,
            Severity::Critical,
        ))
        .unwrap();

        let blocking = repo.get_blocking();
        assert_eq!(blocking.len(), 2); // Error and Critical
    }

    #[test]
    fn test_repository_get_by_tag() {
        let mut repo = PolicyRepository::new();

        let mut policy1 = create_test_policy("p1", PolicyStatus::Active, Severity::Error);
        policy1.metadata.add_tag("production".to_string());

        let mut policy2 = create_test_policy("p2", PolicyStatus::Active, Severity::Error);
        policy2.metadata.add_tag("development".to_string());

        let mut policy3 = create_test_policy("p3", PolicyStatus::Active, Severity::Error);
        policy3.metadata.add_tag("production".to_string());

        repo.add(policy1).unwrap();
        repo.add(policy2).unwrap();
        repo.add(policy3).unwrap();

        let production = repo.get_by_tag("production");
        assert_eq!(production.len(), 2);
    }

    #[test]
    fn test_repository_search() {
        let mut repo = PolicyRepository::new();

        let mut metadata1 = PolicyMetadata::new(
            "p1".to_string(),
            "Budget Policy".to_string(),
            "Controls spending".to_string(),
            PolicyCategory::Budget,
            Severity::Error,
            "alice".to_string(),
            "alice".to_string(),
        );
        metadata1.status = PolicyStatus::Active;

        let mut metadata2 = PolicyMetadata::new(
            "p2".to_string(),
            "Security Policy".to_string(),
            "Security controls".to_string(),
            PolicyCategory::Security,
            Severity::Critical,
            "bob".to_string(),
            "bob".to_string(),
        );
        metadata2.status = PolicyStatus::Active;

        repo.add(PolicyWithMetadata::new(metadata1, ())).unwrap();
        repo.add(PolicyWithMetadata::new(metadata2, ())).unwrap();

        let results = repo.search("budget");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.id, "p1");

        let results = repo.search("security");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].metadata.id, "p2");
    }

    #[test]
    fn test_repository_statistics() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Active,
            Severity::Warning,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p3",
            PolicyStatus::Draft,
            Severity::Critical,
        ))
        .unwrap();

        let stats = repo.statistics();
        assert_eq!(stats.total_policies, 3);
        assert_eq!(stats.by_status[&PolicyStatus::Active], 2);
        assert_eq!(stats.by_status[&PolicyStatus::Draft], 1);
        assert_eq!(stats.by_severity[&Severity::Error], 1);
        assert_eq!(stats.by_severity[&Severity::Warning], 1);
        assert_eq!(stats.by_severity[&Severity::Critical], 1);
    }

    #[test]
    fn test_repository_activate_policies() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Draft,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Draft,
            Severity::Error,
        ))
        .unwrap();

        let activated = repo
            .activate_policies(&["p1".to_string(), "p2".to_string()])
            .unwrap();
        assert_eq!(activated, 2);

        assert_eq!(
            repo.get("p1").unwrap().metadata.status,
            PolicyStatus::Active
        );
        assert_eq!(
            repo.get("p2").unwrap().metadata.status,
            PolicyStatus::Active
        );
    }

    #[test]
    fn test_repository_disable_policies() {
        let mut repo = PolicyRepository::new();

        repo.add(create_test_policy(
            "p1",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();
        repo.add(create_test_policy(
            "p2",
            PolicyStatus::Active,
            Severity::Error,
        ))
        .unwrap();

        let disabled = repo
            .disable_policies(&["p1".to_string(), "p2".to_string()])
            .unwrap();
        assert_eq!(disabled, 2);

        assert_eq!(
            repo.get("p1").unwrap().metadata.status,
            PolicyStatus::Disabled
        );
        assert_eq!(
            repo.get("p2").unwrap().metadata.status,
            PolicyStatus::Disabled
        );
    }
}
