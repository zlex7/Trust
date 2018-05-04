use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::str;
use std::io::{Write, Read, BufRead, Cursor};

//these macros must be defined in main.rs
//#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate maplit;

// struct ThreadPool{
//     workers: Vec<Worker>
// }

pub struct Framework {
    routes: HashMap<Url, Route>
}

impl Framework{
    pub fn new() -> Framework{
        Framework{
            routes: HashMap::new()
        }
    }
    pub fn add(&mut self, path: String, request_type: String, handler: fn(Request)->String) -> &mut Framework{
        let url = Url::from_raw(&path);
        let route = Route{url: url.clone(), handler: handler, request_type: request_type};
        self.routes.insert(url, route);
        self
    }
	pub fn run(&mut self){
		let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
		println!("serving requests on {}", "127.0.0.1:8080");
		for stream in listener.incoming() {
			match stream {
				Ok(mut stream) => {
					println!("new client!");
					self.handle_connection(stream);
				}
				Err(e) => {
					println!("{:?}",e);
				}
			}
		}
	}

	fn handle_connection(&mut self, mut stream: TcpStream){
	    // let buffer = IoBuf::new(stream);
   		let mut request_text : [u8; 10000] = [0 ; 10000];
	   	// let read_size = buffer.read().expect("No response to read");
	    //TODO: buffering for large requests w/ data
	    println!("reading raw request into buffer");
	    stream.read(&mut request_text);
	    let response_code = 200;
	    println!("decoding request...");
	    let request = Request::parse_request(str::from_utf8(&request_text).unwrap().to_string());
		println!("decoded request!");
		let response = Response::new(response_code, (self.routes.get(&request.url).unwrap().handler)(request));
    	stream.write(&response.to_http());
    	// let response = b"HTTP/1.1 200 OK\n Content-Type: text/html; charset=UTF-8\n Connection: close\n\n <html><head></head><body>hello</body></html>";
	}
}

#[derive(Hash)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]

enum UrlParamType {
    INT,
    UINT,
    STRING
}

#[derive(Hash)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]

enum UrlPart {
    WORD(String),
    PARAM(String,UrlParamType)
}

// struct UrlParam {
//     name: &str,
//     type: UrlParamType
// }

#[derive(Hash)]
#[derive(Eq)]
#[derive(PartialEq)]
#[derive(Clone)]
struct Url {
    url_parts: Vec<UrlPart>
}

impl Url {
    fn from_raw(url: &str) -> Url {
        let url_parts_raw: Vec<&str> = url.split("/").collect();
        let mut url_parts_processed: Vec<UrlPart> = Vec::new();
        let mut ind = 0;
        while ind < url_parts_raw.len() {
            if url_parts_raw[ind].starts_with("<") && url_parts_raw[ind].ends_with(">") {
                let param_parts : Vec<&str> = url_parts_raw[ind].get(1..url_parts_raw[ind].len()).unwrap().split(":").collect();
                let param_type = param_parts[1];
                url_parts_processed.push(UrlPart::PARAM(String::from(param_parts[0]),match param_type {
                    "int" => UrlParamType::INT,
                    "uint" => UrlParamType::UINT,
                    "str" => UrlParamType::STRING,
                    _ => UrlParamType::STRING
                }));
            }
            ind += 1;
        }
        return Url {
            url_parts: url_parts_processed
        }
    }
}

struct Route {
    url: Url,
    handler: fn(Request) -> String,
    request_type: String
}

lazy_static! {
    static ref default_headers: HashMap<String, String> = hashmap!{String::from("Content") => String::from("text/html; charset=UTF-8"), String::from("Connection") => String::from("close")};
}

pub struct Request {
	url: Url, 
    method: String,
    headers: HashMap<String,String>,
    content: String
}

impl Request {
    fn new(url: Url, method : String, headers: HashMap<String,String>, content: String) -> Request {
        Request {
			url: url, 
            method: method,
            headers: headers,
            content: content
        }
    }

    fn parse_request(req: String) -> Request{
        let mut cursor = Cursor::new(req.as_bytes());
        let mut method: Vec<u8> = Vec::new();
        cursor.read_until(' ' as u8, &mut method);
        let mut path: Vec<u8> = Vec::new();
        cursor.read_until(' ' as u8, &mut path);
        let mut rand: String = String::new();
        cursor.read_line(&mut rand);
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut rest_of_file: Vec<u8> = Vec::new();
        cursor.read_until('\0' as u8, &mut rest_of_file);
        let rof_split: Vec<&str> = (str::from_utf8(&rest_of_file)).expect("oh").splitn(2, "\r\n\r\n").collect();
        let headers_vec = rof_split[0];

        //println!("rof: {:?}", rof_split);
        let cursor_headers = Cursor::new(headers_vec.as_bytes());

        for line in cursor_headers.lines(){
            let temp : String =  line.unwrap();
            let pairs: Vec<&str> = temp.split(": ").collect();
          	//println!("{} {:?}", temp, pairs);
			if pairs.len() == 2{
            	headers.insert(pairs[0].to_string(), pairs[1].to_string());
        	}
		}
        let content = rof_split[1];
        let method_string = str::from_utf8(&method).expect("method error").to_string();
        let content_string: String = content.to_string();
		let path_string = str::from_utf8(&path).expect("path error").to_string();
        //println!("method: {:?}, headers: {:?}, content: {}", method_string, headers, content_string);
        Request {
			url: Url::from_raw(&path_string),
            method: method_string,
            headers: headers,
            content: content_string
        }
    }
}

struct Response{
    headers: HashMap<String,String>,
    content: String,
    response_code: i32
}

impl Response{
    pub fn new(response_code: i32, content: String) -> Response {
        return Response{
            headers: default_headers.clone(),
            response_code: response_code,
            content: content
        }
    }
    //TODO: unchanged hashmap means reparsing
    fn to_http(&self) -> Vec<u8> {
        let mut http_response = String::from(format!("HTTP/1.1 {}\n", self.response_code));
        println!("{:?}", self.headers);
        for (key, value) in self.headers.iter() {
            http_response.push_str(key);
            http_response.push_str(": ");
            http_response.push_str(value);
            http_response.push('\n');
        }
        println!("{}", http_response);
        http_response.push_str("\n\n");
        http_response.push_str(&self.content);
        return http_response.into_bytes();
    }
}


// fn make_default_headers() {
//
//     default_headers = hashmap!{"Content" => "text/html; charset=UTF-8", "Connection" => "close"};
// }

