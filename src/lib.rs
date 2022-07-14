//! Track and report errors, exceptions and messages from your Rust application to Rollbar.

pub extern crate backtrace;
extern crate futures;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate tokio;

use std::borrow::ToOwned;
use std::sync::Arc;
use std::{error, fmt, panic};
use std::collections::HashMap;

use backtrace::Backtrace;
use futures::TryFutureExt;
use hyper::{Body, Method, Request};
use hyper_tls::HttpsConnector;

#[derive(Clone, Debug)]
pub struct ErrorMessage {
    description: String,
}

impl ErrorMessage {
    pub fn new(new_description: &str) -> ErrorMessage {
        ErrorMessage {
            description: new_description.to_string(),
        }
    }
}

impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl error::Error for ErrorMessage {
    fn description(&self) -> &str {
        &self.description
    }
}

/// Report an error. Any type that implements `error::Error` is accepted.
#[macro_export]
macro_rules! report_error {
    ($err:expr) => {{
        let backtrace = $crate::backtrace::Backtrace::new();
        let line = line!() - 2;
        let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
        let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
        let client = rollbar::Client::new(access_token, environment);

        client
            .build_report()
            .from_error(&$err, None, None)
            .with_frame(
                ::rollbar::FrameBuilder::new()
                    .with_line_number(line)
                    .with_file_name(file!())
                    .build(),
            )
            .with_backtrace(&backtrace)
            .send()
    }};
}

/// Report an error string.
#[macro_export]
macro_rules! report_error_string {
    ($err:expr) => {{
        $crate::report_error!($crate::ErrorMessage::new($err))
    }};
}

/// Report an error, with the request, and custom data.
/// TODO: Unfortunately Rollbar seems to drop "request" even though it matches documentation
/// https://explorer.docs.rollbar.com/#operation/create-item
/// In the interim passing "request" into "custom" does work.
#[macro_export]
macro_rules! report_error_with_request {
    ($err:expr, $request:expr, $custom:expr) => {{
        let backtrace = $crate::backtrace::Backtrace::new();
        let line = line!();
        let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
        let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
        let client = rollbar::Client::new(access_token, environment);

        client
            .build_report()
            .from_error(&$err, $request, $custom)
            .with_frame(
                ::rollbar::FrameBuilder::new()
                    .with_line_number(line)
                    .with_file_name(file!())
                    .build(),
            )
            .with_backtrace(&backtrace)
            .send()
    }};
}

/// Report an error string, with the request, and custom data.
/// TODO: Unfortunately Rollbar seems to drop "request" even though it matches documentation
/// https://explorer.docs.rollbar.com/#operation/create-item
/// In the interim passing "request" into "custom" does work.
#[macro_export]
macro_rules! report_error_string_with_request {
    ($err:expr, $request:expr, $custom:expr) => {{
        $crate::report_error_with_request!($crate::ErrorMessage::new($err), $request, $custom)
    }};
}

/// Report an error message. Any type that implements `fmt::Display` is accepted.
#[macro_export]
macro_rules! report_error_message {
    ($client:ident, $err:expr) => {{
        let backtrace = $crate::backtrace::Backtrace::new();
        let line = line!();

        $client
            .build_report()
            .from_error_message(&$err)
            .with_frame(
                ::rollbar::FrameBuilder::new()
                    .with_line_number(line)
                    .with_file_name(file!())
                    .build(),
            )
            .with_backtrace(&backtrace)
            .send()
    }};
}

/// Set a global hook for the `panic`s your application could raise.
#[macro_export]
macro_rules! report_panics {
    () => {{
        ::std::panic::set_hook(::std::boxed::Box::new(move |panic_info| {
            let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or("".to_string());
            let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or("".to_string());
            let client = rollbar::Client::new(access_token, environment);
            let backtrace = $crate::backtrace::Backtrace::new();
            client
                .build_report()
                .from_panic(panic_info)
                .with_backtrace(&backtrace)
                .send();
        }))
    }};
}

/// Send a plain text message to Rollbar with severity level `INFO`.
#[macro_export]
macro_rules! report_message {
    ($client:ident, $message:expr) => {{
        $client
            .build_report()
            .from_message($message)
            .with_level(::rollbar::Level::INFO)
            .send()
    }};
}

macro_rules! add_field {
    ($n:ident, $f:ident, $t:ty) => (
        pub fn $n(&mut self, val: $t) -> &mut Self {
            self.$f = Some(val);
            self
        }
    );
}

macro_rules! add_generic_field {
    ($n:ident, $f:ident, $t:path) => (
        pub fn $n<T: $t>(&mut self, val: T) -> &mut Self {
            self.$f = Some(val.into());
            self
        }
    );
}

/// Variants for setting the severity level.
/// If not specified, the default value is `ERROR`.
#[derive(Serialize, Clone)]
pub enum Level {
    CRITICAL,
    ERROR,
    WARNING,
    INFO,
    DEBUG,
}

impl<'a> From<&'a str> for Level {
    fn from(s: &'a str) -> Level {
        match s {
            "critical" => Level::CRITICAL,
            "warning" => Level::WARNING,
            "info" => Level::INFO,
            "debug" => Level::DEBUG,
            _ => Level::ERROR,
        }
    }
}

impl ToString for Level {
    fn to_string(&self) -> String {
        match *self {
            Level::CRITICAL => "critical".to_string(),
            Level::ERROR => "error".to_string(),
            Level::WARNING => "warning".to_string(),
            Level::INFO => "info".to_string(),
            Level::DEBUG => "debug".to_string(),
        }
    }
}

// https://rollbar.com/docs/api/items_post/
/// Builder for a generic request to Rollbar.
pub struct ReportBuilder<'a> {
    client: &'a Client,
}

/// Wrapper for a trace, payload of a single exception.
#[derive(Serialize, Default, Debug)]
struct Trace {
    frames: Vec<FrameBuilder>,
    exception: Exception,
}

/// Wrapper for an exception, which describes the occurred error.
#[derive(Serialize, Debug)]
struct Exception {
    class: String,
    message: String,
    description: String,
}

impl Default for Exception {
    fn default() -> Self {
        Exception {
            class: "Generic".to_string(),
            message: String::new(),
            description: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct HttpRequestData {
    headers: HashMap<String, String>,
    method: String,
    url: String,
}

impl HttpRequestData {
    pub fn new(new_headers: &HashMap<String, String>, new_method: &str, new_url: &str) -> HttpRequestData {
        HttpRequestData {
            headers: new_headers.clone(),
            method: new_method.to_string(),
            url: new_url.to_string(),
        }
    }
}

/// Builder for a frame. A collection of frames identifies a stack trace.
#[derive(Serialize, Default, Clone, Debug)]
pub struct FrameBuilder {
    /// The name of the file in which the error had origin.
    #[serde(rename = "filename")]
    file_name: String,

    /// The line of code in in which the error had origin.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lineno")]
    line_number: Option<u32>,

    /// Set the number of the column in which an error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "colno")]
    column_number: Option<u32>,

    /// The method or the function name which caused caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "method")]
    function_name: Option<String>,
}

impl<'a> FrameBuilder {
    pub fn new() -> Self {
        FrameBuilder {
            file_name: file!().to_owned(),
            ..Default::default()
        }
    }

    // Tell the origin of the error by adding the file name to the report.
    pub fn with_file_name<T: Into<String>>(&'a mut self, file_name: T) -> &'a mut Self {
        self.file_name = file_name.into();
        self
    }

    // Set the number of the line in which an error occurred.
    add_field!(with_line_number, line_number, u32);

    // Set the number of the column in which an error occurred.
    add_field!(with_column_number, column_number, u32);

    // Set the method or the function name which caused caused the error.
    add_generic_field!(with_function_name, function_name, Into<String>);

    // Conclude the creation of the frame.
    pub fn build(&self) -> Self {
        self.to_owned()
    }
}

/// Builder specialized for reporting errors.
#[derive(Serialize)]
pub struct ReportErrorBuilder<'a> {
    #[serde(skip_serializing)]
    report_builder: &'a ReportBuilder<'a>,

    /// The trace containing the stack frames.
    trace: Trace,

    /// The severity level of the error. `Level::ERROR` is the default value.
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<Level>,

    /// The title shown in the dashboard for this report.
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    /// The request that caused the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    request: Option<HttpRequestData>,

    /// Custom metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    custom: Option<serde_json::Value>,
}

impl<'a> ReportErrorBuilder<'a> {
    // Attach a `backtrace::Backtrace` to the `description` of the report.
    pub fn with_backtrace(&mut self, backtrace: &'a Backtrace) -> &mut Self {
        self.trace.frames.extend(
            backtrace
                .frames()
                .iter()
                .flat_map(|frames| frames.symbols())
                .map(|symbol|
                    // http://alexcrichton.com/backtrace-rs/backtrace/struct.Symbol.html
                    FrameBuilder {
                        file_name: symbol.filename()
                            .map_or_else(|| "".to_owned(), |p| format!("{}", p.display())),
                        line_number: symbol.lineno(),
                        function_name: symbol.name()
                            .map(|s| format!("{}", s)),
                        ..Default::default()
                    })
                .collect::<Vec<FrameBuilder>>(),
        );

        self
    }

    // Add a new frame to the collection of stack frames.
    pub fn with_frame(&mut self, frame_builder: FrameBuilder) -> &mut Self {
        self.trace.frames.push(frame_builder);
        self
    }

    // Set the security level of the report. `Level::ERROR` is the default value.
    add_generic_field!(with_level, level, Into<Level>);

    // Set the title to show in the dashboard for this report.
    add_generic_field!(with_title, title, Into<String>);

    // Send the report to Rollbar.
    pub fn send(&mut self) -> tokio::task::JoinHandle<Option<ResponseStatus>> {
        let client = self.report_builder.client;
        client.send(self.to_string())
    }
}

impl<'a> ToString for ReportErrorBuilder<'a> {
    fn to_string(&self) -> String {
        let client = self.report_builder.client;

        json!({
            "access_token": client.access_token,
            "data": {
                "environment": client.environment,
                "body": {
                    "trace": self.trace,
                },
                "level": self.level
                    .to_owned()
                    .unwrap_or(Level::ERROR)
                    .to_string(),
                "language": "rust",
                "title": self.title,
                "request": self.request,
                "custom": self.custom,
            }
        })
        .to_string()
    }
}

/// Builder specialized for reporting messages.
pub struct ReportMessageBuilder<'a> {
    report_builder: &'a ReportBuilder<'a>,

    // The message that must be reported.
    message: &'a str,

    // The severity level of the error. `Level::ERROR` is the default value.
    level: Option<Level>,
}

impl<'a> ReportMessageBuilder<'a> {
    // Set the security level of the report. `Level::ERROR` is the default value
    add_generic_field!(with_level, level, Into<Level>);

    // Send the message to Rollbar.
    pub fn send(&mut self) -> tokio::task::JoinHandle<Option<ResponseStatus>> {
        let client = self.report_builder.client;
        client.send(self.to_string())
    }
}

impl<'a> ToString for ReportMessageBuilder<'a> {
    fn to_string(&self) -> String {
        let client = self.report_builder.client;

        json!({
            "access_token": client.access_token,
            "data": {
                "environment": client.environment,
                "body": {
                    "message": {
                        "body": self.message
                    }
                },
                "level": self.level
                    .to_owned()
                    .unwrap_or(Level::INFO)
                    .to_string()
            }
        })
        .to_string()
    }
}

impl<'a> ReportBuilder<'a> {
    /// To be used when a panic report must be sent.
    #[allow(clippy::wrong_self_convention)]
    pub fn from_panic(&'a mut self, panic_info: &'a panic::PanicInfo) -> ReportErrorBuilder<'a> {
        let mut trace = Trace::default();

        let payload = panic_info.payload();
        let message = match payload.downcast_ref::<&str>() {
            Some(s) => *s,
            None => match payload.downcast_ref::<String>() {
                Some(s) => s,
                None => "Box<Any>",
            },
        };
        trace.exception.class = "<panic>".to_owned();
        trace.exception.message = message.to_owned();
        trace.exception.description = trace.exception.message.to_owned();

        if let Some(location) = panic_info.location() {
            trace.frames.push(FrameBuilder {
                file_name: location.file().to_owned(),
                line_number: Some(location.line()),
                ..Default::default()
            });
        }

        ReportErrorBuilder {
            report_builder: self,
            trace,
            level: None,
            title: Some(message.to_owned()),
            request: None,
            custom: None
        }
    }

    // TODO: remove self?
    /// To be used when an `error::Error` must be reported.
    pub fn from_error<E: error::Error>(&'a mut self, error: &'a E, request: Option<HttpRequestData>, custom: Option<serde_json::Value>) -> ReportErrorBuilder<'a> {
        let mut trace = Trace::default();
        trace.exception.class = std::any::type_name::<E>().to_owned();
        trace.exception.message = error.to_string();
        trace.exception.description = error
            .source()
            .map_or_else(|| format!("{:?}", error), |c| format!("{:?}", c));

        ReportErrorBuilder {
            report_builder: self,
            trace,
            level: None,
            title: Some(format!("{}", error)),
            request,
            custom
        }
    }

    /// To be used when a error message must be reported.
    pub fn from_error_message<T: fmt::Display>(
        &'a mut self,
        error_message: &'a T,
    ) -> ReportErrorBuilder<'a> {
        let message = format!("{}", error_message);

        let mut trace = Trace::default();
        trace.exception.class = std::any::type_name::<T>().to_owned();
        trace.exception.message = message.to_owned();
        trace.exception.description = message.to_owned();

        ReportErrorBuilder {
            report_builder: self,
            trace,
            level: None,
            title: Some(message),
            request: None,
            custom: None
        }
    }

    /// To be used when a message must be tracked by Rollbar.
    pub fn from_message(&'a mut self, message: &'a str) -> ReportMessageBuilder<'a> {
        ReportMessageBuilder {
            report_builder: self,
            message,
            level: None,
        }
    }
}

/// The access point to the library.
pub struct Client {
    http_client: Arc<hyper::Client<HttpsConnector<hyper::client::HttpConnector>>>,
    access_token: String,
    environment: String,
}

impl Client {
    /// Create a new `Client`.
    ///
    /// Your available `environment`s are listed at
    /// <https://rollbar.com/{your_organization}/{your_app}/settings/general>.
    ///
    /// You can get the `access_token` at
    /// <https://rollbar.com/{your_organization}/{your_app}/settings/access_tokens>.
    pub fn new<T: Into<String>>(access_token: T, environment: T) -> Client {
        let https = HttpsConnector::new();
        let client = hyper::Client::builder()
            .build::<_, Body>(https);

        Client {
            http_client: Arc::new(client),
            access_token: access_token.into(),
            environment: environment.into(),
        }
    }

    /// Create a `ReportBuilder` to build a new report for Rollbar.
    pub fn build_report(&self) -> ReportBuilder {
        ReportBuilder {
            client: self,
        }
    }

    /// Function used internally to send payloads to Rollbar.
    fn send(&self, payload: String) -> tokio::task::JoinHandle<Option<ResponseStatus>> {
        let body = hyper::Body::from(payload);
        let url = std::env::var("ROLLBAR_ENDPOINT").unwrap_or_else(|_| "".to_string());
        if url.len() == 0 {
            // we should log that the environment variable is missing on load, but dont spew after that
            return tokio::task::spawn(
                async move {
                    None::<ResponseStatus>
                }
            );
        }
        let request = Request::builder()
            .method(Method::POST)
            .uri(url)
            .body(body)
            .expect("Cannot build post request!");

        let response_future = self
            .http_client
            .request(request)
            .map_ok(|res| {
                Some(ResponseStatus::from(res.status()))
            })
            .map_err(|error| {
                println!("Error while sending a report to Rollbar.");
                print!("The error returned by Rollbar was: {:?}.\n\n", error);
                None::<ResponseStatus>
            });

        tokio::task::spawn(
            async move {
                response_future.await.ok().flatten()
            }
        )
    }
}

/// Wrapper for `hyper::StatusCode`.
#[derive(Debug)]
pub struct ResponseStatus(hyper::StatusCode);

impl From<hyper::StatusCode> for ResponseStatus {
    fn from(status_code: hyper::StatusCode) -> ResponseStatus {
        ResponseStatus(status_code)
    }
}

impl ResponseStatus {
    /// Return a description provided by Rollbar for the status code returned by each request.
    pub fn description(&self) -> &str {
        match self.0.as_u16() {
            200 => "The item was accepted for processing.",
            400 => "No JSON payload was found, or it could not be decoded.",
            401 => "No access token was found in the request.",
            403 => "Check that your `access_token` is valid, enabled, and has the correct scope. The response will contain a `message` key explaining the problem.",
            413 => "Max payload size is 128kb. Try removing or truncating unnecessary large data included in the payload, like whole binary files or long strings.",
            422 => "A syntactically valid JSON payload was found, but it had one or more semantic errors. The response will contain a `message` key describing the errors.",
            429 => "Request dropped because the rate limit has been reached for this access token, or the account is on the Free plan and the plan limit has been reached.",
            500 => "There was an error on Rollbar's end",
            _   => "An undefined error occurred."
        }
    }

    /// Return the canonical description for the status code returned by each request.
    pub fn canonical_reason(&self) -> String {
        format!("{}", self.0)
    }
}

impl fmt::Display for ResponseStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Error {}: {}",
            self.canonical_reason(),
            self.description()
        )
    }
}

#[cfg(test)]
mod tests {
    extern crate backtrace;
    extern crate hyper;
    extern crate serde_json;

    use std::collections::HashMap;
    use std::panic;
    use std::sync::mpsc::channel;
    use std::sync::{Arc, Mutex};

    use backtrace::Backtrace;
    use serde_json::Value;

    use super::{Client, FrameBuilder, Level};

    macro_rules! normalize_frames {
        ($payload:expr, $expected_payload:expr, $expected_frames:expr) => {
            // check the description/backtrace is is not empty and also check
            // that it is different from the message and then ignore it from now on
            let payload_ = $payload.to_owned();
            let description = payload_
                .get("data")
                .unwrap()
                .get("body")
                .unwrap()
                .get("trace")
                .unwrap()
                .get("exception")
                .unwrap()
                .get("description")
                .unwrap();
            let message = payload_
                .get("data")
                .unwrap()
                .get("body")
                .unwrap()
                .get("trace")
                .unwrap()
                .get("exception")
                .unwrap()
                .get("message")
                .unwrap();

            match description {
                &Value::String(ref s) => assert!(!s.is_empty()),
                _ => assert!(false),
            }
            match message {
                &Value::String(ref s) => assert!(!s.is_empty()),
                _ => assert!(false),
            }

            $payload
                .get_mut("data")
                .unwrap()
                .get_mut("body")
                .unwrap()
                .get_mut("trace")
                .unwrap()
                .get_mut("frames")
                .unwrap()
                .as_array_mut()
                .unwrap()
                .truncate($expected_frames);
        };
    }

    #[test]
    fn test_report_panics() {
        let (tx, rx) = channel();

        {
            let tx = Arc::new(Mutex::new(tx));

            let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or_else(|_| "ROLLBAR_ACCESS_TOKEN".to_string());
            let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or_else(|_| "ROLLBAR_ENVIRONMENT".to_string());
            let client = Client::new(access_token, environment);
            panic::set_hook(Box::new(move |panic_info| {
                let backtrace = Backtrace::new();
                let payload = client
                    .build_report()
                    .from_panic(panic_info)
                    .with_backtrace(&backtrace)
                    .with_level("info")
                    .to_string();
                let payload = Arc::new(Mutex::new(payload));
                tx.lock().unwrap().send(payload).unwrap();
            }));

            let result = panic::catch_unwind(|| {
                // just to trick the linter
                let zero = "0".parse::<i32>().unwrap();
                let _ = 1 / zero;
            });
            assert!(result.is_err());
        }

        // remove the hook to avoid double panics
        let _ = panic::take_hook();

        let lock = rx.recv().unwrap();
        let payload = match lock.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        let mut payload: Value = serde_json::from_str(&*payload).unwrap();
        let mut expected_payload = json!({
            "access_token": "ROLLBAR_ACCESS_TOKEN",
            "data": {
                "environment": "ROLLBAR_ENVIRONMENT",
                "body": {
                    "trace": {
                        "frames": [{
                            "filename": "src/lib.rs",
                            "lineno": 268
                        }],
                        "exception": {
                            "class": "<panic>",                            "message": "attempt to divide by zero",
                            "description": "attempt to divide by zero"
                        }
                    }
                },
                "level": "info",
                "language": "rust",
                "title": "attempt to divide by zero",
                "request": null,
                "custom": null
            }
        });

        let payload_ = payload.to_owned();
        let line_number = payload_
            .get("data")
            .unwrap()
            .get("body")
            .unwrap()
            .get("trace")
            .unwrap()
            .get("frames")
            .unwrap()
            .get(0)
            .unwrap()
            .get("lineno")
            .unwrap();

        assert!(line_number.as_u64().unwrap() > 0);

        *expected_payload
            .get_mut("data")
            .unwrap()
            .get_mut("body")
            .unwrap()
            .get_mut("trace")
            .unwrap()
            .get_mut("frames")
            .unwrap()
            .get_mut(0)
            .unwrap()
            .get_mut("lineno")
            .unwrap() = line_number.to_owned();

        normalize_frames!(payload, expected_payload, 1);
        assert_eq!(expected_payload.to_string(), payload.to_string());
    }

    #[test]
    fn test_report_error() {
        let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or_else(|_| "ROLLBAR_ACCESS_TOKEN".to_string());
        let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or_else(|_| "ROLLBAR_ENVIRONMENT".to_string());
        let client = Client::new(access_token, environment);

        match "笑".parse::<i32>() {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                let payload = client
                    .build_report()
                    .from_error_message(&e)
                    .with_level(Level::WARNING)
                    .with_frame(FrameBuilder::new().with_column_number(42).build())
                    .with_frame(FrameBuilder::new().with_column_number(24).build())
                    .with_title("w")
                    .to_string();

                let expected_payload = json!({
                    "access_token": "ROLLBAR_ACCESS_TOKEN",
                    "data": {
                        "environment": "ROLLBAR_ENVIRONMENT",
                        "body": {
                            "trace": {
                                "frames": [{
                                    "filename": "src/lib.rs",
                                    "colno": 42
                                }, {
                                    "filename": "src/lib.rs",
                                    "colno": 24
                                }],
                                "exception": {
                                    "class": "core::num::error::ParseIntError",
                                    "message": "invalid digit found in string",
                                    "description": "invalid digit found in string"
                                }
                            }
                        },
                        "level": "warning",
                        "language": "rust",
                        "title": "w",
                        "request": null,
                        "custom": null
                    }
                });

                let mut payload: Value = serde_json::from_str(&*payload).unwrap();
                normalize_frames!(payload, expected_payload, 2);
                assert_eq!(expected_payload.to_string(), payload.to_string());
            }
        }
    }

    #[test]
    fn test_report_error_with_request() {
        let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or_else(|_| "ROLLBAR_ACCESS_TOKEN".to_string());
        let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or_else(|_| "ROLLBAR_ENVIRONMENT".to_string());
        let client = Client::new(access_token, environment);

        match "笑".parse::<i32>() {
            Ok(_) => {
                unreachable!();
            }
            Err(e) => {
                let originating_request = crate::HttpRequestData::new(
                    &HashMap::from([
                        ("Mercury".to_owned(), "tiny".to_owned()),
                        ("Venus".to_owned(), "hot".to_owned()),
                        ("Earth".to_owned(), "just right".to_owned()),
                        ("Mars".to_owned(), "doom".to_owned()),
                    ]),
                    "GET",
                    "/the/planets",
                );
                let custom = json!(originating_request);
                let payload = client
                    .build_report()
                    .from_error(&e, Some(originating_request), Some(custom))
                    .with_level(Level::WARNING)
                    .with_frame(FrameBuilder::new().with_column_number(42).build())
                    .with_frame(FrameBuilder::new().with_column_number(24).build())
                    .with_title("w")
                    .to_string();

                let expected_payload = json!({
                    "access_token": "ROLLBAR_ACCESS_TOKEN",
                    "data": {
                        "environment": "ROLLBAR_ENVIRONMENT",
                        "body": {
                            "trace": {
                                "frames": [{
                                    "filename": "src/lib.rs",
                                    "colno": 42
                                }, {
                                    "filename": "src/lib.rs",
                                    "colno": 24
                                }],
                                "exception": {
                                    "class": "core::num::error::ParseIntError",
                                    "message": "invalid digit found in string",
                                    "description": "ParseIntError { kind: InvalidDigit }"
                                }
                            }
                        },
                        "level": "warning",
                        "language": "rust",
                        "title": "w",
                        "request": {
                            "headers": {
                                "Earth": "just right",
                                "Mars": "doom",
                                "Mercury": "tiny",
                                "Venus": "hot"
                            },
                            "method": "GET",
                            "url": "/the/planets"
                        },
                        "custom": {
                            "headers": {
                                "Earth": "just right",
                                "Mars": "doom",
                                "Mercury": "tiny",
                                "Venus": "hot"
                            },
                            "method": "GET",
                            "url": "/the/planets"
                        }
                    }
                });

                let mut payload: Value = serde_json::from_str(&*payload).unwrap();
                normalize_frames!(payload, expected_payload, 2);
                assert_eq!(expected_payload.to_string(), payload.to_string());
            }
        }
    }

    #[test]
    fn test_report_message() {
        let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or_else(|_| "ROLLBAR_ACCESS_TOKEN".to_string());
        let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or_else(|_| "ROLLBAR_ENVIRONMENT".to_string());
        let client = Client::new(access_token, environment);

        let payload = client
            .build_report()
            .from_message("hai")
            .with_level("warning")
            .to_string();

        let expected_payload = json!({
            "access_token": "ROLLBAR_ACCESS_TOKEN",
            "data": {
                "environment": "ROLLBAR_ENVIRONMENT",
                "body": {
                    "message": {
                        "body": "hai"
                    }
                },
                "level": "warning"
            }
        })
        .to_string();

        assert_eq!(payload, expected_payload);
    }

    #[tokio::test]
    async fn test_response() {
        std::env::set_var("ROLLBAR_ENDPOINT", "https://api.rollbar.com/api/1/item/");
        std::env::set_var("ROLLBAR_ACCESS_TOKEN", "ROLLBAR_ACCESS_TOKEN");
        std::env::set_var("ROLLBAR_ENVIRONMENT", "ROLLBAR_ENVIRONMENT");
        let access_token = std::env::var("ROLLBAR_ACCESS_TOKEN").unwrap_or_else(|_| "".to_string());
        let environment = std::env::var("ROLLBAR_ENVIRONMENT").unwrap_or_else(|_| "".to_string());
        let client = Client::new(access_token, environment);

        let status_handle = client
            .build_report()
            .from_message("hai")
            .with_level("info")
            .send();

        match status_handle.await.unwrap() {
            Some(status) => {
                assert_eq!(
                    status.to_string(),
                    "Error 401 Unauthorized: No access token was found in the request.".to_owned()
                );
            }
            None => {
                unreachable!();
            }
        }
    }
}
