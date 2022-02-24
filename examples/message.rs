#[macro_use]
extern crate rollbar;

fn main() {
    let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
    let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
    let client = rollbar::Client::new(access_token, environment);
    let _ = report_message!(client, "hai").join();

    /* // `report_message!` expands to the following code:
     * client.build_report()
     *     .from_message("hai")
     *     .with_level(rollbar::Level::INFO)
     *     .send();
     * // If you want to customize the message, you might not want to use the macro.
     * // Join the thread only for testing purposes.
    */
}
