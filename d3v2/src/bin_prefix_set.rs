// The depth of each semi-collapsed node
const D: usize = 2;
// The number of elements that can be contained in that depth
const L: usize = 1 << D;
// a bitmask of `D` rightmost 1s
const D_MASK: usize = (1 << D) - 1;

#[derive(Debug)]
pub struct Node {
    pub child_count: u32,
    pub child: Box<SemiCollapsedBinPrefixSet>,
}

/// Okay the goal of a semi-collapsed bin-prefix set is that it's a binary prefix tree but each node contains `D` levels
/// to reduce indirection (at the cost of using `size_of(Option<Node>) * 1<<D` bytes for each node even empty).
/// If `D` is 1 then it's a normal binary prefix tree.
#[derive(Debug)]
pub enum SemiCollapsedBinPrefixSet {
    Empty,
    /// 1 node contains`D` levels of the tree.
    /// For `D` = 1, contains (is_more_first_zeroes, 0 prefix, 1 prefix)
    ///     i.e. 1 bool, 2 subsets
    /// For `D` = 2, contains (is_more_first_zeroes,
    ///                          (is_more_second_zeroes, 00 prefix, 01 prefix),
    ///                          (is_more_second_zeroes, 10 prefix, 11 prefix),
    ///                      )
    ///     i.e. 3 bool, 4 subsets
    /// For `D` = 3, contains (is_more_first_zeroes,
    ///                          (is_more_second_zeroes,
    ///                              (is_more_third_zeroes, 000 prefix, 001 prefix),
    ///                              (is_more_third_zeroes, 010 prefix, 011 prefix),
    ///                          ),
    ///                          (is_more_second_zeroes,
    ///                              (is_more_third_zeroes, 100 prefix, 101 prefix),
    ///                              (is_more_third_zeroes, 110 prefix, 111 prefix),
    ///                          ),
    ///                      )
    ///     i.e. 7 bool, 8 subsets
    /// etc
    Node {
        child_zeroes_minus_ones: [i32; L - 1],
        // the int is a cache of that node's child count
        children: [Option<Node>; L],
    },
    /// The value itself that was inserted
    Leaf(u32),
}

impl SemiCollapsedBinPrefixSet {
    pub fn deconstruct(self) -> ([i32; L - 1], [Option<Node>; L]) {
        match self {
            SemiCollapsedBinPrefixSet::Node {
                child_zeroes_minus_ones,
                children,
            } => (child_zeroes_minus_ones, children),
            other => panic!("expected Node, found {:?}", other),
        }
    }
    // returns true if it was actually inserted
    // only cares about the right-most `i+1` bits
    pub fn insert(&mut self, x: u32, i: usize) -> bool {
        debug_assert!(i % D == 0);

        // I don't like the double match here but what can you do
        if let SemiCollapsedBinPrefixSet::Empty = self {
            // Also not great intentionally constructing an invalid node
            // but we're going to immediately set it so
            *self = SemiCollapsedBinPrefixSet::Node {
                child_zeroes_minus_ones: Default::default(),
                children: Default::default(),
            };
        }

        let child_index = (x as usize >> (i as usize - D)) & D_MASK;
        println!("PREFIX");
        println!("x = {:b}", x);
        dbg!(i);
        println!("x>>i-D = {:b}", x as usize >> (i as usize - D));
        println!("M = {:b}", D_MASK);
        println!(
            "x>>i-D & M = {:b}",
            (x as usize >> (i as usize - D)) & D_MASK
        );
        println!("ix {} {:b}", child_index, child_index);
        println!("");

        if i == D {
            /*
                1 node contains`D` levels of the tree.
                For `D` = 1, contains (is_more_first_zeroes[0], 0 prefix, 1 prefix)
                    i.e. 1 bool, 2 subsets
                For `D` = 2, contains (is_more_first_zeroes[00],
                                         (is_more_second_zeroes[01], 00 prefix, 01 prefix),
                                         (is_more_second_zeroes[10], 10 prefix, 11 prefix),
                                     )
                    i.e. 3 bool, 4 subsets
                For `D` = 3, contains (is_more_first_zeroes[000],
                                         (is_more_second_zeroes[001],
                                             (is_more_third_zeroes[010], 000 prefix, 001 prefix),
                                             (is_more_third_zeroes[011], 010 prefix, 011 prefix),
                                         ),
                                         (is_more_second_zeroes[100],
                                             (is_more_third_zeroes[101], 100 prefix, 101 prefix),
                                             (is_more_third_zeroes[110], 110 prefix, 111 prefix),
                                         ),
                                     )
                    i.e. 7 bool, 8 subsets
                For `D` = 4, contains (is_more_first_zeros[0000],
                                           (is_more_second_zeros[0001],
                                               (is_more_third_zeros[0010],
                                                   (is_more_forth_zeros[0011], 0000 prefix, 0001 prefix),
                                                   (is_more_forth_zeros[0100], 0010 prefix, 0011 prefix),
                                                (is_more_third_zeros[0101],
                                                   (is_more_forth_zeros[0110], 0100 prefix, 0101 prefix),
                                                   (is_more_forth_zeros[0111], 0110 prefix, 0111 prefix),
                                            (is_more_second_zeros[1000],
                                               (is_more_third_zeros[1001],
                                                   (is_more_forth_zeros[1010], 1000 prefix, 1001 prefix),
                                                   (is_more_forth_zeros[1011], 1010 prefix, 1011 prefix),
                                                (is_more_third_zeros[1100],
                                                   (is_more_forth_zeros[1101], 1100 prefix, 1101 prefix),
                                                   (is_more_forth_zeros[1110], 1110 prefix, 1111 prefix),
            */
            // time to insert!
            match self {
                SemiCollapsedBinPrefixSet::Node {
                    children,
                    child_zeroes_minus_ones,
                } => {
                    match &mut children[child_index] {
                        None => {
                            // okay, actually insert!
                            children[child_index] = Some(Node {
                                child_count: 1,
                                child: Box::new(SemiCollapsedBinPrefixSet::Leaf(x)),
                            });
                            // also update ones vs zero counts

                            // for D = 1, set 0
                            // for D = 2, set 0, set 01 or 10
                            // for D = 3, set 0, set 001 or 100, set 010 or 011 or 101 or 110
                            //                                 001 + 001 or 010
                            //                              or 100 +               001 or 010
                            // for D = 4, set 0, set 0001 or 1000, set 0010 or 0101 or 1001 or 1100, set 0011 or 0100 or 0110 or 0111 or 1010 or 1011 or 1101 or 1110
                            //                                  0001 + 0001 or 0100
                            //                                                  1000 + 0001 or 0100,
                            //                                                             0001 + 0001 + 0001 or 0010
                            //                                                                             0001 + 0100 + 0001 or 0010
                            //                                                                                             1000 + 0001 + 0001 or 0010
                            //                                                                                                             1000 + 0100 + 0001 or 0010

                            let mut ix = 0;
                            for j in 0..D {
                                let mask = 1 << (D - j - 1);
                                let jth_leftmost_bit_set = x & mask != 0;
                                println!("MINUS B");
                                dbg!(&child_zeroes_minus_ones);
                                let offset = if jth_leftmost_bit_set {
                                    child_zeroes_minus_ones[ix] -= 1;
                                    mask
                                } else {
                                    child_zeroes_minus_ones[ix] += 1;
                                    1
                                };
                                ix += offset as usize;
                                dbg!(D);
                                dbg!(j);
                                println!("x  = {:b}", x);
                                println!("m  = {:b}", mask);
                                dbg!(jth_leftmost_bit_set);
                                println!("o  = {:b}", offset);
                                println!("ix = {:b}", ix);
                                println!("");
                            }
                            true
                        }
                        Some(_) => {
                            // something was already there, no changes
                            false
                        }
                    }
                }
                SemiCollapsedBinPrefixSet::Empty => panic!("this Should Never Happen(tm)!!"),
                SemiCollapsedBinPrefixSet::Leaf(_) => panic!("too many digits!"),
            }
        } else {
            // recursion time!
            match self {
                SemiCollapsedBinPrefixSet::Node {
                    child_zeroes_minus_ones,
                    children,
                } => {
                    match &mut children[child_index] {
                        None => {
                            // construct new child node
                            // FIXME: got to update ix counts here too I think?
                            let mut new_child = Box::new(SemiCollapsedBinPrefixSet::Empty);
                            // because we know it's new, we know the insert cannot fail and so this chlid always has exactly 1 element
                            new_child.insert(x, i - D);
                            children[child_index] = Some(Node {
                                child_count: 1,
                                child: new_child,
                            });

                            let mut ix = 0;
                            for j in 0..D {
                                let mask = 1 << (D - j - 1);
                                let jth_leftmost_bit_set = x & mask != 0;
                                println!("MINUS RB");
                                dbg!(&child_zeroes_minus_ones);
                                let offset = if jth_leftmost_bit_set {
                                    child_zeroes_minus_ones[ix] -= 1;
                                    mask
                                } else {
                                    child_zeroes_minus_ones[ix] += 1;
                                    1
                                };
                                ix += offset as usize;
                                dbg!(D);
                                dbg!(j);
                                println!("x  = {:b}", x);
                                println!("m  = {:b}", mask);
                                dbg!(jth_leftmost_bit_set);
                                println!("o  = {:b}", offset);
                                println!("ix = {:b}", ix);
                                println!("");
                            }

                            true
                        }
                        Some(Node { child_count, child }) => {
                            // FIXME: got to update ix counts here too I think?
                            let inserted = child.insert(x, i - D);
                            if inserted {
                                *child_count += 1;
                                let mut ix = 0;
                                for j in 0..D {
                                    let mask = 1 << (D - j - 1);
                                    let jth_leftmost_bit_set = x & mask != 0;
                                    println!("MINUS RR");
                                    dbg!(&child_zeroes_minus_ones);
                                    let offset = if jth_leftmost_bit_set {
                                        child_zeroes_minus_ones[ix] -= 1;
                                        mask
                                    } else {
                                        child_zeroes_minus_ones[ix] += 1;
                                        1
                                    };
                                    ix += offset as usize;
                                    dbg!(D);
                                    dbg!(j);
                                    println!("x  = {:b}", x);
                                    println!("m  = {:b}", mask);
                                    dbg!(jth_leftmost_bit_set);
                                    println!("o  = {:b}", offset);
                                    println!("ix = {:b}", ix);
                                    println!("");
                                }
                            }
                            return inserted;
                        }
                    }
                }
                _ => panic!("expected node"),
            }
        }
    }

    // If there's only one leaf in the tree return it
    pub fn sole_leaf(&self) -> Option<u32> {
        match self {
            SemiCollapsedBinPrefixSet::Leaf(x) => Some(*x),
            SemiCollapsedBinPrefixSet::Node { children, .. } => {
                let mut sole_node = None;
                // If the number of children gets large we might want to avoid a linear search
                // by keeping track of which children have size 1 somehow
                for child_option in children {
                    match child_option {
                        None | Some(Node { child_count: 0, .. }) => (), // keep looking
                        Some(Node {
                            child_count: 1,
                            child,
                        }) => {
                            if sole_node.is_none() {
                                sole_node = Some(child);
                            } else {
                                // Abort abort abort more than 1 non-empty child found
                                return None;
                            }
                        }
                        Some(_) => {
                            // child_size > 1 found, impossible to continue
                            return None;
                        }
                    }
                }
                // We found exactly one child with size 1, so keep going
                sole_node.and_then(|child| child.sole_leaf())
            }
            SemiCollapsedBinPrefixSet::Empty => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    #[test]
    fn t1() -> Result<(), Box<dyn Error>> {
        let mut x = SemiCollapsedBinPrefixSet::Empty;
        assert!(x.sole_leaf().is_none());

        x.insert(0b0101, 4);
        assert_eq!(x.sole_leaf().ok_or("none")?, 0b101);

        x.insert(0b0100, 4);
        assert!(x.sole_leaf().is_none());

        x.insert(0b0001, 4);
        assert!(x.sole_leaf().is_none());

        let (counts, children) = x.deconstruct();
        let [child_00, child_01, child_10, child_11] = children;
        assert_eq!(counts, [3, -1, 0]);
        assert!(child_10.is_none());
        assert!(child_11.is_none());

        let node_00 = child_00.ok_or("none")?;
        assert_eq!(node_00.child_count, 1);
        assert_eq!(node_00.child.sole_leaf().ok_or("none")?, 0b0001);
        let (counts_00, children_00) = node_00.child.deconstruct();
        assert_eq!(counts_00, [1, -1, 0]);
        let [child_0000, child_0001, child_0010, child_0011] = children_00;
        assert!(child_0000.is_none());
        assert!(child_0010.is_none());
        assert!(child_0011.is_none());
        let node_0001 = child_0001.ok_or("none")?;
        assert_eq!(node_0001.child_count, 1);
        assert_eq!(node_0001.child.sole_leaf().ok_or("none")?, 0b0001);
        
        let node_01 = child_01.ok_or("none")?;
        assert_eq!(node_01.child_count, 2);
        assert!(node_01.child.sole_leaf().is_none());

        let (counts_01, children_01) = node_01.child.deconstruct();
        assert_eq!(counts_01, [2, 0, 0]);
        let [child_0100, child_0101, child_0110, child_0111] = children_01;
        assert!(child_0110.is_none());
        assert!(child_0111.is_none());

        let node_0100 = child_0100.ok_or("none")?;
        assert_eq!(node_0100.child_count, 1);
        assert_eq!(node_0100.child.sole_leaf().ok_or("none")?, 0b0100);

        let node_0101 = child_0101.ok_or("none")?;
        assert_eq!(node_0101.child_count, 1);
        assert_eq!(node_0101.child.sole_leaf().ok_or("none")?, 0b0101);

        Ok(())
    }
}
