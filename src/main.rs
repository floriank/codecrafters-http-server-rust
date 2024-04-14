use anyhow::Error;
use std::io::{Read, Write};
use std::net::TcpListener;

#[derive(Debug)]
struct HttpRequest {
    path: String,
    method: String,
    version: String,
}
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const ERROR_RESPONSE: &str = "HTTP/1.1 500 Internal Server Error\r\n\r\n";

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(_) => {
                        let req = std::str::from_utf8(&buffer).unwrap();
                        let http_request = parse_req(req).unwrap();
                        let mut resp = NOT_FOUND_RESPONSE;
                        println!("path: {:?}", http_request.path);
                        if http_request.path == "/" {
                            resp = OK_RESPONSE;
                        }

                        match stream.write(resp.as_bytes()) {
                            Ok(_) => println!("OK"),
                            Err(e) => println!("Error! ({:?})", e),
                        }
                    }
                    Err(_) => {
                        stream.write(ERROR_RESPONSE.as_bytes()).unwrap();
                    }
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn parse_req(req: &str) -> Result<HttpRequest, Error> {
    let contents: Vec<&str> = req.lines().collect();
    println!("contents: {:?}", contents);
    let mut method_header = contents[0].split_whitespace();
    let http_request = HttpRequest {
        method: String::from(method_header.next().unwrap()),
        path: String::from(method_header.next().unwrap()),
        version: String::from(method_header.next().unwrap()),
    };
    println!("http req: {:?}", http_request);
    Ok(http_request)
}
