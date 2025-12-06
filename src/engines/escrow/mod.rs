// Software escrow module exports

pub mod package;
pub mod recovery;
pub mod release;

pub use package::{
    EscrowPackage, EscrowPackageBuilder, PackageMetadata, VendorInfo,
    SourceFile, BuildArtifact, DependenciesManifest, BuildInstructions,
    LicenseInfo, VerificationReport, VerificationData, CompletenessCheck,
    DepositType, FileType, ArtifactType,
};

pub use recovery::{
    RecoveryOrchestrator, RecoveryReport, RecoveryStep, RecoveryPlaybook,
};

pub use release::{
    ReleaseAutomation, ReleaseConfig, DepositReceipt, DepositStatus,
    EscrowAgentConfig, ReleaseTrigger,
};
