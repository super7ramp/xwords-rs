/*!
An algorithm that composes algorithms and data structures throughout this
crate. This is where the magic happens.
*/

use std::{collections::HashSet, hash::BuildHasherDefault};

use rustc_hash::FxHasher;

use crate::{
    crossword::{Crossword, WordIterator},
    parse::parse_word_boundaries,
    trie::Trie,
};

use super::{
    build_square_word_boundary_lookup,
    cache::{CachedIsViable, CachedWords},
    fill_one_word, is_viable_reuse, words_orthogonal_to_word, Fill,
};

pub type InterruptCheck<'s> = dyn FnMut() -> bool + 's;

pub struct Filler<'s> {
    word_cache: CachedWords,
    is_viable_cache: CachedIsViable,
    is_interrupted: &'s mut InterruptCheck<'s>,

    trie: &'s Trie,
}

impl<'s> Filler<'s> {
    pub fn new(trie: &'s Trie, is_interrupted: &'s mut InterruptCheck<'s>) -> Filler<'s> {
        Filler {
            word_cache: CachedWords::default(),
            is_viable_cache: CachedIsViable::default(),
            is_interrupted,
            trie,
        }
    }
}

impl<'s> Fill for Filler<'s> {
    fn fill(&mut self, initial_crossword: &Crossword) -> Result<Crossword, String> {

        let word_boundaries = parse_word_boundaries(&initial_crossword);
        let mut already_used = HashSet::with_capacity_and_hasher(
            word_boundaries.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );

        let mut candidates = vec![initial_crossword.to_owned()];

        let word_boundary_lookup = build_square_word_boundary_lookup(&word_boundaries);

        while let Some(candidate) = candidates.pop() {

            if (self.is_interrupted)() {
                break;
            }

            let to_fill = word_boundaries
                .iter()
                .map(|word_boundary| WordIterator::new(&candidate, word_boundary))
                .filter(|iter| iter.clone().any(|c| c == ' '))
                .min_by_key(|iter| {
                    (
                        self.word_cache.words(iter.clone(), self.trie).len(),
                        iter.word_boundary.start_row,
                        iter.word_boundary.start_col,
                    )
                })
                .unwrap();

            let orthogonals =
                words_orthogonal_to_word(&to_fill.word_boundary, &word_boundary_lookup);

            let potential_fills = self.word_cache.words(to_fill.clone(), self.trie);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), &potential_fill);

                let (viable, tmp) = is_viable_reuse(
                    &new_candidate,
                    &orthogonals,
                    self.trie,
                    already_used,
                    &mut self.is_viable_cache,
                );
                already_used = tmp;
                already_used.clear();

                if viable {
                    if !new_candidate.contents.contains(' ') {
                        return Ok(new_candidate);
                    }
                    candidates.push(new_candidate);
                }

            }
        }

        Err(String::from("We failed"))
    }
}

#[cfg(test)]
mod tests {

    use crate::{fill::Fill, Trie};

    use crate::Crossword;

    use std::{cmp::Ordering, time::Instant};

    use super::Filler;

    #[test]
    fn test() {
        assert_eq!((1, 2).cmp(&(3, 4)), Ordering::Less)
    }

    #[test]
    fn medium_grid() {
        let grid = Crossword::square(String::from(
            "
    ***
    ***
    ***
       
***    
***    
***    
",
        ))
        .unwrap();

        let now = Instant::now();
        let trie = Trie::load_default().expect("Failed to load trie");
        let mut never_interrupt = || false;
        let mut filler = Filler::new(&trie, &mut never_interrupt);
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
