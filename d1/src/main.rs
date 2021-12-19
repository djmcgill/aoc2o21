#![feature(maybe_uninit_array_assume_init)]

use std::{str::FromStr, mem::MaybeUninit};
// use itertools::Itertools;
fn main() {
    // Okay it avoids cloning the iterator, and reparsing 
    let input = include_str!("input.txt");
    let part_1 = input.lines()
        .filter_map(|l| u32::from_str(l).ok())
        // OLD BUSTED
        // .tuple_windows()
        // .filter(|(previous, next)| previous < next)
        // NEW HOTNESS
        .window_map(|[prev, next]| prev < next)
        .filter(|i| *i)
        .count();

    let part_2 = input.lines()
        .filter_map(|l| u32::from_str(l).ok())
        // OLD BUSTED:
        // .tuple_windows()
        // .filter(|(one, two, three, four)| 
        //     one + two + three < two + three + four
        // )
        // NEW HOTNESS:
        .window_map(|[one, two, three, four]| {
            // dbg!([one, two, three, four]);
            one + two + three < two + three + four
        })
        .filter(|i| *i)
        .count();

    println!("part 1: {}", part_1);
    println!("part 2: {}", part_2);
        
}

trait WindowMapExt: Iterator + Sized {
    fn window_map<B, F, const N: usize>(self, f: F) -> WindowMap<B, Self, F, N> 
        where F: FnMut([&Self::Item; N]) -> B
    {
        WindowMap {f, i: self, ring_buffer: None}
    }
}
impl<I: Iterator> WindowMapExt for I {}
// Okay to avoid the need to return references, or to clone to avoid returning references, just force the user to consume them immediately
struct WindowMap<B, I: Iterator, F: FnMut([&I::Item; N]) -> B, const N: usize>{
    f: F,
    ring_buffer: Option<([I::Item; N], usize)>,
    i: I,
}
impl<B, I: Iterator, F: FnMut([&I::Item; N]) -> B, const N: usize> Iterator for WindowMap<B, I, F, N> where I::Item: std::fmt::Debug {
    type Item = B;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((buffer, head)) = &mut self.ring_buffer {
            // drop the oldest item, advance the ring buffer
            let new_item = self.i.next()?;
            buffer[*head] = new_item;
            *head = (*head + 1) % N;
        } else {
            // Initialise the buffer. This could instead just be `itertools::array_next` if that PR gets merged
            // SAFETY: The `assume_init` is
            // safe because the type we are claiming to have initialized here is a
            // bunch of `MaybeUninit`s, which do not require initialization.
            let mut buffer: [MaybeUninit<I::Item>; N] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for item in buffer.iter_mut() {
                let val = self.i.next()?;
                *item = MaybeUninit::new(val);
            }

            // SAFETY: we have written all N elements
            let buffer = unsafe { 
                MaybeUninit::array_assume_init(buffer)
            };
            let head = 0;
            self.ring_buffer = Some((buffer, head));
        };
        let (buffer, head) = self.ring_buffer.as_ref().expect("we just checked if it was initialised or not!");
         // okay actually do the test
        // can't be bothered to mess with maybeinit here
        let mut arg = [&buffer[*head]; N];
        // ok clippy lol, sure
        for (i, item) in arg.iter_mut().enumerate().skip(1) {
            let ix = (head + i) % N;
            // dbg!(i, ix);
            *item = &buffer[ix];
        }
        Some((self.f)(arg))
    }
}
