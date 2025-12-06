# Fish shell completion for costpilot
# Install: Copy to ~/.config/fish/completions/costpilot.fish

# Main command completions
complete -c costpilot -f

# Top-level commands
complete -c costpilot -n "__fish_use_subcommand" -a scan -d "Scan Terraform plan for cost issues"
complete -c costpilot -n "__fish_use_subcommand" -a diff -d "Compare cost between two plans"
complete -c costpilot -n "__fish_use_subcommand" -a autofix -d "Generate cost optimization recommendations"
complete -c costpilot -n "__fish_use_subcommand" -a init -d "Initialize CostPilot configuration"
complete -c costpilot -n "__fish_use_subcommand" -a map -d "Generate dependency map"
complete -c costpilot -n "__fish_use_subcommand" -a slo -d "Check cost SLOs"
complete -c costpilot -n "__fish_use_subcommand" -a policy -d "Manage policy lifecycle"
complete -c costpilot -n "__fish_use_subcommand" -a audit -d "Audit log and compliance"
complete -c costpilot -n "__fish_use_subcommand" -a heuristics -d "Manage cost heuristics"
complete -c costpilot -n "__fish_use_subcommand" -a explain -d "Explain cost predictions"
complete -c costpilot -n "__fish_use_subcommand" -a policy-dsl -d "Manage custom policy rules"
complete -c costpilot -n "__fish_use_subcommand" -a group -d "Group resources for cost allocation"
complete -c costpilot -n "__fish_use_subcommand" -a version -d "Show version information"
complete -c costpilot -n "__fish_use_subcommand" -a help -d "Show help information"

# Global options
complete -c costpilot -s v -l verbose -d "Enable verbose output"
complete -c costpilot -s f -l format -a "json text markdown" -d "Output format"

# scan command
complete -c costpilot -n "__fish_seen_subcommand_from scan" -s p -l plan -F -r -a "*.json" -d "Path to Terraform plan JSON"
complete -c costpilot -n "__fish_seen_subcommand_from scan" -s e -l explain -d "Enable detailed explanations"
complete -c costpilot -n "__fish_seen_subcommand_from scan" -l policy -F -r -a "*.yml *.yaml" -d "Path to policy file"
complete -c costpilot -n "__fish_seen_subcommand_from scan" -l exemptions -F -r -a "*.yml *.yaml" -d "Path to exemptions file"
complete -c costpilot -n "__fish_seen_subcommand_from scan" -s f -l format -a "markdown json text" -d "Output format"
complete -c costpilot -n "__fish_seen_subcommand_from scan" -l fail-on-critical -d "Fail on critical severity issues"
complete -c costpilot -n "__fish_seen_subcommand_from scan" -l autofix -d "Show autofix snippets"

# diff command
complete -c costpilot -n "__fish_seen_subcommand_from diff" -s b -l before -F -r -a "*.json" -d "Path to baseline plan"
complete -c costpilot -n "__fish_seen_subcommand_from diff" -s a -l after -F -r -a "*.json" -d "Path to proposed plan"
complete -c costpilot -n "__fish_seen_subcommand_from diff" -s f -l format -a "json text markdown" -d "Output format"

# autofix command
complete -c costpilot -n "__fish_seen_subcommand_from autofix" -s m -l mode -a "snippet patch" -d "Mode"
complete -c costpilot -n "__fish_seen_subcommand_from autofix" -s p -l plan -F -r -a "*.json" -d "Path to plan file"
complete -c costpilot -n "__fish_seen_subcommand_from autofix" -l drift-safe -d "Enable drift-safe mode"

# init command
complete -c costpilot -n "__fish_seen_subcommand_from init" -l no-ci -d "Skip creating CI template"

# map command
complete -c costpilot -n "__fish_seen_subcommand_from map" -s f -l format -a "mermaid graphviz json html" -d "Output format"
complete -c costpilot -n "__fish_seen_subcommand_from map" -s o -l output -F -r -d "Output file path"
complete -c costpilot -n "__fish_seen_subcommand_from map" -l show-costs -d "Show cost information in graph"
complete -c costpilot -n "__fish_seen_subcommand_from map" -l cycle-detection -d "Enable cycle detection"
complete -c costpilot -n "__fish_seen_subcommand_from map" -l depth -d "Maximum depth"
complete -c costpilot -n "__fish_seen_subcommand_from map" -l filter -d "Filter resources"

# slo command and subcommands
complete -c costpilot -n "__fish_seen_subcommand_from slo; and not __fish_seen_subcommand_from check burn" -a check -d "Check SLO compliance"
complete -c costpilot -n "__fish_seen_subcommand_from slo; and not __fish_seen_subcommand_from check burn" -a burn -d "Calculate burn rate"
complete -c costpilot -n "__fish_seen_subcommand_from slo" -l slo -F -r -a "*.yml *.yaml" -d "Path to SLO configuration"
complete -c costpilot -n "__fish_seen_subcommand_from slo" -s d -l snapshots -d "Path to snapshots directory"
complete -c costpilot -n "__fish_seen_subcommand_from slo" -l min-snapshots -d "Minimum snapshots required"
complete -c costpilot -n "__fish_seen_subcommand_from slo" -l min-r-squared -d "Minimum R² threshold"

# policy command and subcommands
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a submit -d "Submit policy for approval"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a approve -d "Approve a policy"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a reject -d "Reject a policy"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a activate -d "Activate an approved policy"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a deprecate -d "Deprecate an active policy"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a status -d "Show policy status"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a history -d "Show policy version history"
complete -c costpilot -n "__fish_seen_subcommand_from policy; and not __fish_seen_subcommand_from submit approve reject activate deprecate status history diff" -a diff -d "Compare two policy versions"

# policy-dsl command and subcommands
complete -c costpilot -n "__fish_seen_subcommand_from policy-dsl; and not __fish_seen_subcommand_from list validate test stats example" -a list -d "List all policies"
complete -c costpilot -n "__fish_seen_subcommand_from policy-dsl; and not __fish_seen_subcommand_from list validate test stats example" -a validate -d "Validate policy syntax"
complete -c costpilot -n "__fish_seen_subcommand_from policy-dsl; and not __fish_seen_subcommand_from list validate test stats example" -a test -d "Test policy against plan"
complete -c costpilot -n "__fish_seen_subcommand_from policy-dsl; and not __fish_seen_subcommand_from list validate test stats example" -a stats -d "Show policy statistics"
complete -c costpilot -n "__fish_seen_subcommand_from policy-dsl; and not __fish_seen_subcommand_from list validate test stats example" -a example -d "Show example policies"
complete -c costpilot -n "__fish_seen_subcommand_from policy-dsl" -l policy -F -r -a "*.yml *.yaml" -d "Path to policy file"

# group command and subcommands
complete -c costpilot -n "__fish_seen_subcommand_from group; and not __fish_seen_subcommand_from module service environment attribution all" -a module -d "Group by Terraform module"
complete -c costpilot -n "__fish_seen_subcommand_from group; and not __fish_seen_subcommand_from module service environment attribution all" -a service -d "Group by AWS service"
complete -c costpilot -n "__fish_seen_subcommand_from group; and not __fish_seen_subcommand_from module service environment attribution all" -a environment -d "Group by environment"
complete -c costpilot -n "__fish_seen_subcommand_from group; and not __fish_seen_subcommand_from module service environment attribution all" -a attribution -d "Generate cost attribution report"
complete -c costpilot -n "__fish_seen_subcommand_from group; and not __fish_seen_subcommand_from module service environment attribution all" -a all -d "Comprehensive report across all dimensions"
complete -c costpilot -n "__fish_seen_subcommand_from group" -l by-category -d "Group by category"
complete -c costpilot -n "__fish_seen_subcommand_from group" -l min-cost -d "Minimum cost threshold"
complete -c costpilot -n "__fish_seen_subcommand_from group" -l max-groups -d "Maximum groups"
complete -c costpilot -n "__fish_seen_subcommand_from group" -l detailed -d "Detailed breakdown"
complete -c costpilot -n "__fish_seen_subcommand_from group" -l detect-anomalies -d "Detect anomalies"
complete -c costpilot -n "__fish_seen_subcommand_from group" -l top-n -d "Top N cost centers"
complete -c costpilot -n "__fish_seen_subcommand_from group" -s o -l output -F -r -d "Output file"

# heuristics command and subcommands
complete -c costpilot -n "__fish_seen_subcommand_from heuristics; and not __fish_seen_subcommand_from list show update validate stats search" -a list -d "List all heuristics"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics; and not __fish_seen_subcommand_from list show update validate stats search" -a show -d "Show specific heuristic"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics; and not __fish_seen_subcommand_from list show update validate stats search" -a update -d "Update heuristics"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics; and not __fish_seen_subcommand_from list show update validate stats search" -a validate -d "Validate heuristics"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics; and not __fish_seen_subcommand_from list show update validate stats search" -a stats -d "Show heuristics statistics"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics; and not __fish_seen_subcommand_from list show update validate stats search" -a search -d "Search heuristics"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics" -l region -d "AWS region"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics" -l instance-type -d "Instance type"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics" -l resource-type -d "Resource type"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics" -l file -F -r -a "*.toml" -d "Heuristics file"
complete -c costpilot -n "__fish_seen_subcommand_from heuristics" -l query -d "Search query"

# explain command and subcommands
complete -c costpilot -n "__fish_seen_subcommand_from explain; and not __fish_seen_subcommand_from prediction detection policy regression" -a prediction -d "Explain cost prediction"
complete -c costpilot -n "__fish_seen_subcommand_from explain; and not __fish_seen_subcommand_from prediction detection policy regression" -a detection -d "Explain resource detection"
complete -c costpilot -n "__fish_seen_subcommand_from explain; and not __fish_seen_subcommand_from prediction detection policy regression" -a policy -d "Explain policy violation"
complete -c costpilot -n "__fish_seen_subcommand_from explain; and not __fish_seen_subcommand_from prediction detection policy regression" -a regression -d "Explain cost regression"
complete -c costpilot -n "__fish_seen_subcommand_from explain" -l resource -d "Resource ID"
complete -c costpilot -n "__fish_seen_subcommand_from explain" -l plan -F -r -a "*.json" -d "Plan file"
complete -c costpilot -n "__fish_seen_subcommand_from explain" -l policy -F -r -a "*.yml *.yaml" -d "Policy file"

# version command
complete -c costpilot -n "__fish_seen_subcommand_from version" -s d -l detailed -d "Show detailed version info"
