use cc26x2::commands as cmd;

#[derive(Debug, Clone, Copy)]
pub enum RfcOperationStatus {
    Idle,
    Pending,
    Active,
    Skipped,
    SendDone,
    CommandDone,
    LastCommandDone,
    RxOk,
    TxDone,
    Setup,
    Invalid,
}

#[derive(Debug, Clone, Copy)]
pub enum State {
    Start,
    Pending,
    CommandStatus(RfcOperationStatus),
    Done,
    Invalid,
}

type CommandStatus = Result<u32, u32>;

#[derive(Clone, Copy)]
pub enum RFCommandStatus {
    // Operation not finished
    Idle = 0x0000,
    Pending = 0x0001,
    Active = 0x0002,
    Skipped = 0x0003,
    // Operation finished normally
    DoneOK = 0x0400,
    DoneCountdown = 0x0401,
    DoneRxErr = 0x0402,
    DoneTimeout = 0x0403,
    DoneStopped = 0x0404,
    DoneAbort = 0x0405,
    // Operation finished with error
    ErrorPastStart = 0x0800,
    ErrorStartTrig = 0x0801,
    ErrorCondition = 0x0802,
    ErrorPar = 0x0803,
    ErrorPointer = 0x0804,
    ErrorCmdID = 0x0805,
    ErrorNoSetup = 0x0807,
    ErrorNoFS = 0x0808,
    ErrorSynthProg = 0x0809,
    ErrorTxUNF = 0x080A,
    ErrorRxOVF = 0x080B,
    ErrorNoRx = 0x080C,
}

pub static mut CMD_STACK: [RadioCommands; 6] = [
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
    RadioCommands::NotSupported,
];

#[derive(Debug, Clone, Copy)]
pub enum RadioCommands {
    Direct { c: cmd::DirectCommand },
    RadioSetup { c: cmd::CmdRadioSetup },
    Common { c: cmd::CmdNop },
    FSPowerup { c: cmd::CmdFSPowerup },
    FSPowerdown{ c: cmd::CmdFSPowerdown },
    StartRat { c: cmd::CmdSyncStartRat },
    StopRat { c: cmd::CmdSyncStopRat },
    NotSupported,
}

/*
impl Default for RadioCommands {
    fn default() -> RadioCommands {
        RadioCommands::Common { cmd::CmdNop::new() }
    }
}
*/

impl From<usize> for RfcOperationStatus {
    fn from(val: usize) -> RfcOperationStatus {
        match val {
            0 => RfcOperationStatus::Idle,
            1 => RfcOperationStatus::Pending,
            2 => RfcOperationStatus::Active,
            3 => RfcOperationStatus::Skipped,
            4 => RfcOperationStatus::SendDone,
            5 => RfcOperationStatus::TxDone,
            6 => RfcOperationStatus::CommandDone,
            7 => RfcOperationStatus::LastCommandDone,
            8 => RfcOperationStatus::RxOk,
            9 => RfcOperationStatus::TxDone,
            val => {
                debug_assert!(false, "{} does not represent a valid command.", val);
                RfcOperationStatus::Invalid
            }
        }
    }
}
