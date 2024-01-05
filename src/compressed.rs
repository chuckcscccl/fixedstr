#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]

// BWT + RLE Compression.

pub struct compactstr<const N : usize>
{
   chrs : [u8; N],
   olen : usize,  // original length
   clen : usize,  // compact length, including a zero
   // eof : usize,  or, position of first zero
} // compactstr

impl<const N:usize> compactstr<N>
{

  pub fn new() -> Self {
    compactstr {
      chrs : [0;N],
      olen : 0,
      clen : 0,
    }
  }
  fn repeats(&self) -> usize {
    let mut ax = 0;
    let mut counts = [0u8;N];
    for i in 0..self.clen {
      let uc = self.chrs[i] as usize;
      if uc != 0 { ax += counts[uc] as usize; counts[uc] += 1; }
    }//for
    ax
  }// counts the number of character repetitions found

  fn rle(s:&str) -> Self {
    let bytes = s.as_bytes();
    let blen = bytes.len();
    let mut compact = compactstr::new();
    if blen==0 {return compact;}
    compact.chrs[0] = bytes[0];
    compact.chrs[1] = 1; // at least one of these bytes
    let mut ci = 1; // indexes compact.chrs
    for bi in 1..blen { // bi indexes bytes
      if bytes[bi] == bytes[bi-1] {
        compact.chrs[ci] += 1;
      }
      else if ci+2 >= N {break;}    
      else {
        compact.chrs[ci+1] = bytes[bi];
        ci += 2;
      }
    } //for
    compact
  }//rle compression, character followed by instances
  

} // impl compactstr

/// Counts the number of repeated *bytes* in a string slice.  This function
/// uses a stack-allocated `[u8;256]` array and runs in linear time.
pub fn count_repeats(s:&str) -> usize {
    let bytes = s.as_bytes();
    let mut ax = 0;
    let mut counts = [0u8;256];
    for i in 0..s.len() {
      let uc = bytes[i] as usize;
      ax += counts[uc] as usize;
      counts[uc] += 1;
    }//for
    ax
}//count_repeats

/// calculates the *compression ratio* of a string as the number of repeated
/// characters divided by its length.
pub fn compression_ratio(s:&str) -> f32 {
  let reps = count_repeats(s) as f32;
  if s.len()==0 {0.0} else { reps / (s.len() as f32) }
}

/*
SACA-K(T, SA, K, n, level)
 T : input string;
 SA: suffix array of T ;
 K: alphabet size of T ;
 n: size of T ;
 level: recursion level;
 Stage 1: induced sort the LMS-substrings of T .
1 if level = 0
then
2 Allocate an array of K integers for bkt;
3 Induced sort all the LMS-substrings of T , using bkt for bucket counters;
else
4 Induced sort all the LMS-substrings of T , reusing the start or
the end of each bucket as the bucket’s counter;
 SA is reused for storing T1 and SA1.
 Stage 2: name the sorted LMS-substrings of T .
5 Compute the lexicographic names for the sorted LMS-substrings to produce T1;
 Stage 3: sort recursively.
6 if K1 = n1  each character in T1 is unique.
then
7 Directly compute SA(T1) from T1;
else
8 SACA-K(T1, SA1, K1, n1, level + 1);
 Stage 4: induced sort SA(T ) from SA(T1).
9 if level = 0
then
10 Induced sort SA(T ) from SA(T1), using bkt for bucket counters;
11 Free the space allocated for bkt;
else
12 Induced sort SA(T ) from SA(T1), reusing the start or the end of
each bucket as the bucket’s counter;
13 return ;
*/
