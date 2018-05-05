use std::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::LinkedList;
use std::hash::Hash;
use std::hash::Hasher;
use std::str;
use std::io::{Write, Read, BufRead, Cursor};
use std::thread;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Mutex;
use std::sync::Arc;

//these macros must be defined in main.rs
//#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate maplit;

lazy_static! {
    static ref default_headers: HashMap<String, String> = hashmap!{String::from("Content") => String::from("text/html; charset=UTF-8"), String::from("Connection") => String::from("close")};
}

lazy_static! {
    static ref accepted_methods: HashSet<String> = hashset!{String::from("GET"), String::from("POST"), String::from("PUT"), String::from("DELETE")};
}

fn handle_connection(mut route: Option<Route>, request: Result<Request,Error>, mut stream: TcpStream){
    match request {
        Ok(mut request) => {
            println!("url: {:?}", request.url);
            let response = match route {
                Some(ref mut route) => {
                    request.values = request.url.get_param_hashmap(&route.url);
                    println!("values: {:?}", request.values);
                    // let response_code = match {
                    //     _ => 200
                    // };
                    let response_code = 200;
                    Response::new(response_code, (route.handler)(request))
                }
                None => Response::new(404, "".to_string())
            };
            println!("decoded request!");
            // println!("routes: {:?}", routes);
            stream.write(&response.to_http());
        }
        Err(e) => {
            let response = Response::new(e as i32, "".to_string());
            stream.write(&response.to_http());
        }
    }
}

pub struct Framework {
    routes: HashMap<Url, Route>,
    pool: ThreadPool
}

impl Framework{
    pub fn new() -> Framework{
        Framework{
            routes: HashMap::new(),
            pool: ThreadPool::new(1)
        }
    }

    pub fn add(&mut self, path: &str, request_type: &str, handler: fn(Request)->String) -> &mut Framework{
        let url = Url::from_keyed(&path.to_string());
        println!("{:?}",url);
        let route = Route{url: url.clone(), handler: handler, request_type: request_type.to_string()};
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
                    let mut request_text: [u8; 10000] = [0 ; 10000];
            	    //TODO: buffering for large requests w/ data
            	    println!("reading raw request into buffer");
            	    stream.read(&mut request_text);
            	    println!("decoding request...");
            	    let mut request = Request::parse_request(str::from_utf8(&request_text).unwrap().to_string());
                    let t = (self.routes.get(&(request.clone()).unwrap().url));
                    println!("url = {:?}",t);
                    self.pool.add_job(handle_connection, Option::Some(t.unwrap().clone()), stream, request);
				}
				Err(e) => {
					println!("{:?}",e);
				}
			}
		}
	}

    pub fn getRouteString(&self) -> String {
        return format!("{:?}", self.routes);
    }


    // let response = b"HTTP/1.1 200 OK\n Content-Type: text/html; charset=UTF-8\n Connection: close\n\n <html><head></head><body>hello</body></html>";
}

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
enum UrlParamType {
    INT
}

#[derive(Eq, Clone, Debug)]
pub enum UrlPart {
    WORD(String),
    PARAM(String,String,UrlParamType)
}

impl ::std::hash::Hash for UrlPart {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            &UrlPart::WORD(ref word) => {
                word.hash(state);
            },
            &UrlPart::PARAM(ref name, ref value, ref param_type) => {
                param_type.hash(state);
            }
        }
    }
}

impl PartialEq for UrlPart {
    fn eq(&self, other: &UrlPart) -> bool {
        match self {
            &UrlPart::WORD(ref word) => {
                match other {
                    &UrlPart::WORD(ref word2) => {
                        return word == word2
                    }
                    _ => return false
                }
            },
            &UrlPart::PARAM(ref name, ref value, ref param_type) => {
                match other {
                    &UrlPart::PARAM(ref name2,ref value2,ref param_type2) => {
                        return param_type == param_type2
                    }
                    _ => return false
                }
            }
        }
    }
}

// impl Eq for UrlPart {}

#[derive(Eq, Clone, Debug)]
struct Url {
    parts: Vec<UrlPart>,
}

impl ::std::hash::Hash for Url {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.parts.hash(state);
    }
}

impl PartialEq for Url {
    fn eq(&self, other: &Url) -> bool {
        return self.parts == other.parts;
    }
}

impl Url {
    pub fn get_param_hashmap(&mut self, other: &Url) -> HashMap<String,UrlPart>{
        let mut params : HashMap<String,UrlPart> = HashMap::new();
        let mut ind = 0;
        while ind < self.parts.len() {
            match &self.parts[ind] {
                &UrlPart::PARAM(ref name,ref value,ref param_type) => {
                    match &other.parts[ind] {
                        &UrlPart::PARAM(ref name2,ref value2,ref param_type2) => {
                                let new_part = UrlPart::PARAM(name2.clone(),value.clone(),param_type2.clone());
                                params.insert(name2.clone(),new_part);

                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            ind += 1;
        }
        return params;
    }

    fn from_filled(url: &str) -> Url {
        let url_parts_raw: Vec<&str> = url.split("/").collect();
        println!("{:?}",url_parts_raw);
        let mut url_parts_processed: Vec<UrlPart> = Vec::new();
        let params : HashMap<String,UrlPart> = HashMap::new();
        let mut ind = 1;
        while ind < url_parts_raw.len() {
            if url_parts_raw[ind].parse::<i64>().is_ok(){
                let param_value = url_parts_raw[ind];
                url_parts_processed.push(UrlPart::PARAM(String::from(""), param_value.to_string(), UrlParamType::INT));
            }
            else {
                url_parts_processed.push(UrlPart::WORD(String::from(url_parts_raw[ind])));
            }
            ind += 1;
        }
        return Url {
            parts: url_parts_processed,
        }
    }

    fn from_keyed(url: &str) -> Url {
        let url_parts_raw: Vec<&str> = url.split("/").collect();
        println!("{:?}",url_parts_raw);
        let mut url_parts_processed: Vec<UrlPart> = Vec::new();
        let mut ind = 1;
        while ind < url_parts_raw.len() {
            if url_parts_raw[ind].starts_with("<") && url_parts_raw[ind].ends_with(">") {
                let param_parts : Vec<&str> = url_parts_raw[ind].get(1..url_parts_raw[ind].len()).unwrap().split(":").collect();
                let param_type = param_parts[1];
                let param_name = param_parts[0];
                url_parts_processed.push(UrlPart::PARAM(String::from(param_name),"".to_string(), match param_type {
                    "int" => UrlParamType::INT,
                    _ => UrlParamType::INT
                }));
            }
            else {
                url_parts_processed.push(UrlPart::WORD(String::from(url_parts_raw[ind])));
            }
            ind += 1;
        }
        return Url {
            parts: url_parts_processed,
        }
    }
}

#[derive(Debug, Clone)]
struct Route {
    url: Url,
    handler: fn(Request) -> String,
    request_type: String
}

#[derive(Debug, Clone)]
pub struct Request {
	url: Url,
    method: String,
    headers: HashMap<String,String>,
    content: String,
    pub values: HashMap<String, UrlPart>
}

#[derive(Debug, Clone, Copy)]
enum Error {
    BadRequestError = 400,
    ForbiddenError = 403,
    UrlNotFoundError = 404,
    MethodNotAllowedError = 405,
    ServerError = 500,
}

impl Request {
    fn new(url: Url, method : String, headers: HashMap<String,String>, content: String, params: HashMap<String,UrlPart>) -> Request {
        Request {
			url: url,
            method: method,
            headers: headers,
            content: content,
            values: params
        }
    }

    fn parse_request(req: String) -> Result<Request, Error> {
        let mut cursor = Cursor::new(req.as_bytes());
        let mut method: Vec<u8> = Vec::new();
        cursor.read_until(' ' as u8, &mut method);
        let mut path: Vec<u8> = Vec::new();
        cursor.read_until(' ' as u8, &mut path);

        let mut version: String = String::new();
        cursor.read_line(&mut version);

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
        let method_string = str::from_utf8(&method).expect("method error").trim().to_string();
        let content_string: String = content.trim().to_string();
		let path_string = str::from_utf8(&path).expect("path error").trim().to_string();

        if !accepted_methods.contains(&method_string) {
            return Result::Err(Error::UrlNotFoundError);
        }
        //println!("method: {:?}, headers: {:?}, content: {}", method_string, headers, content_string);
        let request = Request {
			url: Url::from_filled(&path_string),
            method: method_string,
            headers: headers,
            content: content_string,
            values: HashMap::new()
        };
        return Result::Ok(request);
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
        if self.response_code == 404 {
            let mut file = File::open("templates/404.html").unwrap();
            let mut contents = String::new();

            file.read_to_string(&mut contents).unwrap();
            http_response.push_str(&contents.to_string());
        }

        http_response.push_str(&self.content);
        return http_response.into_bytes();
    }
}

struct ThreadPool
{
    free_workers: Vec<Worker>,
    used_workers: Vec<Worker>,
    jobs: Arc<Mutex<LinkedList<Job>>>
}

impl ThreadPool{
    fn new(size: i32) -> ThreadPool{
        let mut t = ThreadPool{
            free_workers: Vec::with_capacity(size as usize),
            used_workers: Vec::with_capacity(size as usize),
            jobs: Arc::new(Mutex::new(LinkedList::new()))
        };

        for i in 0..size {
            t.free_workers.push(Worker::new(i));
            t.free_workers[i as usize].start(Arc::clone(&t.jobs));
        }

        return t;
    }

    fn add_job(&mut self, handler: fn(Option<Route>,Result<Request, Error>,TcpStream) -> (), route: Option<Route>, stream: TcpStream, request: Result<Request, Error>){
        let job = Job::new(handler, route, request, stream);
        self.jobs.lock().unwrap().push_back(job);
    }
}

struct Worker {
    id: i32
}

impl Worker {
    fn new(id: i32) -> Worker {
        Worker {
            id: id
        }
    }

    fn start(&self, jobs: Arc<Mutex<LinkedList<Job>>>){
        println!("starting thread, worker id = {}",self.id);
        let job_box = Box::new(jobs);
        let join_handle = thread::spawn(move ||{
            loop{
                println!("waiting for job... here's the job queue: ");
                while job_box.lock().unwrap().is_empty(){ }
                let mut job_queue = job_box.lock().unwrap();
                let job = job_queue.pop_front();
                println!("we've gotten a job: {:?}", job);
                drop(job_queue);
                match job {
                    Some(job) => (job.handler)(job.route, job.request, job.stream),
                    None => {}
                };
                // if jobs.lock().unwrap().len() > 0{
                // let vec = match jobs.lock().unwrap().pop_front() {
                    // Some(job) => job,
                    // None => {}
                // };
                //pop off a job if its is availible otherwise block
                //call the job.handler(request)
            }
        });
    }
}

#[derive(Debug)]
struct Job {
    handler: fn(Option<Route>,Result<Request, Error>,TcpStream) -> (),
    route: Option<Route>,
    request: Result<Request,Error>,
    stream: TcpStream,
    // join_handle: thread::JoinHandle<fn()>
}

impl Job {
    fn new(handler: fn(Option<Route>,Result<Request, Error>,TcpStream) -> (), route: Option<Route>, request: Result<Request,Error>, stream: TcpStream) -> Job{
        return Job {
            handler: handler,
            request: request,
            route: route,
            stream: stream
        }
    }
}
