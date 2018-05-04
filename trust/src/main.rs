#[macro_use] extern crate lazy_static;
#[macro_use] extern crate maplit;
mod trust;
use trust::Framework;
use trust::Request;


fn abc(req: Request) -> String{
	return "abc".to_string();
}

fn root(req: Request) -> String{
	return "root".to_string();
}

fn main(){
	let mut f = Framework::new();
	f.add("/".to_string(), "get".to_string(), root)
	 .add("/abc".to_string(), "get".to_string(), abc)
	 .run();	
}


