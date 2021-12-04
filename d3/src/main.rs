#![feature(iter_partition_in_place)]
use std::iter::FromIterator;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

fn main() -> Result<(), Box<dyn Error>> {
    let n = 11;
    let lines = BufReader::new(File::open("input.txt")?)
        .lines()
        .filter_map(|rline| rline.ok().and_then(|l| u32::from_str_radix(&l, 2).ok()));

    // We need to do the first partition ourselves since the two answers are gotten from the two different partitions
    let mut vec_iter = Vec::from_iter(lines);
    let (countg0, ones, zeroes) = step(n, vec_iter.as_mut());
    let (msbs, lsbs) = if countg0 { (ones, zeroes) } else { (zeroes, ones) };
    println!("{}", go(true, n - 1, msbs) * go(false, n - 1, lsbs));
    Ok(())
}

fn go(msb: bool, n: usize, xs: &mut [u32]) -> u32 {
    let (countg0, v1, v0) = step(n, xs);
    let candidates = if countg0 == msb { v1 } else { v0 };
    if candidates.len() <= 1 {
        return candidates[0];
    }
    go(msb, n - 1, candidates)
}

fn step(n: usize, xs: &mut [u32]) -> (bool, &mut [u32], &mut [u32]) {
    let xsn = xs.len();
    let v1n = xs.iter_mut().partition_in_place(|&x| x & (1 << n) != 0);
    let (v1, v0) = xs.split_at_mut(v1n);
    (v1n*2 >= xsn, v1, v0) // integer divisionnnnn!!!!
}
