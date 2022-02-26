#[macro_use]
extern crate rollbar;
extern crate backtrace;
use rollbar::ErrorMessage;

fn main() {
    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            let error_string = &e.to_string();
            report_error!(error_string);
        }
    }
}
