// Uncomment this block to pass the first stage
use std::net::TcpListener;
use std::io::Write;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let response: &str = "HTTP/1.1 200 OK\r\n\r\n";
                stream.write(response.as_bytes()).unwrap();

            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
