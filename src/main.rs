use std::{net::{TcpListener, TcpStream}, io::{Read}, fmt, error::Error, collections::HashMap, time::Instant};
use substring::Substring;
use threadpool::ThreadPool;
use urlencoding::decode;
mod webserver;

#[derive(Debug, Clone, Copy)]
enum RequestMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    INVALID
}

impl fmt::Display for RequestMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RequestMethod::GET => write!(f, "GET"),
            RequestMethod::POST => write!(f, "POST"),
            RequestMethod::PUT => write!(f, "PUT"),
            RequestMethod::PATCH => write!(f, "PATCH"),
            RequestMethod::DELETE => write!(f, "DELETE"),
            RequestMethod::INVALID => write!(f, "INVALID"),
        }
    }
}

struct HTTPRequest {
    method: RequestMethod,
    endpoint: String,
    headers: HashMap<String, String>,
    cookies: HashMap<String, String>,
    request: HashMap<String, String>,
    body_prams: HashMap<String, String>
}

fn parse_request_type(data: &str) -> RequestMethod {
    if data == "GET" { return RequestMethod::GET; }
    if data == "POST" { return RequestMethod::POST; }
    if data == "PUT" { return RequestMethod::PUT; }
    if data == "PATCH" { return RequestMethod::PATCH; }
    if data == "DELETE" { return RequestMethod::DELETE; }
    return RequestMethod::INVALID;
}

fn handle_tcp_connection(mut socket : TcpStream) -> Result<(), Box<dyn Error>> {
    let now = Instant::now();
    let mut buffer = [0; 5120];
    socket.read(&mut buffer)?; // read the input stream

    let request_string = String::from_utf8(buffer.to_vec())?;
    let request : Vec<&str> = request_string.split("\n").collect();

    if request.len() <= 0 { return Ok(()); }
    if !request[0].contains("HTTP/1.1") { return Ok(()); } // we dont support anything else 

    let mut request_details: Vec<&str> = request[0].split(" ").collect();
    let mut headers: HashMap<String, String> = HashMap::new();
    let mut cookies: HashMap<String, String> = HashMap::new();
    let mut prams: HashMap<String, String> = HashMap::new();
    let mut body_prams: HashMap<String, String> = HashMap::new();

    for header in &request {
        let find_index = header.find(":");
        if find_index.is_none() { continue; }
        let header_end = find_index.unwrap();
        headers.insert(header.split_at(header_end).0.to_string(), header.split_at(header_end + 2).1.to_string());
    }

    if headers.contains_key("Content-Type") && headers.get("Content-Type").unwrap().contains("application/x-www-form-urlencoded") {
        let data = request[request.len() - 1].to_string();
        let pram_string_decoded = decode(&data)?;
        let raw_prams = pram_string_decoded.split("&");

        for s in raw_prams {
            let contains_key_end = s.find("=");
            if contains_key_end.is_none() {continue;} // skip broke prams
            let key_end = contains_key_end.unwrap();
            body_prams.insert(s.substring(0, key_end).to_string(), s.substring(key_end + 1, s.len()).to_string());
        }
    }

    if headers.contains_key("Cookie") {
        let raw_cookies = headers.get("Cookie").unwrap().split("; ");
        for s in raw_cookies {
            let contains_key_end = s.find("=");
            if contains_key_end.is_none() {continue;} // skip broke cookies
            let key_end = contains_key_end.unwrap();
            cookies.insert(s.substring(0, key_end).to_string(), s.substring(key_end + 1, s.len()).to_string());
        }
    }

    let first_question_exists = request_details[1].find("?");
    if first_question_exists.is_some() {
        let first_question = first_question_exists.unwrap();        
        let seperate_prams = request_details[1].split_at(first_question);
        request_details[1] = seperate_prams.0; // dont give the route handler the raw string

        let pram_string_raw = &seperate_prams.1.replace("?", "");
        let pram_string_decoded = decode(&pram_string_raw)?;
        let raw_prams = pram_string_decoded.split("&");

        for s in raw_prams {
            let contains_key_end = s.find("=");
            if contains_key_end.is_none() {continue;} // skip broke prams
            let key_end = contains_key_end.unwrap();
            prams.insert(s.substring(0, key_end).to_string(), s.substring(key_end + 1, s.len()).to_string());
        }
    }

    let mut http_request = HTTPRequest {
        method: parse_request_type(&request_details[0]),
        endpoint: request_details[1].to_string(),
        headers: headers,
        cookies: cookies,
        request: prams,
        body_prams: body_prams
    };

    webserver::handle_http_request(&mut http_request, socket)?;

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?} on {}", elapsed, http_request.endpoint);
    Ok(())
}

fn main()  {
    println!("Starting webserver");
    let listener = TcpListener::bind("0.0.0.0:80").unwrap();
    let thread_pool = ThreadPool::new(1);

    for stream in listener.incoming() {
        thread_pool.execute(|| {
            handle_tcp_connection(stream.unwrap()).unwrap();
       });
    }
}