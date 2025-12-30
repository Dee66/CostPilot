use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use ignore::WalkBuilder;
use rayon::prelude::*;
use anyhow::{Result, Context};

#[derive(Parser)]
#[command(name = "repo-scanner")]
#[command(about = "Accurate repository analysis tool")]
struct Args {
    /// Path to repository to scan
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output format
    #[arg(long, default_value = "json")]
    format: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanResult {
    summary: Summary,
    languages: HashMap<String, LanguageStats>,
    security_findings: SecurityFindings,
    compliance_status: ComplianceStatus,
}

#[derive(Debug, Serialize, Deserialize)]
struct Summary {
    total_files: usize,
    total_lines: usize,
    languages_detected: usize,
    scan_duration_ms: u128,
}

#[derive(Debug, Serialize, Deserialize)]
struct LanguageStats {
    files: usize,
    lines: usize,
    percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct SecurityFindings {
    patterns_found: HashMap<String, usize>,
    risk_score: f64,
    evidence_based: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct ComplianceStatus {
    standards_checked: Vec<String>,
    compliance_level: String,
    notes: Vec<String>,
}

#[derive(Debug)]
struct FileAnalysis {
    path: PathBuf,
    language: Option<String>,
    lines: usize,
    content: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let start_time = std::time::Instant::now();

    println!("Scanning repository: {}", args.path.display());

    // Build walker with proper ignore handling
    let walker = WalkBuilder::new(&args.path)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map_or(false, |ft| ft.is_file()))
        .collect::<Vec<_>>();

    if args.verbose {
        println!("Found {} files to analyze", walker.len());
    }

    // Analyze files in parallel
    let analyses: Vec<FileAnalysis> = walker
        .par_iter()
        .filter_map(|entry| analyze_file(entry.path()).ok())
        .collect();

    // Aggregate results
    let mut languages: HashMap<String, LanguageStats> = HashMap::new();
    let mut total_lines = 0;
    let mut security_patterns: HashMap<String, usize> = HashMap::new();

    for analysis in &analyses {
        total_lines += analysis.lines;

        if let Some(lang) = &analysis.language {
            let stats = languages.entry(lang.clone()).or_insert(LanguageStats {
                files: 0,
                lines: 0,
                percentage: 0.0,
            });
            stats.files += 1;
            stats.lines += analysis.lines;
        }

        // Evidence-based security pattern detection
        detect_security_patterns(&analysis.content, &mut security_patterns);
    }

    // Calculate percentages
    let total_files = analyses.len();
    for stats in languages.values_mut() {
        stats.percentage = (stats.files as f64 / total_files as f64) * 100.0;
    }

    // Calculate risk score based on evidence
    let risk_score = calculate_risk_score(&security_patterns, total_files);

    let summary = Summary {
        total_files,
        total_lines,
        languages_detected: languages.len(),
        scan_duration_ms: start_time.elapsed().as_millis(),
    };

    let security_findings = SecurityFindings {
        patterns_found: security_patterns,
        risk_score,
        evidence_based: true,
    };

    let compliance_status = assess_compliance(&args.path)?;

    let result = ScanResult {
        summary,
        languages,
        security_findings,
        compliance_status,
    };

    // Output results
    match args.format.as_str() {
        "json" => println!("{}", serde_json::to_string_pretty(&result)?),
        "text" => print_text_output(&result),
        _ => println!("{}", serde_json::to_string_pretty(&result)?),
    }

    Ok(())
}

fn analyze_file(path: &Path) -> Result<FileAnalysis> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;

    let lines = content.lines().count();
    let language = detect_language(path, &content);

    Ok(FileAnalysis {
        path: path.to_path_buf(),
        language,
        lines,
        content,
    })
}

fn detect_language(path: &Path, content: &str) -> Option<String> {
    // File extension based detection
    if let Some(ext) = path.extension() {
        match ext.to_str()? {
            "rs" => return Some("Rust".to_string()),
            "py" => return Some("Python".to_string()),
            "js" => return Some("JavaScript".to_string()),
            "ts" => return Some("TypeScript".to_string()),
            "java" => return Some("Java".to_string()),
            "go" => return Some("Go".to_string()),
            "cpp" | "cc" | "cxx" => return Some("C++".to_string()),
            "c" => return Some("C".to_string()),
            "sh" => return Some("Shell".to_string()),
            "md" => return Some("Markdown".to_string()),
            "json" => return Some("JSON".to_string()),
            "yml" | "yaml" => return Some("YAML".to_string()),
            "toml" => return Some("TOML".to_string()),
            _ => {}
        }
    }

    // Content-based detection for files without extensions
    if content.contains("#!/usr/bin/env python") || content.contains("#!/usr/bin/python") {
        return Some("Python".to_string());
    }
    if content.contains("#!/bin/bash") || content.contains("#!/bin/sh") {
        return Some("Shell".to_string());
    }
    if content.contains("fn main()") && content.contains("use std::") {
        return Some("Rust".to_string());
    }

    None
}

fn detect_security_patterns(content: &str, patterns: &mut HashMap<String, usize>) {
    let security_patterns = [
        (r"password|Password|PASSWORD", "password_usage"),
        (r"secret|Secret|SECRET", "secret_usage"),
        (r"token|Token|TOKEN", "token_usage"),
        (r"auth|Auth|AUTH", "auth_usage"),
        (r"encrypt|Encrypt|ENCRYPT", "encryption_usage"),
        (r"decrypt|Decrypt|DECRYPT", "decryption_usage"),
        (r"hash|Hash|HASH", "hash_usage"),
        (r"sql|SQL", "sql_usage"),
        (r"xss|XSS", "xss_usage"),
        (r"injection|Injection|INJECTION", "injection_usage"),
    ];

    for (pattern, name) in &security_patterns {
        let re = Regex::new(pattern).unwrap();
        let count = re.find_iter(content).count();
        if count > 0 {
            *patterns.entry(name.to_string()).or_insert(0) += count;
        }
    }
}

fn calculate_risk_score(patterns: &HashMap<String, usize>, total_files: usize) -> f64 {
    let mut score = 0.0;

    // Weight different patterns
    let weights = [
        ("password_usage", 2.0),
        ("secret_usage", 3.0),
        ("token_usage", 1.5),
        ("auth_usage", 1.0),
        ("encryption_usage", -1.0), // Negative because encryption is good
        ("decryption_usage", 0.5),
        ("hash_usage", -0.5), // Hashing can be good or bad
        ("sql_usage", 1.0),
        ("xss_usage", 2.0),
        ("injection_usage", 2.5),
    ];

    for (pattern, weight) in &weights {
        if let Some(count) = patterns.get(*pattern) {
            score += *count as f64 * weight;
        }
    }

    // Normalize by file count and cap at 10.0
    let normalized_score = score / total_files as f64 * 10.0;
    normalized_score.max(0.0).min(10.0)
}

fn assess_compliance(repo_path: &Path) -> Result<ComplianceStatus> {
    let mut standards_checked = Vec::new();
    let mut notes = Vec::new();
    let mut compliance_level = "Unknown".to_string();

    // Check for common compliance files
    let compliance_files = [
        "LICENSE",
        "LICENSE.md",
        "LICENSE.txt",
        "SECURITY.md",
        "CODE_OF_CONDUCT.md",
        "CONTRIBUTING.md",
    ];

    for file in &compliance_files {
        if repo_path.join(file).exists() {
            standards_checked.push(file.to_string());
        }
    }

    // Basic assessment
    if standards_checked.contains(&"LICENSE".to_string()) {
        compliance_level = "Basic".to_string();
        notes.push("License file found".to_string());
    } else {
        notes.push("No license file found".to_string());
    }

    if standards_checked.contains(&"SECURITY.md".to_string()) {
        compliance_level = "Standard".to_string();
        notes.push("Security policy found".to_string());
    }

    Ok(ComplianceStatus {
        standards_checked,
        compliance_level,
        notes,
    })
}

fn print_text_output(result: &ScanResult) {
    println!("Repository Scan Results");
    println!("======================");
    println!("Total Files: {}", result.summary.total_files);
    println!("Total Lines: {}", result.summary.total_lines);
    println!("Languages Detected: {}", result.summary.languages_detected);
    println!("Scan Duration: {}ms", result.summary.scan_duration_ms);
    println!();

    println!("Languages:");
    for (lang, stats) in &result.languages {
        println!("  {}: {} files ({:.1}%), {} lines",
                lang, stats.files, stats.percentage, stats.lines);
    }
    println!();

    println!("Security Findings:");
    println!("  Risk Score: {:.2}/10.0", result.security_findings.risk_score);
    println!("  Evidence-based: {}", result.security_findings.evidence_based);
    println!("  Patterns Found:");
    for (pattern, count) in &result.security_findings.patterns_found {
        println!("    {}: {}", pattern, count);
    }
    println!();

    println!("Compliance Status:");
    println!("  Level: {}", result.compliance_status.compliance_level);
    println!("  Standards Checked: {}", result.compliance_status.standards_checked.join(", "));
    for note in &result.compliance_status.notes {
        println!("  Note: {}", note);
    }
}
