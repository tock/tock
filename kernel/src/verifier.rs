use tock_tbf::types::TbfFooterV2Credentials;
use tock_tbf::types::TbfFooterV2CredentialsType;

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

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ShortID {
    id: u32
}

pub trait Compress {
    fn to_short_id(credentials: &TbfFooterV2Credentials) -> Option<ShortID>;
}


pub struct VerifierStandard {}

impl Verify for VerifierStandard {
    fn require_credentials(&self) -> bool {
        true
    }
    fn check_credentials(&self,
                         credentials: &TbfFooterV2Credentials,
                         _binary: &[u8]) -> VerificationResult {
        match credentials.format {
            TbfFooterV2CredentialsType::Rsa4096Key => {
                VerificationResult::Accept
            },
            _ => VerificationResult::Pass,
        }
    }
}

impl Compress for VerifierStandard {
    fn to_short_id(credentials: &TbfFooterV2Credentials) -> Option<ShortID> {
        match credentials.format {
            TbfFooterV2CredentialsType::Padding => None,
            TbfFooterV2CredentialsType::CleartextID => {
                // Convert big-endian to little endian
                let val: u32 = credentials.data[0] as u32 |
                               (credentials.data[1] as u32) << 8 |
                               (credentials.data[3] as u32) << 16 |
                               (credentials.data[3] as u32) << 24;
                Some(ShortID {id: val})
            },
            TbfFooterV2CredentialsType::Rsa3072Key => None,
            TbfFooterV2CredentialsType::Rsa4096Key => None,
            TbfFooterV2CredentialsType::Rsa3072KeyWithID => None,
            TbfFooterV2CredentialsType::Rsa4096KeyWithID => None,
        } 
    }
}

