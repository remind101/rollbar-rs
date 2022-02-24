#[macro_use]
extern crate rollbar;
extern crate backtrace;
use std::env;

fn main() {
    let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
    let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
    let client = rollbar::Client::new(access_token, environment);

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => { let _ = report_error!(client, e).join(); }
    }

    /* // `report_error!` expands to the following code:
     * let backtrace = backtrace::Backtrace::new();
     * let line = line!() - 2;
     *
     * client.build_report()
     *     .from_error(&e)
     *     .with_frame(rollbar::FrameBuilder::new()
     *                 .with_line_number(line)
     *                 .with_file_name(file!())
     *                 .build())
     *     .with_backtrace(&backtrace)
     *     .send();
     * // If you want to customize the report, you might not want to use the macro.
     * // Join the thread only for testing purposes.
    */
}
