// Policy DSL parser - Custom rule language for cost governance

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A policy rule written in CostPilot DSL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub severity: RuleSeverity,
    pub conditions: Vec<Condition>,
    pub action: RuleAction,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Rule severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Condition to evaluate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub condition_type: ConditionType,
    pub operator: Operator,
    pub value: ConditionValue,
    #[serde(default)]
    pub negate: bool,
}

/// Type of condition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionType {
    /// Resource type matches (e.g., aws_instance)
    ResourceType,
    /// Resource attribute (e.g., instance_type, memory_size)
    ResourceAttribute { attribute: String },
    /// Monthly cost estimate
    MonthlyCost,
    /// Cost increase percentage
    CostIncrease,
    /// Module path matches
    ModulePath,
    /// Tag exists or has value
    Tag { key: String },
    /// Resource count in plan
    ResourceCount { resource_type: String },
    /// Custom expression
    Expression { expr: String },
}

/// Comparison operator
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    Matches, // Regex match
    In,      // Value in list
    NotIn,
}

/// Condition value (can be number, string, boolean, or list)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    Number(f64),
    String(String),
    Boolean(bool),
    List(Vec<String>),
}

/// Action to take when rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuleAction {
    /// Block the deployment
    Block { message: String },
    /// Warn but allow
    Warn { message: String },
    /// Require approval
    RequireApproval {
        approvers: Vec<String>,
        message: String,
    },
    /// Set cost budget
    SetBudget { monthly_limit: f64 },
    /// Tag resource
    TagResource { tags: HashMap<String, String> },
}

/// DSL parser for policy rules
pub struct DslParser {
    rules: Vec<PolicyRule>,
}

impl DslParser {
    /// Create new parser
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }

    /// Parse rules from YAML string
    pub fn parse_yaml(yaml: &str) -> Result<Vec<PolicyRule>, ParseError> {
        let rules: Vec<PolicyRule> =
            serde_yaml::from_str(yaml).map_err(|e| ParseError::YamlError(e.to_string()))?;

        // Validate rules
        for rule in &rules {
            Self::validate_rule(rule)?;
        }

        Ok(rules)
    }

    /// Parse rules from JSON string
    pub fn parse_json(json: &str) -> Result<Vec<PolicyRule>, ParseError> {
        let rules: Vec<PolicyRule> =
            serde_json::from_str(json).map_err(|e| ParseError::JsonError(e.to_string()))?;

        // Validate rules
        for rule in &rules {
            Self::validate_rule(rule)?;
        }

        Ok(rules)
    }

    /// Validate a rule
    fn validate_rule(rule: &PolicyRule) -> Result<(), ParseError> {
        if rule.name.is_empty() {
            return Err(ParseError::InvalidRule(
                "Rule name cannot be empty".to_string(),
            ));
        }

        if rule.conditions.is_empty() {
            return Err(ParseError::InvalidRule(format!(
                "Rule '{}' must have at least one condition",
                rule.name
            )));
        }

        // Validate each condition
        for condition in &rule.conditions {
            Self::validate_condition(condition)?;
        }

        Ok(())
    }

    /// Validate a condition
    fn validate_condition(condition: &Condition) -> Result<(), ParseError> {
        // Check operator compatibility with value type
        match &condition.value {
            ConditionValue::Number(_) => {
                if !matches!(
                    condition.operator,
                    Operator::Equals
                        | Operator::NotEquals
                        | Operator::GreaterThan
                        | Operator::GreaterThanOrEqual
                        | Operator::LessThan
                        | Operator::LessThanOrEqual
                ) {
                    return Err(ParseError::InvalidCondition(format!(
                        "Operator {:?} not compatible with numeric value",
                        condition.operator
                    )));
                }
            }
            ConditionValue::String(_) => {
                // All operators valid for strings
            }
            ConditionValue::Boolean(_) => {
                if !matches!(condition.operator, Operator::Equals | Operator::NotEquals) {
                    return Err(ParseError::InvalidCondition(format!(
                        "Operator {:?} not compatible with boolean value",
                        condition.operator
                    )));
                }
            }
            ConditionValue::List(_) => {
                if !matches!(condition.operator, Operator::In | Operator::NotIn) {
                    return Err(ParseError::InvalidCondition(format!(
                        "Operator {:?} not compatible with list value",
                        condition.operator
                    )));
                }
            }
        }

        Ok(())
    }

    /// Add a rule to the parser
    pub fn add_rule(&mut self, rule: PolicyRule) -> Result<(), ParseError> {
        Self::validate_rule(&rule)?;
        self.rules.push(rule);
        Ok(())
    }

    /// Get all rules
    pub fn rules(&self) -> &[PolicyRule] {
        &self.rules
    }

    /// Get enabled rules only
    pub fn enabled_rules(&self) -> Vec<&PolicyRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    /// Get rules by severity
    pub fn rules_by_severity(&self, severity: RuleSeverity) -> Vec<&PolicyRule> {
        self.rules
            .iter()
            .filter(|r| r.enabled && r.severity == severity)
            .collect()
    }
}

impl Default for DslParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse error types
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("YAML parse error: {0}")]
    YamlError(String),

    #[error("JSON parse error: {0}")]
    JsonError(String),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    #[error("Invalid condition: {0}")]
    InvalidCondition(String),
}

/// Rule evaluator - evaluates rules against resource data
pub struct RuleEvaluator {
    parser: DslParser,
}

impl RuleEvaluator {
    /// Create new evaluator with parsed rules
    pub fn new(rules: Vec<PolicyRule>) -> Self {
        let mut parser = DslParser::new();
        for rule in rules {
            let _ = parser.add_rule(rule);
        }
        Self { parser }
    }

    /// Evaluate all rules against resource context
    pub fn evaluate(&self, context: &EvaluationContext) -> EvaluationResult {
        let mut result = EvaluationResult::new();

        for rule in self.parser.enabled_rules() {
            if self.evaluate_rule(rule, context) {
                result.add_match(RuleMatch {
                    rule_name: rule.name.clone(),
                    severity: rule.severity,
                    action: rule.action.clone(),
                    message: self.format_message(rule, context),
                });
            }
        }

        result
    }

    /// Evaluate single rule
    fn evaluate_rule(&self, rule: &PolicyRule, context: &EvaluationContext) -> bool {
        // All conditions must be true (AND logic)
        rule.conditions
            .iter()
            .all(|cond| self.evaluate_condition(cond, context))
    }

    /// Evaluate single condition
    fn evaluate_condition(&self, condition: &Condition, context: &EvaluationContext) -> bool {
        let result = match &condition.condition_type {
            ConditionType::ResourceType => self.evaluate_resource_type(condition, context),
            ConditionType::ResourceAttribute { attribute } => {
                self.evaluate_attribute(attribute, condition, context)
            }
            ConditionType::MonthlyCost => self.evaluate_monthly_cost(condition, context),
            ConditionType::CostIncrease => self.evaluate_cost_increase(condition, context),
            ConditionType::ModulePath => self.evaluate_module_path(condition, context),
            ConditionType::Tag { key } => self.evaluate_tag(key, condition, context),
            ConditionType::ResourceCount { resource_type } => {
                self.evaluate_resource_count(resource_type, condition, context)
            }
            ConditionType::Expression { expr } => {
                self.evaluate_expression(expr, condition, context)
            }
        };

        if condition.negate {
            !result
        } else {
            result
        }
    }

    fn evaluate_resource_type(&self, condition: &Condition, context: &EvaluationContext) -> bool {
        if let Some(resource_type) = &context.resource_type {
            self.compare_value(resource_type, &condition.operator, &condition.value)
        } else {
            false
        }
    }

    fn evaluate_attribute(
        &self,
        attribute: &str,
        condition: &Condition,
        context: &EvaluationContext,
    ) -> bool {
        if let Some(value) = context.attributes.get(attribute) {
            self.compare_value(value, &condition.operator, &condition.value)
        } else {
            false
        }
    }

    fn evaluate_monthly_cost(&self, condition: &Condition, context: &EvaluationContext) -> bool {
        if let Some(cost) = context.monthly_cost {
            if let ConditionValue::Number(limit) = condition.value {
                self.compare_number(cost, &condition.operator, limit)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn evaluate_cost_increase(&self, condition: &Condition, context: &EvaluationContext) -> bool {
        if let Some(increase) = context.cost_increase_percent {
            if let ConditionValue::Number(threshold) = condition.value {
                self.compare_number(increase, &condition.operator, threshold)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn evaluate_module_path(&self, condition: &Condition, context: &EvaluationContext) -> bool {
        if let Some(module_path) = &context.module_path {
            self.compare_value(module_path, &condition.operator, &condition.value)
        } else {
            false
        }
    }

    fn evaluate_tag(&self, key: &str, condition: &Condition, context: &EvaluationContext) -> bool {
        if let Some(tag_value) = context.tags.get(key) {
            self.compare_value(tag_value, &condition.operator, &condition.value)
        } else {
            false
        }
    }

    fn evaluate_resource_count(
        &self,
        resource_type: &str,
        condition: &Condition,
        context: &EvaluationContext,
    ) -> bool {
        if let Some(&count) = context.resource_counts.get(resource_type) {
            if let ConditionValue::Number(limit) = condition.value {
                self.compare_number(count as f64, &condition.operator, limit)
            } else {
                false
            }
        } else {
            false
        }
    }

    fn evaluate_expression(
        &self,
        _expr: &str,
        _condition: &Condition,
        _context: &EvaluationContext,
    ) -> bool {
        // Expression evaluation not yet implemented (future enhancement)
        false
    }

    fn compare_value(&self, actual: &str, operator: &Operator, expected: &ConditionValue) -> bool {
        match expected {
            ConditionValue::String(s) => self.compare_string(actual, operator, s),
            ConditionValue::List(list) => self.compare_list(actual, operator, list),
            _ => false,
        }
    }

    fn compare_string(&self, actual: &str, operator: &Operator, expected: &str) -> bool {
        match operator {
            Operator::Equals => actual == expected,
            Operator::NotEquals => actual != expected,
            Operator::Contains => actual.contains(expected),
            Operator::StartsWith => actual.starts_with(expected),
            Operator::EndsWith => actual.ends_with(expected),
            Operator::Matches => {
                if let Ok(re) = regex::Regex::new(expected) {
                    re.is_match(actual)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    fn compare_list(&self, actual: &str, operator: &Operator, list: &[String]) -> bool {
        match operator {
            Operator::In => list.iter().any(|item| item == actual),
            Operator::NotIn => !list.iter().any(|item| item == actual),
            _ => false,
        }
    }

    fn compare_number(&self, actual: f64, operator: &Operator, expected: f64) -> bool {
        match operator {
            Operator::Equals => (actual - expected).abs() < f64::EPSILON,
            Operator::NotEquals => (actual - expected).abs() >= f64::EPSILON,
            Operator::GreaterThan => actual > expected,
            Operator::GreaterThanOrEqual => actual >= expected,
            Operator::LessThan => actual < expected,
            Operator::LessThanOrEqual => actual <= expected,
            _ => false,
        }
    }

    fn format_message(&self, rule: &PolicyRule, _context: &EvaluationContext) -> String {
        match &rule.action {
            RuleAction::Block { message } => message.clone(),
            RuleAction::Warn { message } => message.clone(),
            RuleAction::RequireApproval { message, .. } => message.clone(),
            _ => format!("Rule '{}' triggered", rule.name),
        }
    }
}

/// Context for rule evaluation
#[derive(Debug, Clone, Default)]
pub struct EvaluationContext {
    pub resource_type: Option<String>,
    pub resource_id: Option<String>,
    pub attributes: HashMap<String, String>,
    pub monthly_cost: Option<f64>,
    pub cost_increase_percent: Option<f64>,
    pub module_path: Option<String>,
    pub tags: HashMap<String, String>,
    pub resource_counts: HashMap<String, usize>,
}

impl EvaluationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_resource_type(mut self, resource_type: String) -> Self {
        self.resource_type = Some(resource_type);
        self
    }

    pub fn with_monthly_cost(mut self, cost: f64) -> Self {
        self.monthly_cost = Some(cost);
        self
    }

    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// Result of rule evaluation
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    pub matches: Vec<RuleMatch>,
}

impl EvaluationResult {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
        }
    }

    pub fn add_match(&mut self, rule_match: RuleMatch) {
        self.matches.push(rule_match);
    }

    pub fn has_blocks(&self) -> bool {
        self.matches
            .iter()
            .any(|m| matches!(m.action, RuleAction::Block { .. }))
    }

    pub fn has_warnings(&self) -> bool {
        self.matches
            .iter()
            .any(|m| matches!(m.action, RuleAction::Warn { .. }))
    }

    pub fn requires_approval(&self) -> bool {
        self.matches
            .iter()
            .any(|m| matches!(m.action, RuleAction::RequireApproval { .. }))
    }
}

impl Default for EvaluationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// A matched rule
#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub rule_name: String,
    pub severity: RuleSeverity,
    pub action: RuleAction,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let yaml = r#"
- name: "Expensive EC2"
  description: "Block expensive EC2 instances"
  enabled: true
  severity: High
  conditions:
    - condition_type:
        type: resource_type
      operator: equals
      value: "aws_instance"
    - condition_type:
        type: monthly_cost
      operator: greater_than
      value: 1000.0
  action:
    type: block
    message: "EC2 instance costs more than $1000/month"
"#;

        let rules = DslParser::parse_yaml(yaml).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "Expensive EC2");
        assert_eq!(rules[0].conditions.len(), 2);
    }

    #[test]
    fn test_evaluate_rule() {
        let rule = PolicyRule {
            name: "Test Rule".to_string(),
            description: "Test".to_string(),
            enabled: true,
            severity: RuleSeverity::High,
            conditions: vec![Condition {
                condition_type: ConditionType::ResourceType,
                operator: Operator::Equals,
                value: ConditionValue::String("aws_instance".to_string()),
                negate: false,
            }],
            action: RuleAction::Warn {
                message: "Test warning".to_string(),
            },
            metadata: HashMap::new(),
        };

        let evaluator = RuleEvaluator::new(vec![rule]);
        let context = EvaluationContext::new().with_resource_type("aws_instance".to_string());

        let result = evaluator.evaluate(&context);
        assert_eq!(result.matches.len(), 1);
    }
}
