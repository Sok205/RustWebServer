use RustWebServer::ThreadPool;
use std::{
    fs,
    io::{BufReader, prelude::*},
    net::{TcpListener, TcpStream},
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
    thread,
    time::Duration,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    let active_connections = Arc::new(AtomicUsize::new(0));
    let connection_counter = Arc::new(AtomicUsize::new(1));

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let active = Arc::clone(&active_connections);
        let counter = Arc::clone(&connection_counter);

        pool.execute(move || {
            let conn_num = counter.fetch_add(1, Ordering::SeqCst);
            let current = active.fetch_add(1, Ordering::SeqCst) + 1;
            handle_connection(stream, conn_num, current);
            active.fetch_sub(1, Ordering::SeqCst);
        });
    }
}

fn handle_connection(mut stream: TcpStream, conn_num: usize, active: usize) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        }
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };

    let mut contents = fs::read_to_string(filename).unwrap();
    contents = contents.replace("{{CONN_NUM}}", &conn_num.to_string());
    contents = contents.replace("{{ACTIVE_CONN}}", &active.to_string());

    let length = contents.len();
    let response =
        format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");

    stream.write_all(response.as_bytes()).unwrap();
}