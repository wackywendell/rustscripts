// Need vec of starting substring frequencies, and then (substring, distance_from_start) --P--> (substring),
// as well as frequency of ending substring

/*
Probabilities

Plan: first build struct so it has the types as below.

Then use word_lengths to select a word length.

Then use starts to find a starting substring with a length as chosen above.

Calculate remaining length, and use pieces[start] to choose the next character.

Repeat above until remaining length is 0; word is finished, and the last piece had a nonzero
ending probability.


Alternate plan:

Use sum(word_lengths * starts[substring]) to choose a starting substring.

Use sum(pieces[substring] * (word_lengths[remaining length:])) to calculate 
probabilities for each continuance.

Will finish at appropriate time on its own.
*/
struct Probabilities {
    // length -> frequency (unnormalized)
    word_lengths: HashMap<u64, u64>,
    // (starting string, [frequency for each length])
    // TODO: could go in pieces as '^'
    starts: Vec<(String, Vec<u64>)>,
    // <previous substring -> [next char, [frequency for each remaining length]]
    pieces: HashMap<String, Vec<(String, Vec<u64>)>>,
}

impl Probabilities {
    fn new(list: Vec<String>, sublens: u32) -> Probabilities {
        return unimplemented!();
    }

    fn dot_product(self, current_length: u64, continuances: &Vec<u64>) {
        let sum = 0;
        for (i, freq) in continuances {
            if freq == 0 {
                continue;
            }
            match self.word_lengths.get(i + current_length) {
                None => break,
                Some(f) => sum += f * freq,
            }
        }
        return sum;
    }

    fn word(&mut self) -> Option<String> {
        for w in self.starts {}
    }
}
