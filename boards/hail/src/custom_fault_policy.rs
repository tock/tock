use kernel::process;
use kernel::process::Process;
use kernel::process::ProcessFaultPolicy;

/// This policy differentiates what happens when a process faults based on its
/// credentials. If the process is credentialed, it restarts, if not, it is
/// stopped.
pub struct CredentialedFaultPolicy {}

impl CredentialedFaultPolicy {
    pub const fn new() -> CredentialedFaultPolicy {
        CredentialedFaultPolicy {}
    }
}

impl ProcessFaultPolicy for CredentialedFaultPolicy {
    fn action(&self, proc: &dyn Process) -> process::FaultAction {
        match proc.get_credentials() {
            Some(_credential) => process::FaultAction::Restart,
            None => process::FaultAction::Stop,
        }
    }
}
