use std::fs::File;
use std::io::prelude::*;
use std::collections::HashMap;
use regex::Regex;
use regex::Match;


pub fn render_template<P: ToString>(filename: &str, params: HashMap<String,P>) -> String {
    let mut f = File::open(filename).expect("file not found");
    let mut contents = String::new();
    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    // for mat in Regex::new(r"\{\{[^}]*\}\}").unwrap().find_iter(&contents) {
    //     println!("match = {:?}",mat);
    // }
    let matches : Vec<Match> = Regex::new(r"\{\{[^}]*\}\}").unwrap().find_iter(&contents).collect();
    println!("matches = {:?}",matches);
    let mut templated = String::new();
    let mut original_ind = 0;
    let mut matches_ind = 0;

    while (original_ind < matches.len()) {
        while matches_ind < matches[original_ind].start() {
            templated.push_str  (contents.get(matches_ind as usize .. matches_ind+1 as usize).unwrap().clone());
            matches_ind += 1;
        }
        let mat =  matches[original_ind];
        println!("{}", contents.get(mat.start() as usize..mat.end() as usize).unwrap().clone());
        let word = contents.get(mat.start() + 2 as usize..mat.end() - 2 as usize).unwrap().clone().trim();
        let value = params.get(word).expect("The key doesn't exist in the parameters for the jinja template");
        templated.push_str(&value.to_string());
        matches_ind = matches[original_ind].end();
        original_ind += 1;
    }

    while matches_ind < contents.len() {
        templated.push_str(contents.get(matches_ind as usize .. matches_ind+1 as usize).unwrap().clone());
        matches_ind+=1;
    }

    return templated;
}
