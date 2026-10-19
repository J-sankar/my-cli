pub struct CallbackData {
    pub code: String,
    pub state: String,
}

use std::{
    io::{BufRead, BufReader, Write}, net::TcpListener,
};
use url::Url;

pub fn get_code() -> Result<CallbackData, String> {
    let server = tiny_http::Server::http("127.0.0.1:8080").map_err(|error| error.to_string())?;
    loop {
        // blocks until the next request is received
        match server.recv() {
            Ok(mut rq) => {
                if rq.url().starts_with("/callback") {
                    let full_url = format!("http://127.0.0.1:8080{}", rq.url());
                    let url = url::Url::parse(&full_url).map_err(|error| error.to_string())?;

                    let mut code: Option<String> = None;
                    let mut state: Option<String> = None;

                    for (key, value) in url.query_pairs() {
                        match key.as_ref() {
                            "code" => code = Some(value.into_owned()),
                            "state" => state = Some(value.into_owned()),
                            "error" => return Err(format!("Google denied login: {value}")),
                            _ => {}
                        }
                    }
                    let code = code
                        .ok_or_else(|| "Google did not return an authorization code".to_string())?;
                    let state =
                        state.ok_or_else(|| "Google did not return a state value".to_string())?;
                    rq.respond(tiny_http::Response::from_string(
                        "Login successful. You can close this tab and return to the terminal.",
                    ))
                    .map_err(|error| error.to_string())?;

                    return Ok(CallbackData { code, state });
                }
                rq.respond(tiny_http::Response::from_string("Not found").with_status_code(404))
                    .map_err(|error| error.to_string())?;
            }
            Err(e) => {
                println!("error: {}", e);
                return Err(format!("{}", e.to_string()));
            }
        };

        // do something with the request
        // ...
    }
}

pub fn bind_listener() -> Result<(TcpListener, u16), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    Ok((listener, port))
}

pub fn wait_for_code(listener: &TcpListener) -> Result<CallbackData, String> {
    loop {
        let (mut stream, _) = listener.accept().map_err(|error| error.to_string())?;
        let mut reader = BufReader::new(&stream);

        let mut request = String::new();
        reader.read_line(&mut request).map_err(|error|error.to_string())?;
        println!("{request}");
        let path = request.split_whitespace().nth(1).unwrap_or("");
        let url =
            Url::parse(&format!("http:127.0.0.1{path}")).map_err(|error| error.to_string())?;
        println!("{}", url.as_str());
        let mut code = None;
        let mut state =None ;
        let mut error=None ;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "code" => code = Some(value.into_owned()),
                "state" => state = Some(value.into_owned()),
                "error" => error = Some(value.into_owned()),
                _ => {}
            }
        }
        let code = code.ok_or_else(|| "Google did not return an authorization code".to_string())?;
        let state = state.ok_or_else(|| "Google did not return a state value".to_string())?;

        let body = if error.is_some() {
            "Login failed. You can close this tab."
        } else {
            "Login successful! You can close this tab."
        };
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(), body
        );
        stream.write_all(response.as_bytes()).map_err(|error|error.to_string())?;

        return Ok(CallbackData {
            code,
            state
        });
    }
}
