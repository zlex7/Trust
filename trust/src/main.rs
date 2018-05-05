#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
extern crate regex;
mod trust;
use trust::Framework;
use trust::Request;
use trust::UrlPart;
mod jinja;
use jinja::*;
use std::collections::HashMap;

// use trust::Route;


fn abc(req: Request) -> String{
	return "abc".to_string();
}

fn root(req: Request) -> String{
	return "root".to_string();
}

fn i32_test(request: Request) -> String{
	return match request.values.get("test").unwrap(){
		&UrlPart::PARAM(ref x, ref name, ..) => name.to_string(),
		_ => String::from("this didn't work")
	}
}

fn jinja_test(request: Request) -> String {
	return render_template("templates/404.html",hashmap!{String::from("url") => String::from("jinjatest (this actually worked lol)")});
}

fn gina(request: Request) -> String {
	let age = match request.values.get("age").unwrap(){
		&UrlPart::PARAM(ref x, ref name, ..) => name.to_string(),
		_ => String::from("this didn't work")
	};
	return render_template("templates/gina.html",hashmap!{String::from("age") => age});
}

fn main(){
	let mut f = Framework::new();
	f.add("/", "GET", root)
	 .add("/abc", "GET", abc)
	 .add("/super/<test: int>", "GET", i32_test)
	 .add("/jinjatemplate","GET",jinja_test)
	 .add("/gina/<age: int>","GET",gina);
	println!("{:?}",f.getRouteString());
	f.run();
	// let mut hashmap : HashMap<String,String>= HashMap::new();
	// hashmap.insert("url".to_string(),"abc".to_string());
	// println!("{}",render_template("templates/404.html",hashmap));
}
