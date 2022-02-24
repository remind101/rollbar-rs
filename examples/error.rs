#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => { let _ = report_error!(e).join(); }
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
