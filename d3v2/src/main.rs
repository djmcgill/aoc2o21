use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
};

mod bin_prefix_set;
use bin_prefix_set::SemiCollapsedBinPrefixSet;

fn main() -> Result<(), Box<dyn Error>> {
    // let (n, path): (_, &'static str) = (5c, "test.txt");
    let (n, path): (_, &'static str) = (12, "input.txt");
    let mut tree = SemiCollapsedBinPrefixSet::Empty;
    BufReader::new(File::open(path)?)
        .lines()
        .filter_map(|rline| rline.ok().and_then(|l| u32::from_str_radix(&l, 2).ok()))
        .for_each(|l| {
            tree.insert(l, n);
        });

    // We need to do the first partition ourselves since the two answers are gotten from the two different partitions
    // FIXME: this is fixed for D = 2 and not sufficiently general yet
    let (counts, children) = tree.deconstruct();
    let [children_00, children_01, children_10, children_11] = children;
    let (msbs, lsbs) = if counts[0] >= 0 {
        if counts[1] >= 0 {
            (children_00, children_11)
        } else {
            (children_01, children_10)
        }
    } else {
        if counts[2] >= 0 {
            (children_10, children_01)
        } else {
            (children_11, children_00)
        }
    };
    // n.b. assuming this isn't 1 level deep
    let msbs = *msbs.expect("invalid problem").child;
    let lsbs = *lsbs.expect("invalid problem").child;
    println!("{}", go(true, msbs) * go(false, lsbs));
    Ok(())
}

// FIXME: this is fixed for D = 2 and not sufficiently general yet
fn go(msb: bool, xs: SemiCollapsedBinPrefixSet) -> u32 {
    let (counts, children) = xs.deconstruct();
    let [children_00, children_01, children_10, children_11] = children;
    let debug_str = format!("invalid problem {}\n{:?}\n{:?}\n{:?}\n{:?}\n{:?}", msb, counts, children_00, children_01, children_10, children_11);
    let keep_zero = if counts[0] == 0 {
        !msb
    } else {
        (counts[0] > 0) == msb
    };
    let candidates = if keep_zero {
        let keep_zero = if counts[1] == 0 {
            !msb
        } else {
            (counts[1] > 0) == msb
        };
        if keep_zero {
            children_00
        } else {
            children_01
        }
    } else {
        let keep_zero = if counts[2] == 0 {
            !msb
        } else {
            (counts[2] > 0) == msb
        };
        if keep_zero {
            children_10
        } else {
            children_11
        }
    }.unwrap_or_else(|| panic!("{}", debug_str));

    // FIXME: add sole_leaf method to `Node` too, doesn't make sense to throw away the count here
    candidates.child
        .sole_leaf()
        .unwrap_or_else(|| go(msb, *candidates.child))
}
