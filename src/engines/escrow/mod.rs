// Software escrow module exports

pub mod package;
pub mod recovery;
pub mod release;

pub use package::{
    ArtifactType, BuildArtifact, BuildInstructions, CompletenessCheck, DependenciesManifest,
    DepositType, EscrowPackage, EscrowPackageBuilder, FileType, LicenseInfo, PackageMetadata,
    SourceFile, VendorInfo, VerificationData, VerificationReport,
};

pub use recovery::{RecoveryOrchestrator, RecoveryPlaybook, RecoveryReport, RecoveryStep};

pub use release::{
    DepositReceipt, DepositStatus, EscrowAgentConfig, ReleaseAutomation, ReleaseConfig,
    ReleaseTrigger,
};
