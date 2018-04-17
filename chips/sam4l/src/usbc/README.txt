Union
    _Always (RAMACERE)
    Disabled ()
Ctrl
    Init (RXSTP)
      RXSTP => match client.ctrl_setup() {
                 Ok_IN => ReadIn
                 Ok_OUT => WriteOut
                 _ => stall(); Init
    ReadIn (TXIN?, NAKOUT)
      NAKOUT => CtrlReadStatus
      TXIN => match client.ctrl_in() {
                Ok(transfer_complete, ..) =>
                  if transfer_complete { !TXIN } else {}
                Delay =>
                  InDelay
                _ =>
                  Init
    ReadStatus (RXOUT)
      RXOUT => Init
    WriteOut (RXOUT, NAKIN),
      RXOUT => match client.ctrl_out() {
                 Ok => !RXOUT
                 Delay => !RXOUT
                 _ => stall(); Init
      NAKIN => CtrlWriteStatus
    WriteStatus (TXIN),
      TXIN => WriteStatusWait
    WriteStatusWait (TXIN),
      TXIN => Init
    CtrlInDelay (NAKOUT)
      {}
BulkIn
    Init (RXSTP, TXIN)
      TXIN => match client.bulk_in() {
                Ok => !FIFOCON
                Delay => BulkInDelay
                _ => stall();
    Delay (RXSTP)
