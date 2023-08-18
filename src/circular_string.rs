#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]

//! fixed strings with circular-queue backing

#[derive(Debug,Copy,Clone)]
pub struct cstr<const N : usize=32>
{
  chrs: [u8;N],
  front: u16,
  len: u16,
} //cstr 

impl<const N:usize> cstr<N>
{
   /// create `cstr` from `&str` with silent truncation; panics if
   /// N is greater than 65536
   pub fn make(src:&str) -> cstr<N> {
     if N > 65536 { panic!("cstr strings are limited to a maximum capacity of 65536");}
     let mut m = cstr::new();
     let length = core::cmp::min(N,src.len());
     m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     m
   }//make

   /// version of make that does not truncate.  Also checks if N is
   /// not greater than 65536 without panic.
   pub fn try_make(src:&str) -> Option<cstr<N>> {
     let length = src.len();
     if length>N || N>65536 {return None;}
     let mut m = cstr::new();
     m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     Some(m)
   }//try_make

   /// version of make that returns a pair consisting of the made
   /// `cstr` and the remainder `&str` that was truncated; panics if
   /// N is greater than 65536
   pub fn make_remainder(src:&str) -> (cstr<N>,&str) {
     if N > 65536 { panic!("cstr strings are limited to a maximum capacity of 65536");}   
     let mut m = cstr::new();
     let length = core::cmp::min(N,src.len());
     m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     (m,&src[length..])
   }//try_make

   // make from a pair of str slices, does not truncate, and checks that
   // N is not greater than 65536 without panic
   pub fn from_pair(left:&str, right:&str) -> Option<cstr<N>> {
     let (llen,rlen) = (left.len(), right.len());
     if llen+rlen > N || N > 65536 { return None; }
     let mut m = cstr::new();
     m.len = (llen+rlen) as u16;
     m.chrs[..llen].copy_from_slice(&left.as_bytes()[..llen]);
     m.chrs[llen..].copy_from_slice(&right.as_bytes()[llen..]);
     Some(m)
   }//from_pair

   /// checks if the underlying representation of the string is contiguous
   /// (without wraparound).
   #[inline(always)]
   pub fn is_contiguous(&self) -> bool {
     (self.front as usize + self.len as usize) < N
   }

   /// resets the internal representation of the cstr so that it is
   /// represented contiguously, without wraparound. **Calling this function
   /// has non-constant time cost both in terms of speed and memory** as
   /// it requires a secondary buffer as well as copying.**
   pub fn reset(&mut self) {
     if self.front==0 {return;}
     let mut mhrs = [0;N];
     for i in 0..self.len as usize {
       mhrs[i] = self.chrs[self.index(i)];
     }
     self.chrs = mhrs;
     self.front = 0;
   }//reset

   /// pushes string to the end of the string, returns remainder
   pub fn push_str<'t>(&mut self, src:&'t str) -> &'t str {
     let srclen = src.len();
     let slen = self.len as usize;
     let bytes = &src.as_bytes();
     let length = core::cmp::min(slen+srclen , N);
     let remain = if N>(slen+srclen) {0} else {(srclen+slen)-N};
     let mut i = 0;
     while i<srclen && i+slen<N {
       self.chrs[self.index(slen+i)] = bytes[i];
       i += 1;
     }//while
     self.len += i as u16;
     &src[slen-remain..]
   }//push_str

   /// pushes string to the **front** of the string, returns remainder.
   /// because of the circular-queue backing, this operation as the same
   /// cost as pushing to the back of the string ([Self::push_str]).
   pub fn push_front<'t>(&mut self, src:&'t str) -> &'t str {
     let srclen = src.len();
     let slen = self.len as usize;
     let bytes = &src.as_bytes();
     let length = core::cmp::min(slen+srclen , N);
     let remain = if N>=(slen+srclen) {0} else {(srclen+slen)-N};
     let mut i = 0;
     while i<srclen && i+slen<N {
       self.front = (self.front + (N as u16) -1) % (N as u16);
       self.chrs[self.front as usize] = bytes[srclen-1-i];
       i += 1;
     }//while
     self.len += i as u16;     
     &src[..remain]
   }//push_front


   // convenience
   #[inline(always)]
   fn endi(&self) -> usize {  // index of last value plus 1
     (self.front as usize + self.len as usize )%N
   }// last

   #[inline(always)]
   fn index(&self, i:usize) -> usize {
     (self.front as usize +i)%N
   } // index of ith vale

/*
   #[inline(always)]
   fn right_of(&self, i:usize) -> usize {
     (i+1) % N
   }

   #[inline(always)]
   fn left_of(&self, i:usize) -> usize {
     (i + N - 1) % N
   }
*/

   /// length of string in bytes
   #[inline(always)]
   pub fn len(&self) -> usize  { self.len as usize }

   /// construct new, empty string (same as `cstr::default`)
   #[inline(always)]   
   pub fn new() -> Self {
      Self::default()
   }//new

   /// returns a pair of string slices `(left,right)` which, if concactenated,
   /// will yield an equivalent string underneath.  In case of no wraparound,
   /// the right str will be empty.
   pub fn to_strs(&self) -> (&str,&str) {
     let answer;
     if self.len()==0 {answer = ("","");}
     else if self.is_contiguous() {
       answer = (core::str::from_utf8(&self.chrs[self.front as usize .. self.endi()]).unwrap(),
        "")
     }
     else {
       answer=(core::str::from_utf8(&self.chrs[self.front as usize .. ]).unwrap(),
        core::str::from_utf8(&self.chrs[.. self.endi()]).unwrap())
     }
     answer
   }//to_strs

}//main impl


impl<const N :usize> Default for cstr<N> {
  fn default() -> Self {
    cstr {
       chrs: [0;N],
       front: 0,
       len:0,
    }
  }
}//impl default

