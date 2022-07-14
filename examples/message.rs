#[macro_use]
extern crate rollbar;

#[tokio::main]
async fn main() {
    println!("report message");
    let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
    let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
    let client = rollbar::Client::new(access_token, environment);
    let result = report_message!(client, "hai").await;
    println!("{:#?}", result);

    /* // `report_message!` expands to the following code:
     * client.build_report()
     *     .from_message("hai")
     *     .with_level(rollbar::Level::INFO)
     *     .send();
     * // If you want to customize the message, you might not want to use the macro.
     * // Join the thread only for testing purposes.
    */
}
