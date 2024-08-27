// Copyright OxidOS Automotive 2024.

/// Types of ssyscall filters available for Tock platforms.
#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone, Copy)]
pub enum SyscallFilterType {
    #[default]
    None,
    TbfHeaderFilterDefaultAllow,
}

impl std::fmt::Display for SyscallFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyscallFilterType::None => write!(f, "None"),
            SyscallFilterType::TbfHeaderFilterDefaultAllow => {
                write!(f, "TbfHeaderFilterDefaultAllow")
            }
        }
    }
}
