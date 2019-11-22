/// Prints an error message and  usage string. Used to report command line
/// argument errors.
fn usage_error(message: &str) {
    println!(
        "{}

Usage: ALERT_CODE <code>
Print a description of ALERT_CODE.

ALERT_CODE must be specified in hexadecimal, with or without a 0x prefix.

Examples:
  alert_code 0x01  Prints a description of alert 1",
        message
    );
}

/// Returns the alert code specified on the command line, or prints a usage
/// error if it was omitted or incorrectly specified.
fn get_alert_code() -> Result<u32, ()> {
    let mut args = std::env::args_os();
    if args.len() != 2 {
        usage_error("Incorrect number of arguments");
        return Err(());
    }
    let code_os_str = args.nth(1).expect("Unable to read alert_code");
    let code_string = if let Ok(code) = code_os_str.into_string() {
        code
    } else {
        usage_error(
            "alert_code is not valid Unicode. \
             Expecting a hexadecimal integer.",
        );
        return Err(());
    };
    let parse_result = u32::from_str_radix(code_string.trim_start_matches("0x"), 16);
    let code = if let Ok(code) = parse_result {
        code
    } else {
        usage_error("alert_code must be a hexadecimal integer.");
        return Err(());
    };
    Ok(code)
}

fn main() {
    let alert_code = match get_alert_code() {
        Ok(code) => code,
        _ => return,
    };
    let message = match alert_code {
        0x01 => "Application panic (e.g. a Rust application called panic!())",

        0x02 => {
            "A statically-linked app was not installed \
             in the correct location in flash."
        }

        _ => "Unknown alert code",
    };
    println!("{}", message);
}
