//use std::io::{BufRead, BufReader, Write};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

// Uncomment this block to pass the first stage
// use std::net::{TcpListener, TcpStream};
use regex::Regex;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};

const HTTP_OK: &str = "HTTP/1.1 200 OK\r\n\r\n";
const HTTP_NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

//     for stream in listener.incoming() {
//         match stream {
//             Ok(mut _stream) => {
//                 let mut reader: BufReader<TcpStream> = BufReader::new(_stream.try_clone().unwrap());
//                 let mut header = String::new();
//                 reader.read_line(&mut header).unwrap();
//                 let path = parse_http_header(header.as_str()).unwrap();
//                 if path.starts_with("/echo/") {
//                     let res = &path.as_str()["/echo/".len()..];
//                     let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", res.len(), res);
//                     _stream.write_all(resp.as_bytes()).unwrap();
//                 } else if path == "/" {
//                     _stream.write_all(HTTP_OK.as_bytes()).unwrap();
//                 } else if path == "/user-agent" {
//                     let mut line = String::new();
//                     while let Ok(_) = reader.read_line(&mut line) {
//                         if let Some(agent) = parse_user_agent(line.as_str()) {
//                             let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", agent.len(), agent);
//                             _stream.write_all(resp.as_bytes()).unwrap();
//                             break;
//                         }
//                         line.clear();
//                     }
//                 }
//                 else {
//                     _stream.write_all(HTTP_NOT_FOUND.as_bytes()).unwrap();
//                 }
//             }
//             Err(e) => {
//                 println!("error: {}", e);
//             }
//         }
//     }
// }

async fn process(stream: TcpStream) {
    let (read, write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let mut writer = BufWriter::new(write);
    let mut header = String::new();
    reader.read_line(&mut header).await.unwrap();
    let path = parse_http_line(header.as_str(), r"GET (.*) HTTP/1.1").unwrap();
    handle_request(path.as_str(), &mut reader, &mut writer).await;
}

async fn handle_request(path: &str, reader: &mut BufReader<OwnedReadHalf>, writer: &mut BufWriter<OwnedWriteHalf>) {
    match path {
        "/" => {
            writer.write_all(HTTP_OK.as_bytes()).await.unwrap();
        },
        "/user-agent" => {
            let mut line = String::new();
            while let Ok(_) = reader.read_line(&mut line).await {
                if let Some(agent) = parse_http_line(line.as_str(), r"User-Agent: (.*)\n") {
                    let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", agent.len(), agent);
                    writer.write_all(resp.as_bytes()).await.unwrap();
                    break;
                }
                line.clear();
            }
        },
        _ if path.starts_with("/echo") => {
             {
                let res = &path["/echo/".len()..];
                let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", res.len(), res);
                writer.write_all(resp.as_bytes()).await.unwrap();
            }
        }
        _ => {
            writer.write_all(HTTP_NOT_FOUND.as_bytes()).await.unwrap();
        }
    }
    writer.flush().await.unwrap();
}

fn parse_http_line(line: &str, re: &str) -> Option<String> {
    let re = Regex::new(re).unwrap();
    let cap = re.captures(line)?;
    let path = cap.get(1)?;
    Some(path.as_str().trim().to_string())
}
