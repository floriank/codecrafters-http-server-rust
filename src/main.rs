use anyhow::Error;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;

#[derive(Debug)]
struct HttpRequest {
    path: String,
    method: HttpMethod,
    version: String,
    user_agent: String,
}

impl HttpRequest {
    fn new(path: &str, version: &str, method: &str, agent: &str) -> Self {
        let http_method = match method {
            "GET" => HttpMethod::GET,
            _ => todo!("implement later"),
        };
        Self {
            path: path.to_string(),
            method: http_method,
            version: version.to_string(),
            user_agent: agent.to_string(),
        }
    }
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n";
const CONTENT_TYPE: &str = "Content-Type: text/plain\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const ERROR_RESPONSE: &str = "HTTP/1.1 500 Internal Server Error\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                let result = match stream.read(&mut buffer) {
                    Ok(_) => {
                        let req = std::str::from_utf8(&buffer).unwrap();
                        let http_request = parse_req(req).unwrap();
                        let resp = handle_request(&http_request);
                        let with_linebreaks = format!("{}\r\n", resp);
                        stream.write(with_linebreaks.as_bytes())
                    }
                    Err(_) => stream.write(ERROR_RESPONSE.as_bytes()),
                };

                match result {
                    Ok(_) => println!("ok"),
                    Err(e) => println!("error: {:?}", e),
                }

                stream.flush().unwrap();
            }

            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
#[derive(Debug)]
enum HttpMethod {
    GET,
}

fn echo_response(path: &str) -> String {
    let body = path.strip_prefix("/echo/").unwrap();
    let length = body.len();
    let content_length = format!("content-length: {}", length);
    format!(
        "{}{}{}\r\n\r\n{}",
        OK_RESPONSE, CONTENT_TYPE, content_length, body
    )
}

fn user_agent(agent: &str) -> String {
    let length = agent.len();
    let content_length = format!("content-length: {}", length);
    format!(
        "{}{}{}\r\n\r\n{}",
        OK_RESPONSE, CONTENT_TYPE, content_length, agent
    )
}

fn handle_request(http_request: &HttpRequest) -> String {
    let req_path = http_request.path.as_str();
    let agent = http_request.user_agent.as_str();

    match http_request.method {
        HttpMethod::GET => match req_path {
            "/" => OK_RESPONSE.to_string(),
            path if path.starts_with("/echo") => echo_response(path),
            path if path.starts_with("/user-agent") => user_agent(agent),
            _ => NOT_FOUND_RESPONSE.to_string(),
        },
    }
}
fn parse_req(req: &str) -> Result<HttpRequest, Error> {
    let contents: Vec<&str> = req.lines().collect();
    let mut method_header = contents[0].split_whitespace();
    let mut header_lines: Vec<&str> = contents.into_iter().take_while(|&s| s != "").collect();
    header_lines.remove(0);
    let mut headers = HashMap::new();
    for line in header_lines.into_iter() {
        let parts: Vec<&str> = line.split(":").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let key = parts[0].to_lowercase();
            let value = parts[1].trim().to_string();
            headers.insert(key, value);
        }
    }
    let method = method_header.next().unwrap();
    let path = method_header.next().unwrap();
    let version = method_header.next().unwrap();
    let agent = match headers.get("user-agent") {
        Some(value) => value,
        None => "",
    };
    let http_request = HttpRequest::new(&path, &version, &method, &agent);
    Ok(http_request)
}
