use tock_tbf::types::TbfFooterV2Credentials;

pub enum VerificationResult {
    Accept,
    Pass,
    Reject
}

pub trait Verify {
    fn require_credentials(&self) -> bool;
    fn check_credentials(&self,
                         credentials: &TbfFooterV2Credentials,
                         binary: &[u8]) -> VerificationResult;
}

pub struct VerifierStandard {}

impl Verify for VerifierStandard {
    fn require_credentials(&self) -> bool {
        true
    }
    fn check_credentials(&self,
                         _credentials: &TbfFooterV2Credentials,
                         _binary: &[u8]) -> VerificationResult {
        VerificationResult::Pass
    }
}
