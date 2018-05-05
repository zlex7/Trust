#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
mod trust;
use trust::Framework;
use trust::Request;
use trust::UrlPart;
// use trust::Route;


fn abc(req: Request) -> String{
	return "abc".to_string();
}

fn root(req: Request) -> String{
	return "root".to_string();
}

fn i32test(request: Request) -> String{
	return match request.values.get("test").unwrap(){
		&UrlPart::PARAM(ref x, ref name, ..) => name.to_string(),
		_ => String::from("this didn't work")
	}
}

fn main(){
	let mut f = Framework::new();
	f.add("/", "GET", root)
	 .add("/abc", "GET", abc)
	 .add("/super/<test: int>", "GET", i32test);
	println!("{:?}",f.getRouteString());
	f.run();
}
