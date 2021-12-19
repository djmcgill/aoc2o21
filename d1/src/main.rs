use std::str::FromStr;
use itertools::Itertools;
fn main() {
    // Okay it avoids cloning the iterator, and reparsing 
    let input = include_str!("input.txt");
    let part_1 = input.lines()
        .filter_map(|l| u32::from_str(l).ok())
        .tuple_windows()
        .filter(|(previous, next)| previous < next)
        .count();

    let part_2 = input.lines()
        .filter_map(|l| u32::from_str(l).ok())
        .tuple_windows()
        .filter(|(one, two, three, four)| 
            one + two + three < two + three + four
        )
        .count();

    println!("part 1: {}", part_1);
    println!("part 2: {}", part_2);
        
}
