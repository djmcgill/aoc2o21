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
            let new_item = self.i.next()?;
            // drop the oldest item, advance the ring buffer
            buffer[*head] = new_item;
            *head = (*head + 1) % N;

            // okay actually do the test
            // can't be bothered to mess with maybeinit here
            let mut arg = [&buffer[*head]; N];
            for i in 1..N {
                arg[i] = &buffer[(*head + i) % N];
            }

            Some((self.f)(arg))
        } else {
            // this could instead just be `itertools::array_next` if that PR gets merged
            // SAFETY: The `assume_init` is
            // safe because the type we are claiming to have initialized here is a
            // bunch of `MaybeUninit`s, which do not require initialization.
            let mut buffer: [MaybeUninit<I::Item>; N] = unsafe {
                MaybeUninit::uninit().assume_init()
            };
            for i in 0..N {
                let val = self.i.next()?;
                // dbg!(i, &val);
                buffer[i] = MaybeUninit::new(val);
            }

            // SAFETY: we have written all N elements, and `[MaybeUninit<T; N]` and `[T; N]` have the same memory layout
            let buffer = unsafe { 
                let init_arr_ptr = &buffer as *const _ as *const [I::Item; N];
                core::ptr::read(init_arr_ptr)
            };
            let head = 0;


            // okay actually do the test
            // can't be bothered to mess with maybeinit here
            let mut arg = [&buffer[head]; N];
            for i in 1..N {
                let ix = (head + i) % N;
                // dbg!(i, ix);
                arg[i] = &buffer[ix];
            }
            let ret = Some((self.f)(arg));
            // dbg!("Init", &buffer);
            self.ring_buffer = Some((buffer, head));
            ret
        }    
    }
}
