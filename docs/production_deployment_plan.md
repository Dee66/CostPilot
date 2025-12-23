# CostPilot Production Deployment Implementation Plan

**Date:** 22 December 2025
**Focus Areas:** Cryptographic Key Management & Performance Scaling Enhancements

## Executive Summary

This implementation plan addresses the critical production readiness gaps identified in the Technical Intelligence Report. The focus is on two high-priority areas:

1. **Cryptographic Key Management**: Transition from compile-time key generation to production-grade external key management
2. **Performance Scaling**: Implement streaming processing and configurable resource limits for large Terraform plans

**Timeline:** 2-3 weeks for core implementation, 1-2 months for full rollout
**Risk Level:** MEDIUM (with proper testing and gradual rollout)

## 1. Cryptographic Key Management Implementation

### Current State Analysis

**Problems:**
- Compile-time Ed25519 keypair generation in `build.rs`
- Public keys can be overridden via environment variables, but private keys are still generated at build time
- No integration with HSMs, KMS, or secure vaults
- License issuer requires manual key file management
- Keys may be exposed in build artifacts

**Security Impact:** HIGH - Keys generated during build are unsuitable for production environments

### Implementation Plan

#### Phase 1: External Key Provider Interface (Week 1)

**Objective:** Create pluggable key management interface supporting multiple backends

**Changes Required:**

1. **New Key Provider Trait** (`src/pro_engine/key_provider.rs`)
```rust
pub trait KeyProvider: Send + Sync {
    async fn get_license_public_key(&self, issuer: &str) -> Result<[u8; 32], KeyError>;
    async fn get_wasm_public_key(&self) -> Result<[u8; 32], KeyError>;
    async fn sign_license(&self, message: &[u8], issuer: &str) -> Result<Vec<u8>, KeyError>;
    async fn sign_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>, KeyError>;
}
```

2. **Built-in Providers:**
   - `FileKeyProvider`: Loads keys from encrypted files (current behavior)
   - `EnvironmentKeyProvider`: Keys via environment variables
   - `HashiCorpVaultProvider`: Integration with Vault
   - `AwsKmsProvider`: AWS KMS integration
   - `AzureKeyVaultProvider`: Azure Key Vault integration

3. **Configuration System** (`src/config/key_config.rs`)
```rust
#[derive(Deserialize)]
pub struct KeyConfig {
    pub provider: KeyProviderType,
    pub vault_url: Option<String>,
    pub aws_region: Option<String>,
    pub key_ids: HashMap<String, String>, // issuer -> key_id mapping
}
```

#### Phase 2: Build System Refactoring (Week 1-2)

**Objective:** Remove compile-time key generation, implement runtime key loading

**Changes Required:**

1. **Modify `build.rs`**
   - Remove `generate_crypto_keys()` function
   - Generate placeholder constants only
   - Add feature flags for different key providers

2. **Runtime Key Loading** (`src/pro_engine/crypto.rs`)
   - Add async key provider initialization
   - Implement lazy key loading with caching
   - Add key rotation detection and cache invalidation

3. **License Issuer Updates** (`src/bin/license_issuer.rs`)
   - Add support for key provider backends
   - Implement key rotation commands
   - Add key status and health checks

#### Phase 3: Key Rotation Infrastructure (Week 2)

**Objective:** Enable seamless key rotation without service disruption

**Changes Required:**

1. **Key Versioning System**
   - Add version fields to license format
   - Implement key version negotiation
   - Support multiple active key versions during rotation

2. **Rotation Procedures**
   - Automated rotation scripts
   - Graceful key deactivation
   - Audit logging for key operations

3. **Monitoring & Alerting**
   - Key expiration warnings
   - Rotation success/failure alerts
   - Key usage metrics

### Testing Strategy

1. **Unit Tests:** Key provider interface compliance
2. **Integration Tests:** End-to-end key loading and signing
3. **Security Tests:** Key isolation, access control validation
4. **Performance Tests:** Key loading latency, caching effectiveness

## 2. Performance Scaling Enhancements

### Current State Analysis

**Problems:**
- Fixed WASM limits: 256MB memory, 2000ms timeout
- No streaming processing for large Terraform plans
- Synchronous processing blocks on large inputs
- Memory usage scales linearly with plan size

**Performance Impact:** MEDIUM - Large plans may timeout or exceed memory limits

### Implementation Plan

#### Phase 1: Configurable Resource Limits (Week 1)

**Objective:** Make WASM limits configurable per deployment/environment

**Changes Required:**

1. **Dynamic Limit Configuration** (`src/wasm/runtime.rs`)
```rust
#[derive(Deserialize)]
pub struct WasmLimitsConfig {
    pub max_memory_bytes: usize,
    pub max_execution_ms: u64,
    pub max_file_size_bytes: usize,
    pub max_stack_depth: usize,
    pub enable_streaming: bool,
    pub chunk_size_bytes: usize,
}
```

2. **Environment-Based Limits**
   - Development: Higher limits for testing
   - Production: Conservative limits for stability
   - Enterprise: Configurable per customer requirements

3. **Limit Validation**
   - Sanity checks on configuration values
   - Runtime limit enforcement
   - Graceful degradation on limit exceeded

#### Phase 2: Streaming Processing Architecture (Week 1-2)

**Objective:** Implement incremental processing for large plans

**Changes Required:**

1. **Streaming Parser** (`src/engines/detection/terraform/streaming.rs`)
   - Chunked JSON parsing
   - Incremental resource processing
   - Memory-bounded processing queues

2. **Pipeline Optimization**
   - Parallel engine processing where possible
   - Intermediate result caching
   - Memory pool management

3. **Resource Management**
   - Memory usage monitoring
   - Automatic cleanup of intermediate results
   - Configurable memory thresholds

#### Phase 3: Performance Monitoring (Week 2)

**Objective:** Add comprehensive performance tracking and alerting

**Changes Required:**

1. **Performance Metrics**
   - Processing time per engine
   - Memory usage patterns
   - Resource utilization statistics

2. **Performance Budgets**
   - Configurable performance thresholds
   - Automatic performance regression detection
   - Performance alerting

3. **Optimization Opportunities**
   - Identify performance bottlenecks
   - Memory optimization recommendations
   - Scaling guidance

### Testing Strategy

1. **Performance Tests:** Large plan processing benchmarks
2. **Load Tests:** Concurrent processing validation
3. **Memory Tests:** Memory leak detection and limit testing
4. **Streaming Tests:** Incremental processing correctness

## 3. Deployment Strategy

### Gradual Rollout Plan

#### Phase 1: Development Environment (Week 1)
- Deploy key management changes to development
- Test with mock key providers
- Validate performance improvements

#### Phase 2: Staging Environment (Week 2)
- Deploy to staging with real key providers
- Test key rotation procedures
- Performance benchmarking with production-like data

#### Phase 3: Production Rollout (Week 3)
- Canary deployment with 10% traffic
- Gradual rollout with monitoring
- Rollback procedures ready

### Rollback Strategy

1. **Feature Flags:** Ability to disable new features
2. **Configuration Rollback:** Revert to previous configurations
3. **Key Management Fallback:** Maintain compatibility with old key format
4. **Monitoring:** Automated rollback triggers on error thresholds

## 4. Risk Mitigation

### Security Risks

1. **Key Exposure During Transition**
   - **Mitigation:** Encrypted key storage, access logging, audit trails

2. **Service Disruption During Key Rotation**
   - **Mitigation:** Multi-version key support, gradual rotation, monitoring

3. **Performance Regression**
   - **Mitigation:** Comprehensive benchmarking, gradual rollout, performance budgets

### Operational Risks

1. **Configuration Complexity**
   - **Mitigation:** Clear documentation, validation, defaults

2. **Monitoring Gaps**
   - **Mitigation:** Comprehensive metrics, alerting, dashboards

3. **Training Requirements**
   - **Mitigation:** Documentation, training sessions, support

## 5. Success Criteria

### Key Management
- ✅ External key providers integrated and tested
- ✅ Key rotation procedures documented and validated
- ✅ No key exposure in build artifacts
- ✅ Audit logging for all key operations

### Performance Scaling
- ✅ Configurable WASM limits implemented
- ✅ Streaming processing for large plans (>100MB)
- ✅ Memory usage reduced by 30% for large plans
- ✅ Performance monitoring and alerting active

### Production Readiness
- ✅ All security tests passing
- ✅ Performance benchmarks meeting targets
- ✅ Documentation updated
- ✅ Operations team trained

## 6. Dependencies & Prerequisites

### External Dependencies
- Key management service (Vault/KMS/Azure Key Vault)
- Monitoring infrastructure (Prometheus/Grafana)
- Configuration management system

### Internal Prerequisites
- Access to key management infrastructure
- Performance testing environment
- Security review approval

## 7. Timeline & Milestones

| Week | Milestone | Deliverables |
|------|-----------|--------------|
| 1 | Core implementation complete | Key provider interface, configurable limits, basic streaming |
| 2 | Integration testing | Full system integration, performance testing, security validation |
| 3 | Production deployment | Gradual rollout, monitoring, documentation |

## 8. Cost Estimate

### Development Costs
- Engineering time: 3-4 weeks
- Security review: 1 week
- Testing infrastructure: $5K-10K

### Operational Costs
- Key management service: $500-2000/month
- Monitoring infrastructure: $200-500/month
- Training: $2K-5K one-time

### Risk Mitigation Costs
- Rollback procedures: $1K
- Additional monitoring: $500/month

## Conclusion

This implementation plan provides a structured approach to address the critical production readiness gaps while maintaining system stability and security. The phased approach allows for thorough testing and gradual rollout, minimizing risk to production operations.

**Recommended Next Step:** Begin Phase 1 implementation with the key provider interface and configurable limits.</content>
<parameter name="filePath">/home/dee/workspace/AI/GuardSuite/CostPilot/docs/production_deployment_plan.md
