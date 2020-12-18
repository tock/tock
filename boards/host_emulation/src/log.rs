#[macro_export]
macro_rules! log {
    (  $lvl:expr, $( $x:expr ),* ) => {
        {
            use crate::emulation_config::Config;
            let lvl = $lvl as u8;
            let log_lvl = Config::get().emulation_log_level as u8;
            if ( lvl <= log_lvl ) {
                println!("KERN {:?}: {}", $lvl, format!($( $x),* ));
            }
        }
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! log_error {
    ( $( $x:expr ),* ) => {
        {
            use crate::emulation_config::LogLevel;
            log!(LogLevel::ERROR, $( $x),* )
        }
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! log_warn {
    ( $( $x:expr ),* ) => {
        {
            use crate::emulation_config::LogLevel;
            log!(emulation_config::LogLevel::WARNING, $( $x),* )
        }
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! log_info {
    ( $( $x:expr ),* ) => {
        {
            use crate::emulation_config::LogLevel;
            log!(LogLevel::INFO, $( $x),* )
        }
    }
}

#[allow(unused_macros)]
#[macro_export]
macro_rules! log_dbg {
    ( $( $x:expr ),* ) => {
        {
            use crate::emulation_config::LogLevel;
            log!(LogLevel::DEBUG, $( $x),* )
        }
    }
}
