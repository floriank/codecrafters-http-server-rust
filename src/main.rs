use anyhow::Error;
use std::io::{Read, Write};
use std::net::TcpListener;

#[derive(Debug)]
struct HttpRequest {
    path: String,
    method: HttpMethod,
    version: String,
}

impl HttpRequest {
    fn new(path: &str, version: &str, method: &str) -> Self {
        let http_method = match method {
            "GET" => HttpMethod::GET,
            _ => todo!("implement later"),
        };
        Self {
            path: path.to_string(),
            method: http_method,
            version: version.to_string(),
        }
    }
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n";
const CONTENT_TYPE: &str = "Content-Type: text/plain\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n";
const ERROR_RESPONSE: &str = "HTTP/1.1 500 Internal Server Error\r\n";

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
                        println!("{:?}", resp);
                        stream.write(resp.as_bytes())
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
    let values: Vec<&str> = path.split("/").collect();
    println!("{:?}", values);
    let body = values[2];
    let length = body.len();
    let content_length = format!("Content-Length: {}", length);
    format!("{}{}{}\r\n\r\n{}", OK_RESPONSE, CONTENT_TYPE, content_length, body)
}

fn handle_request(http_request: &HttpRequest) -> String {
    let req_path = http_request.path.as_str();
    match http_request.method {
        HttpMethod::GET => match req_path {
            "/" => OK_RESPONSE.to_string(),
            path if path.starts_with("/echo") => echo_response(path),
            _ => NOT_FOUND_RESPONSE.to_string(),
        },
    }
}

fn parse_req(req: &str) -> Result<HttpRequest, Error> {
    let contents: Vec<&str> = req.lines().collect();
    println!("contents: {:?}", contents);
    let mut method_header = contents[0].split_whitespace();
    // TODO clean this up omg - this depends on the field order at the moment
    let method = method_header.next().unwrap();
    let path = method_header.next().unwrap();
    let version = method_header.next().unwrap();
    let http_request = HttpRequest::new(&path, &version, &method);
    println!("http req: {:?}", http_request);
    Ok(http_request)
}
