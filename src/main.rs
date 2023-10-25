use std::io::{Read, Write};
// Uncomment this block to pass the first stage
use std::net::TcpListener;
use regex::Regex;

const HTTP_OK: &str = "HTTP/1.1 200 OK\r\n\r\n";
const HTTP_NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\n\r\n";


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut buf: Vec<u8> = vec![];
                _stream.read_to_end(&mut buf).unwrap();
                let req = String::from_utf8_lossy(&buf);
                let header = req.lines().next().unwrap();
                let path = parse_http_header(header).unwrap();
                if path == "/" {
                    _stream.write_all(HTTP_OK.as_bytes()).unwrap();
                } else {
                    _stream.write_all(HTTP_NOT_FOUND.as_bytes()).unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn parse_http_header(header: &str) -> Option<String> {
    let re = Regex::new(r"GET (.*) HTTP/1.1").unwrap();
    let cap = re.captures(header)?;
    let path = cap.get(1)?;
    Some(path.as_str().to_string())
}
