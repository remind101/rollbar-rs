#[macro_use]
extern crate rollbar;
extern crate backtrace;

fn main() {
    let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
    let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
    let client = rollbar::Client::new(access_token, environment);
    report_panics!(client);

    /* // `report_panics!` expands to the following code:
     * std::panic::set_hook(Box::new(move |panic_info| {
     *     let backtrace = backtrace::Backtrace::new();
     *     client.build_report()
     *         .from_panic(panic_info)
     *         .with_backtrace(&backtrace)
     *         .send();
     * }));
     * // If you want to customize the reports, you might not want to use the macro.
     * // Join the thread only for testing purposes.
    */

    let zero = "0".parse::<i32>().unwrap(); // let's trick the lint a bit!
    let _ = 42/zero;
}
