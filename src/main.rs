use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use regex::Regex;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::fs::File;
use std::env;

const HTTP_OK: &str = "HTTP/1.1 200 OK\r\n\r\n";
const HTTP_NOT_FOUND: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            process(socket).await;
        });
    }
}

async fn process(stream: TcpStream) {
    let (read, write) = stream.into_split();
    let mut reader = BufReader::new(read);
    let mut writer = BufWriter::new(write);
    let mut header = String::new();
    reader.read_line(&mut header).await.unwrap();
    if let Some(path) = parse_http_line(header.as_str(), r"GET (.*) HTTP/1.1") {
        handle_get_request(path.as_str(), &mut reader, &mut writer).await;
    } else if let Some(path) = parse_http_line(header.as_str(), r"POST (.*) HTTP/1.1") {
        handle_post_request(path.as_str(), &mut reader, &mut writer).await;
    } else {
        println!("Not implemented");
    }
}

async fn handle_get_request(path: &str, reader: &mut BufReader<OwnedReadHalf>, writer: &mut BufWriter<OwnedWriteHalf>) {
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
        },
        _ if path.starts_with("/files") => {
            let args: Vec<String> = env::args().collect();
            let file_name = &path["/files/".len()..];
            let file_path = format!("{}{}", args.get(2).unwrap(), file_name);
            println!("File path {}", file_path);
            if let Ok(mut file) = File::open(file_path).await {
                let mut content = String::new();
                file.read_to_string(&mut content).await.unwrap();
                let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", content.len(), content);
                writer.write_all(resp.as_bytes()).await.unwrap();
            } else {
                writer.write_all(HTTP_NOT_FOUND.as_bytes()).await.unwrap();
            }
        },
        _ => {
            writer.write_all(HTTP_NOT_FOUND.as_bytes()).await.unwrap();
        }
    }
    writer.flush().await.unwrap();
}

async fn handle_post_request(path: &str, reader: &mut BufReader<OwnedReadHalf>, writer: &mut BufWriter<OwnedWriteHalf>) {
    match path {
        _ if path.starts_with("/files") => {
            let mut line = String::new();
            let content_length;
            loop {
                line.clear();
                reader.read_line(&mut line).await.unwrap();
                if line.starts_with("Content-Length") {
                    content_length = parse_http_line(line.as_str(), r"Content-Length: (.*)\n").unwrap().parse().unwrap();
                    line.clear();
                    reader.read_line(&mut line).await.unwrap(); // This is for content type reading
                    break;
                }
            }
            let mut buf = [0u8; 2]; // That is for CRLF
            reader.read_exact(&mut buf).await.unwrap();

            let mut buf = vec![0u8; content_length];
            reader.read_exact(&mut buf).await.unwrap();
            let body = String::from_utf8_lossy(buf.as_slice()).to_string();

            let args: Vec<String> = env::args().collect();
            let file_name = &path["/files/".len()..];
            let file_path = format!("{}{}", args.get(2).unwrap(), file_name);

            let mut file = File::create(file_path).await.unwrap();

            file.write_all(body.as_bytes()).await.unwrap();

            writer.write_all("HTTP/1.1 201 OK\r\n\r\n".as_bytes()).await.unwrap();
        },
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
