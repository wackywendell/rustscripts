#![feature(plugin)]
#![plugin(docopt_macros)]

extern crate "rustc-serialize" as rustc_serialize;

extern crate docopt;
extern crate rand;

extern crate rustscripts;

#[warn(non_camel_case_types)]
#[warn(non_snake_case)]
#[warn(unused_qualifications)]
#[warn(non_upper_case_globals)]
#[warn(missing_docs)]

/** Creates fake words.
 *  First, it uses a predefined list of words to generate a markov chain
 *  for word prefixes; it then probabilistically follows this to generate
 *  fake words.
**/

use docopt::Docopt;

use rand::Rng;

use std::collections::{HashSet,HashMap};
use std::old_io::{File,BufferedReader};
use std::slice::AsSlice;
//use std::iter::{FromIterator,IteratorExt};

use rustscripts::counter;

//* Build a HashMap of "substring" : "number of occurrences"
fn to_hashes(wordlist : &[String], sublens : u32) -> HashMap<String, u32> {
	println!("Got wordlist, length {}", wordlist.len());
	
	let iter = wordlist.iter().filter(|k| {
		if k.chars().count() == 0 {false}
		else if k.contains("\'") {false}
		else if k.find(|&: c : char|{!c.is_lowercase()}).is_some() {false}
		else {true}
	});
	let trimchars : &[char] = &[' ', '\t', '\r', '\n'];
	let iter = iter.flat_map(|k| {
		let kslice = k.as_slice();
		let fullword : String = (["^", kslice.trim_matches(trimchars), "$"]).concat();
        let fullchars : Vec<char> = fullword.chars().collect();
        
        let chvec : Vec<String> =
		fullchars.windows(sublens as usize).map(|chars| {
			chars.iter().map(|&x| x).collect::<String>()
		})
		.collect();
		
		//~ println!("k: {}, chvec: {}", k, chvec);
		
		chvec.into_iter()
	});
	counter(iter)
}

struct WordBuilder {
    subs : HashMap<String, u32>,
    // list : Vec<String>,
    // sublens : u32,
    wordset : HashSet<String>,
    wordlens : Vec<u32>
}

struct WordIter<'a> {
    p : &'a mut WordBuilder
}

impl WordBuilder {
    fn new(list : Vec<String>, sublens : u32) -> WordBuilder {
        let mut h = HashSet::new();
        let mut wlens : Vec<u32> = Vec::new();
        for w in list.iter() {
            h.insert(w.to_string());
            let wordlen = w.len();
            for _ in range(wlens.len(), wordlen+1){
                wlens.push(0);
            }
            wlens[wordlen] += 1;
        }
        
        WordBuilder {
            subs : to_hashes(list.as_slice(), sublens),
            //list : list,
            //sublens : sublens,
            wordset : h,
            wordlens : wlens
        }
    }
    
    fn word(&mut self) -> Option<String> {
        let mut s : String = "^".to_string();
        
        loop {
            let mut fullsum = 0u32;
            let mut endsum = 0u32;
            let possibilities : Vec<(&str, u32)> = self.subs.iter().filter_map(
                |(k,v)| {
                    /* the beginning of k and the end of s must match for
                     * k to be a possibility
                     * if s is long, then the first (klength - 1) letters
                     * of k must match the last (klength - 1) letters of s
                     * otherwise, the first (slength) characters
                    */
                    let slen = s.as_slice().chars().count();
                    let kslice = k.as_slice();
                    let klen = kslice.chars().count();
                    let kcut = if slen < klen - 1 {slen} else {klen - 1};
                    if s.as_slice().ends_with(kslice.slice_chars(0, kcut)){
                        fullsum += *v;
                        if kslice.ends_with("$") {endsum += *v;}
                        Some((kslice,*v))
                    } else {None}
                }
            ).collect();
            if fullsum == 0 {
                panic!("s: \"{}\"", s);
            }
            
            let endprob = if self.wordlens.len() > s.len() {
                let wordlenslice : &[u32] = &(self.wordlens[(s.len()-1)..self.wordlens.len()]);
                let c = wordlenslice[0];
                let l = wordlenslice.iter().fold(0, |a,&b|{a+b});
                (c as f64) / (l as f64)
            } else {
                //~ println!("Too long: {}", s);
                return None;
            };
            
            let randnum = rand::thread_rng().gen_range(0.0, 1.0);
            
            let endtime = randnum < endprob;
            if (endtime && (endsum == 0)) || ((!endtime) && (fullsum-endsum==0)) {
                //~ println!("Failed to end: {}", s);
                return None;
            }
            
            //~ println!("endtime: {} {} : ({},{}) {}", endtime, randnum < endprob,
                //~ endsum, fullsum, if(endtime){endsum} else {fullsum - endsum});
            let randnum = rand::thread_rng().gen_range(0.0,
                (if endtime {endsum} else {fullsum - endsum} as f64));
            
            let mut psum = 0;
            
            for &(k,v) in possibilities.iter() {
                if endtime ^ k.ends_with("$") {continue;};
                
                if randnum < ((psum + v) as f64) {
                    let slen = s.as_slice().chars().count();
                    let klen = k.chars().count();
                    let kcut = if slen < klen - 1 {slen} else {klen - 1};
                    //~ let olds = s.to_string();
                    s.push_str(k.slice_chars(kcut,klen));
                    break;
                }
                psum += v;
            }
            
            let slen = s.as_slice().chars().count();
            if s.as_slice().slice_chars(slen-1, slen) == "$" {
                return Some(s.as_slice().slice_chars(1, slen-1).to_string());
            }
        };
    }
    
    fn iter<'a>(&'a mut self) -> WordIter<'a> {
        WordIter{p:self}
    }
}

impl<'a> Iterator for WordIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            let optw = self.p.word();
            match optw {
                None => {},
                Some(w) => {
                    if !self.p.wordset.contains(&w)
                        {return Some(w)};
                }
            };
        }
    }
}

docopt!(Args, "
Usage: fakewords2 [-h | --help] [-n <number>] [<dictfile>]

Options:
    -n <number>    use number length substrings for markovian chain
    <dictfile>     use dictfile instead of /usr/share/dict/words
", flag_n : Option<u32>, arg_dictfile : Option<String>);

pub fn main(){
	let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    
    let flag_n : Option<u32> = args.flag_n;

    let subsetn : u32 = flag_n.unwrap_or(4);
    
    let pathstr = args.arg_dictfile.map(|s| {s}).unwrap_or("/usr/share/dict/words".to_string());
    
    let path = Path::new(pathstr.clone());
    let file = match File::open(&path) {
		Ok(f) => f,
		Err(std::old_io::IoError{kind: std::old_io::FileNotFound, desc: _, detail: _}) => {
			let _ = writeln!(&mut std::old_io::stderr(), "File not found: {}", pathstr);
			std::env::set_exit_status(-1);
			return;
		}
		Err(e) => panic!("failed to open file: {}", e)
	};
    
    let mut file = BufferedReader::new(file);
    
    let trimchars : &[char] = &[' ', '\t', '\r', '\n'];
    
    let lines: Vec<String> = file.lines().map(|orl| {
		let unwrapl : String = match orl {
			Ok(l) => l,
			Err(e) => panic!("Failed reading file: {}", e)
		};
		unwrapl.trim_matches(trimchars).to_string()
	}).collect();
    let mut wb = WordBuilder::new(lines, subsetn);
    
    println!("Now have a map of length {}", wb.subs.len());
    
    for w in wb.iter(){
        println!("{}",w);
    };
}
