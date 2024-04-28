use anyhow::Error;
use clap::Parser;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::main;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[derive(Debug)]
enum HttpMethod {
    GET,
    POST,
}

#[derive(Debug)]
struct HttpRequest {
    path: String,
    method: HttpMethod,
    version: String,
    user_agent: String,
    body: String,
}

impl HttpRequest {
    fn new(path: &str, version: &str, method: &str, agent: &str, body: &str) -> Self {
        let http_method = match method {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            _ => todo!("implement later"),
        };
        Self {
            path: path.to_string(),
            method: http_method,
            version: version.to_string(),
            user_agent: agent.to_string(),
            body: body.to_string(),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    directory: Option<PathBuf>,
}

const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n";
const CREATED_RESPONSE: &str = "HTTP/1.1 201 Created\r\n";
const CONTENT_TYPE: &str = "Content-Type: text/plain\r\n";
const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const ERROR_RESPONSE: &str = "HTTP/1.1 500 Internal Server Error\r\n\r\n";
const OCTET_STREAM: &str = "Content-Type: application/octet-stream\r\n";

#[main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = SocketAddr::from(([127, 0, 0, 1], 4221));
    let listener = TcpListener::bind(&address).await?;

    println!("Listening on address https://{}", address);

    loop {
        let (mut stream, _) = listener.accept().await?;
        let args = Arc::new(Args::parse());
        tokio::spawn(async move {
            let mut buffer = [0; 1024];
            let args = Arc::clone(&args);
            match stream.read(&mut buffer).await {
                Ok(_) => {
                    let req = std::str::from_utf8(&buffer).unwrap();
                    let http_request = parse_req(req).unwrap();
                    let resp = handle_request(&http_request, &args);
                    let with_linebreaks = format!("{}\r\n", resp);
                    let _ = stream.write(with_linebreaks.as_bytes()).await;
                }
                Err(_) => {
                    let _ = stream.write(ERROR_RESPONSE.as_bytes()).await;
                }
            }
        });
    }
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

fn handle_request(http_request: &HttpRequest, args: &Arc<Args>) -> String {
    let req_path = http_request.path.as_str();
    let agent = http_request.user_agent.as_str();

    match http_request.method {
        HttpMethod::GET => match req_path {
            "/" => OK_RESPONSE.to_string(),
            path if path.starts_with("/echo") => echo_response(path),
            path if path.starts_with("/user-agent") => user_agent(agent),
            path if path.starts_with("/files/") => send_file(path, args),
            _ => NOT_FOUND_RESPONSE.to_string(),
        },
        HttpMethod::POST => match req_path {
            path if path.starts_with("/files/") => save_file(&http_request.body, path, args),
            _ => NOT_FOUND_RESPONSE.to_string(),
        },
    }
}

fn save_file(body: &str, req_path: &str, args: &Arc<Args>) -> String {
    let mut path = PathBuf::new();
    path.push(args.directory.as_ref().unwrap_or(&PathBuf::from(".")));
    path.push(&req_path[7..]);
    match File::create(path) {
        Ok(mut file) => {
            let _ = file.write_all(body.as_bytes());
            println!("{}", body);
            let content_length = format!("content-length: {}", body.len());
            format!("{}{}{}", CREATED_RESPONSE, OCTET_STREAM, content_length)
        }
        Err(_) => ERROR_RESPONSE.to_string(),
    }
}
fn send_file(req_path: &str, args: &Arc<Args>) -> String {
    let mut path = PathBuf::new();
    path.push(args.directory.as_ref().unwrap_or(&PathBuf::from(".")));
    path.push(&req_path[7..]);
    match File::open(path) {
        Ok(mut file) => {
            let mut content = String::new();
            match file.read_to_string(&mut content) {
                Err(_) => NOT_FOUND_RESPONSE.to_string(),
                Ok(_) => {
                    let content_length = format!("content-length: {}", content.len());
                    format!(
                        "{}{}{}\r\n\r\n{}",
                        OK_RESPONSE, OCTET_STREAM, content_length, content
                    )
                }
            }
        }
        Err(_) => NOT_FOUND_RESPONSE.to_string(),
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
    let body_start = req.find("\r\n\r\n").unwrap();
    let body = &req[(body_start + 4)..];

    let http_request = HttpRequest::new(&path, &version, &method, &agent, &body);
    Ok(http_request)
}
