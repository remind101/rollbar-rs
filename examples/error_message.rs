#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
    let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
    let client = rollbar::Client::new(access_token, environment);
    let _ = report_error_message!(client, "＿|￣|○").join();

    /* // `report_error_message!` expands to the following code:
     * let backtrace = backtrace::Backtrace::new();
     * let line = line!();
     *
     * client.build_report()
     *     .from_error_message("＿|￣|○")
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
