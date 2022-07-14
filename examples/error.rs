#[macro_use]
extern crate rollbar;
extern crate backtrace;
use rollbar::{ErrorMessage, HttpRequestData};

fn main() {
    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            let error_string = &e.to_string();
            report_error!(ErrorMessage::new(error_string));
        }
    }

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            let error_string = &e.to_string();
            report_error_string!(error_string);
        }
    }

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            let error_string = &e.to_string();
            report_error_with_request!(ErrorMessage::new(error_string), None, None);
        }
    }

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            let error_string = &e.to_string();
            let originating_request = HttpRequestData::new(
                &std::collections::HashMap::from([
                    ("Mercury".to_owned(), "tiny".to_owned()),
                    ("Venus".to_owned(), "hot".to_owned()),
                    ("Earth".to_owned(), "just right2".to_owned()),
                    ("Mars".to_owned(), "doom".to_owned()),
                ]),
                "GET",
                "/the/planets",
            );
            let custom = serde_json::json!(originating_request);
            report_error_with_request!(ErrorMessage::new(error_string), Some(originating_request), Some(custom));
        }
    }

    match "笑".parse::<i32>() {
        Ok(_) => { println!("lolnope"); },
        Err(e) => {
            let error_string = &e.to_string();
            let originating_request = HttpRequestData::new(
                &std::collections::HashMap::from([
                    ("Mercury".to_owned(), "tiny".to_owned()),
                    ("Venus".to_owned(), "hot2".to_owned()),
                    ("Earth".to_owned(), "just right".to_owned()),
                    ("Mars".to_owned(), "doom".to_owned()),
                ]),
                "GET",
                "/the/planets",
            );
            let custom = serde_json::json!(originating_request);
            report_error_string_with_request!(error_string, Some(originating_request), Some(custom));
        }
    }
}
