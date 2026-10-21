pub struct CallbackData {
    pub code: String,
    pub state: String,
}

use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};
use url::Url;

pub fn bind_listener() -> Result<(TcpListener, u16), String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|error| error.to_string())?;
    let port = listener
        .local_addr()
        .map_err(|error| error.to_string())?
        .port();
    Ok((listener, port))
}

pub fn wait_for_code(
    listener: &TcpListener,
    expected_state: &str,
    timeout: Duration,
) -> Result<CallbackData, String> {
    listener
        .set_nonblocking(true)
        .map_err(|err| err.to_string())?;
    let deadline = Instant::now() + timeout;
    loop {
        if Instant::now() >= deadline {
            return Err("timeout on waiting for google.".to_string());
        }

        let mut stream = match listener.accept() {
            Ok((s, _)) => s,
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => return Err(e.to_string()),
        };
        stream
            .set_nonblocking(false)
            .map_err(|err| err.to_string())?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|err| err.to_string())?;
        match handle_request(&mut stream) {
            Ok(None) => continue,
            Ok(data) => match data {
                Some((code, state)) => {
                    if state != expected_state {
                        return Err("Invalid request, please try again".to_string());
                    }
                    respond(
                        &mut stream,
                        "200 OK",
                        "Login successful! You can close this tab.",
                    )
                    .map_err(|err| err.to_string())?;

                    return Ok(CallbackData { code, state });
                }
                _ => return Err("request failed".to_string()),
            },
            Err(err) => return Err(err.to_string()),
        }
    }
}

fn handle_request(stream: &mut TcpStream) -> Result<Option<(String, String)>, String> {
    let mut line = String::new();
    BufReader::new(&*stream)
        .read_line(&mut line)
        .map_err(|err| err.to_string())?;

    let target = line.split_whitespace().nth(1).unwrap_or_default();
    let url =
        Url::parse(&format!("http://127.0.0.1{target}")).map_err(|error| error.to_string())?;

    if url.path() != "/callback" {
        respond(stream, "404", "Not found").map_err(|err| err.to_string())?;
        return Ok(None);
    }
    let (mut code, mut state, mut error) = (None, None, None);
    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "code" => code = Some(v.into_owned()),
            "state" => state = Some(v.into_owned()),
            "error" => error = Some(v.into_owned()),
            _ => {}
        }
    }
    if let Some(e) = error {
        respond(stream, "200 OK", "Login failed. You can close this tab.")
            .map_err(|err| err.to_string())?;
        return Err(format!("Google returned error: {e}"));
    }
    match (code, state) {
        (Some(code), Some(state)) => Ok(Some((code, state))),
        _ => {
            respond(stream, "400 Bad Request", "Missing code or state")
                .map_err(|err| err.to_string())?;
            Ok(None)
        }
    }
}

fn respond(stream: &mut TcpStream, status: &str, body: &str) -> std::io::Result<()> {
    let resp = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(resp.as_bytes())
}
