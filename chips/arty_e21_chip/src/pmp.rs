//! Instantiate the PMP for the e21 core.

use rv32i::PMPConfigMacro;

// The arty-e21 has four PMP entries.
PMPConfigMacro!(4);
