extern crate getopts;
extern crate rand;
extern crate rustscripts;

/** Creates fake words.
 *  First, it uses a predefined list of words to generate a markov chain
 *  for word prefixes; it then probabilistically follows this to generate
 *  fake words.
**/

use getopts::Options;
use rand::Rng;

use std::collections::{HashSet,HashMap};
use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{Write,BufRead,BufReader};
use std::convert::AsRef;
//use std::iter::{FromIterator,IteratorExt};

use rustscripts::counter;

//* Build a HashMap of "substring" : "number of occurrences"
fn to_hashes(wordlist : &[String], sublens : u32) -> HashMap<String, u32> {
	println!("Got wordlist, length {}", wordlist.len());

	let iter = wordlist.iter().filter(|k| {
		if k.chars().count() == 0 {false}
		else if k.contains("\'") {false}
		else if k.find(|c : char|{!c.is_lowercase()}).is_some() {false}
		else {true}
	});
	let trimchars : &[char] = &[' ', '\t', '\r', '\n'];
	let iter = iter.flat_map(|k| {
		let fullword : String = (["^", k.trim_matches(trimchars), "$"]).concat();
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
            for _ in wlens.len()..wordlen+1 {
                wlens.push(0);
            }
            wlens[wordlen] += 1;
        }

        WordBuilder {
            subs : to_hashes(list.as_ref(), sublens),
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
                    let slen = s.chars().count();
                    let klen = k.chars().count();
                    let kcut = if slen < klen - 1 {slen} else {klen - 1};
					let kstart : String = k.chars().take(kcut).collect();
					if s.ends_with(&kstart){
                        fullsum += *v;
                        if k.ends_with("$") {endsum += *v;}
                        Some((k.as_ref(),*v))
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
                    let slen = s.chars().count();
                    let klen = k.chars().count();
                    let kcut = if slen < klen - 1 {slen} else {klen - 1};
                    //~ let olds = s.to_string();
                    s.push_str(&k[kcut..klen]);
                    break;
                }
                psum += v;
            }

            let slen = s.chars().count();
            if &s[slen-1..slen] == "$" {
                return Some(s[1..slen-1].to_string());
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

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [-h | --help] [-n <number>] [<dictfile>]", program);
    print!("{}", opts.usage(&brief));
}

// Options:
//     -n <number>    use number length substrings for markovian chain
    // <dictfile>     use dictfile instead of /usr/share/dict/words
// flag_n : Option<u32>, arg_dictfile : Option<String>);

pub fn main(){
	 let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("n", "", "use number length substrings for markovian chain", "NUMBER");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    let number = matches.opt_str("n");
    let subsetn : u32 = match number {
        Some(n_str) => n_str.parse().unwrap_or_else(|e|
            panic!("Could not parse -n {} as integer", n_str)),
        None => 4
    };

    let pathstr = if matches.free.is_empty() {
        "/usr/share/dict/words".to_string()
    } else if matches.free.len() == 1 {
        matches.free[0].clone()
    } else {
        panic!("Only expected one dictfile");
    };

    let path = Path::new(&pathstr);
    let file = match File::open(&path) {
		Ok(f) => f,
		Err(e) => match e.kind() {
			std::io::ErrorKind::NotFound => {
				let _ = writeln!(&mut std::io::stderr(), "File not found: {}", pathstr);
				std::process::exit(-1);
			}
			_ => panic!("failed to open file: {}", e)
		}
	};

	let file = BufReader::new(file);

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
