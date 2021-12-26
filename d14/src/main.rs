use std::{collections::{BTreeMap}, };

use nom::{character::complete::{alpha1, anychar, line_ending}, multi::separated_list1, bytes::complete::tag};
use itertools::Itertools;

type Rule<A> = BTreeMap<(char, char), A>;
type Count = BTreeMap<char, u64>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 40 = one pass of 2^^3 and one pass of 2^^5
    let input = include_str!("input.txt");
    let first_pass = 3;
    let rules_passes = 5;

    // 10 = one pass of 2^^1 and one pass of 2^^3 
    // let input = include_str!("test.txt");
    // let first_pass = 1;
    // let rules_passes = 3;

    let problem = Problem::parse(input)?.1;
    let caches = precalculate_caches(first_pass, rules_passes, problem.rules);

    let mut chars = problem.template.chars().peekable();

    // the algo iterates over pairs, adding the 2nd of each pair, so the very first char needs to be manually added
    let first_char = *chars.peek().unwrap();

    // We just use a single mutable map to keep track of char counts
    // Could probably do everything more nicely with a fold to combine the btreemaps?
    let mut count_map: Count = BTreeMap::from([(first_char, 1)]);
   
    // We expand the actual rules out once, to a depth of `first_pass`, to ensure that the remaining `n` is a power of 2.
    // Some cases might have to do it more than once but both 10 and 40 are the sum of two different powers of 2 so it's fine.
    chars.tuple_windows().for_each(|(first, second)| {
        let count = if let Some(expansion) = caches.first_pass_expansion.get(&(first, second)) {
            let mut expansion_count = BTreeMap::new();
            // Okay now we have a semi-large expansion, there's no way we could actually expand it again so instead
            // we use the pre-calculated counts to work out what the counts of its expansion would be.
            let full_expansion = sandwich(first, expansion, second);
            for (inner_first, inner_second) in full_expansion.tuple_windows() {
                let inner_count = expand_with_counts(inner_first, inner_second, &caches.final_pass_expansion_count);
                add_counts(&mut expansion_count, &inner_count);
            }
            expansion_count
        } else {
            // If there's no rule to apply, just add the 2nd of the pair
            BTreeMap::from([(second, 1)])
        };
        add_counts(&mut count_map, &count);
    });

    let ans = max_minus_min(count_map.into_values());
    println!("{}", ans);

    Ok(())
}

fn max_minus_min(iter: impl Iterator<Item=u64>) -> u64 {
    let mut min = u64::MAX;
    let mut max = 0u64;
    for x in iter {
        if x < min {min = x}
        if x > max {max = x}
    }
    max - min
}

struct PreCalculatedCaches {
    first_pass_expansion: Rule<Vec<char>>,
    final_pass_expansion_count: Rule<Count>,
}

fn precalculate_caches(first_pass: usize, rules_passes: usize, rules: Vec<(char, char, char)>, ) -> PreCalculatedCaches {

    let mut rules_maps = vec![];

    // Okay doing rules one at a time is for suckers let's precalculate their expansion
    let mut rules_map = BTreeMap::new();
    for (k1, k2, v) in rules {
        rules_map.insert((k1, k2), vec![v]);
    }
    rules_maps.push(rules_map);

    // Until we get too large, (n = 6 is right out), let's expand the map ourselves
    for i in 1..rules_passes {
        let mut new_map = BTreeMap::new();
        for ((first, second), old_expansion) in &rules_maps[i-1] {

            let full_old_expansion = sandwich(*first, old_expansion, *second);
            let mut new_expansion = vec![];
            for (inner_first, inner_second) in full_old_expansion.tuple_windows() {
                if let Some(inner_match) = rules_maps[i-1].get(&(inner_first, inner_second)) {
                    for inner_v in inner_match {
                        new_expansion.push(*inner_v);
                    }
                    new_expansion.push(inner_second);
                }
            }
            new_expansion.pop();
            new_map.insert((*first, *second), new_expansion);
            
        }
        rules_maps.push(new_map);
    }
    let last_rule_map = rules_maps.remove(rules_passes-1);
    let first_pass_rule_map = rules_maps.remove(first_pass);

    // get outta here buddy
    std::mem::drop(rules_maps);

    // Okay this countmap's goal is to turn each rule from
    // (A, E) -> "BCD"
    // to 
    // (A, E) ->  {B: 1, C: 1, D: 1}
    let mut semifinal_rules_pass_count_map = BTreeMap::new();
    for ((k0, k1), vs) in &last_rule_map {
        let mut count_map = BTreeMap::new();
        for v in vs {
            map_add(&mut count_map, *v, 1);
        }
        semifinal_rules_pass_count_map.insert((*k0, *k1), count_map);
    }

    // Okay this countmap's goal is to do the same as the above, except also do a level of expansion before counting
    let final_rules_pass_count_map = 
        calculate_final_rules_pass_count_map(
            &last_rule_map, 
            &semifinal_rules_pass_count_map,
        );

    PreCalculatedCaches {
        first_pass_expansion: first_pass_rule_map,
        final_pass_expansion_count: final_rules_pass_count_map,
    }
}

struct Problem {
    template: String,
    rules: Vec<(char, char, char)>,
}
impl Problem {
    fn parse(input: &str) -> nom::IResult<&str, Self> {
        let (input, template) = alpha1(input)?;
        let (input, _) = line_ending(input)?;
        let (input, _) = line_ending(input)?;
        let (input, rules) = separated_list1(line_ending, parse_triple)(input)?;
        Ok((input, Self {template: template.to_string(), rules}))
    }
}

fn parse_triple(input: &str) -> nom::IResult<&str, (char, char, char)> {
    let (input, left) = anychar(input)?;
    let (input, right) = anychar(input)?;
    let (input, _) = tag(" -> ")(input)?;
    let (input, end) = anychar(input)?;
    Ok((input, (left, right, end)))
}

fn calculate_final_rules_pass_count_map(previous_rule: &Rule<Vec<char>>, previous_count: &Rule<Count>) -> Rule<Count> {
    let mut final_rules_pass_count_map = BTreeMap::new();
    for ((k0, k1), vs) in previous_rule {
        let mut count_map = BTreeMap::new();

        // For each pair of chars we add the sum
        let full_expansion = sandwich(*k0, &vs[..], *k1);
        for (first, second) in full_expansion.tuple_windows() {
            if let Some(sub_expansion) = previous_count.get(&(first, second)) {
                add_counts(&mut count_map, sub_expansion);
            }
            map_add(&mut count_map, second, 1);
        }
        // okay since we've doind BB -> ABA and DON'T want the bookends, but we've been adding the 2nd as we go, we gotta take it off
        map_sub(&mut count_map, *k1, 1);
        final_rules_pass_count_map.insert((*k0, *k1), count_map);
    }
    final_rules_pass_count_map
}


fn map_add<K: Ord>(map: &mut BTreeMap<K, u64>, k: K, n: u64) {
    let count = map.entry(k).or_insert(0);
    *count += n;
}

fn map_sub<K: Ord>(map: &mut BTreeMap<K, u64>, k: K, n: u64) {
    let count = map.entry(k).or_insert(0);
    *count -= n;
}

fn add_counts(this: &mut Count, other: &Count) {
    for (v, v_count) in other {
        map_add(this, *v, *v_count);
    }
}

fn sandwich<'a, T: Clone>(start: T, middle: &'a [T], end: T) -> impl Iterator<Item=T> + 'a {
    std::iter::once(start).chain(middle.into_iter().cloned()).chain(std::iter::once(end))
}

// Do an expansion, but instead of expanding out the text, use the pre-calculated expansion counts
fn expand_with_counts(first: char, second: char, final_count_map: &Rule<Count>) -> Count {
    let mut count = BTreeMap::from([(second, 1)]);

    if let Some(expansion_count) = final_count_map.get(&(first, second)) {
        add_counts(&mut count, expansion_count);
    }
    count
}
