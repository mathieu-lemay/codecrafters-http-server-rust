use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Result, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::thread;
use std::{env, str};

fn main() {
    let args: Vec<String> = env::args().collect();
    let directory = if args.len() == 3 && args[1] == "--directory" {
        PathBuf::from(&args[2])
    } else {
        PathBuf::from(".")
    };

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let directory = directory.clone();
                thread::spawn(|| handle(_stream, directory));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle(mut stream: TcpStream, directory: PathBuf) {
    let addr = stream.peer_addr().expect("Unable to get peer_addr");
    println!("accepted new connection from {:?}", addr);

    let req = match parse_request(&mut stream) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading stream: {:?}", e);
            return;
        }
    };

    let resp = if req.path == "/" {
        get_empty_resp("200 OK")
    } else if req.path.starts_with("/echo/") {
        get_echo_resp(&req)
    } else if req.path == "/user-agent" {
        get_user_agent_resp(&req)
    } else if req.path.starts_with("/files") {
        match req.method.as_str() {
            "GET" => get_file_resp(&req, &directory),
            "POST" => post_file_resp(&req, &directory),
            _ => get_empty_resp("405 Method Not Allowed"),
        }
    } else {
        get_empty_resp("404 Not Found")
    };

    match stream.write(resp.as_bytes()) {
        Ok(n) => println!("Successfully wrote {} bytes to stream", n),
        Err(e) => eprintln!("Error writing to stream: {:?}", e),
    }

    stream
        .shutdown(Shutdown::Both)
        .expect("Error closing stream");
}

#[derive(Debug)]
#[allow(dead_code)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

fn parse_request(stream: &mut TcpStream) -> Result<Request> {
    let mut buf = [0; 2048];

    let n = stream.read(&mut buf)?;
    println!("Successfully read {} bytes from stream", n);

    let raw_request = str::from_utf8(&buf[..n]).expect("Invalid http request");
    println!("\nRaw request:\n======\n{}\n======\n", raw_request);

    let mut data = raw_request.split("\r\n");

    let start_line = data.next().expect("No start line");

    let mut headers = HashMap::new();

    loop {
        let header = data.next().expect("request truncated");
        if header.is_empty() {
            break;
        }

        if let [k, v] = header.splitn(2, ": ").collect::<Vec<&str>>()[..] {
            headers.insert(k.to_lowercase(), v.to_string());
        } else {
            eprintln!("Error parsing header: {}", header);
        }
    }

    let body = data.next().expect("request truncated");

    let parts: Vec<&str> = start_line.split(' ').collect();

    let request = Request {
        method: String::from(parts[0]),
        path: String::from(parts[1]),
        headers,
        body: String::from(body),
    };
    println!("Request: {:#?}", request);

    Ok(request)
}

fn get_empty_resp(status: &str) -> String {
    format!("HTTP/1.1 {}\r\n\r\n", status)
}

fn get_echo_resp(request: &Request) -> String {
    let value = &request.path.strip_prefix("/echo/").unwrap();
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
        value.len(),
        value
    )
}

fn get_user_agent_resp(request: &Request) -> String {
    if let Some(agent) = request.headers.get("user-agent") {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{}",
            agent.len(),
            agent
        )
    } else {
        get_empty_resp("404 Not Found")
    }
}

fn get_file_resp(request: &Request, directory: &PathBuf) -> String {
    let mut path = directory.clone();
    path.push(request.path.strip_prefix("/files/").unwrap());

    if !path.exists() {
        return get_empty_resp("404 Not Found");
    }

    let mut buf: Vec<u8> = Vec::new();

    let mut reader = BufReader::new(File::open(&path).unwrap());
    reader.read_to_end(&mut buf).expect("error reading file");

    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\n\r\n{}",
        buf.len(),
        String::from_utf8(buf).unwrap()
    )
}

fn post_file_resp(request: &Request, directory: &PathBuf) -> String {
    let mut path = directory.clone();
    path.push(request.path.strip_prefix("/files/").unwrap());

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .expect("error opening file for writing");

    let mut writer = BufWriter::new(file);
    writer
        .write_all(&request.body.as_bytes())
        .expect("Error writing file");

    "HTTP/1.1 201 Created\r\n\r\n".to_string()
}
