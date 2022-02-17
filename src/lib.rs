//! Library for strings of fixed maximum lengths that can be copied and
//! stack-allocated using Rust's new const generics feature.
//!
//! **The main structure provided by this crate are [fstr] and [zstr].**
//!
//! **Version 0.2.x** adds **unicode support** and a new module for
//! **zero_terminated strings** in the structure [zstr].
//! These strings are more memory efficient than [fstr] but less efficient
//! in terms of run time.

//! Example:
//!
//!```ignore
//!   let s1 = str16::from("abc"); // str16 is alias for fstr<16>
//!   let mut s2 = str32::from("and xyz");
//!   s2.push(" and 1234");  // adds to end of s2
//!   println!("{} {}, {}", s1, &s2, s2.len());  
//!   println!("{}", &s1=="abc");   // can compare with &str
//!   let s3 = s1;     // copied, not moved
//!   println!("{}", "abc"==&s1);
//!   println!("{}, {} ", s1==s3, s1==s2.resize()); 
//!   println!("{}", s2.substr(2,6));
//!   let s4:fstr<64> = s1.resize();  // resize copies to new-capacity fstr
//!   let owned_string = s4.to_string();
//!   let str_slice:&str = s4.to_str();
//!   let z:zstr<8> = zstr::from("λxλy.x");
//!```

// Fixed, :Copy strings of limited size.  size of each fstr is N or less,
// array chrs is 0-terminated if size of string is less than N:

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]

pub mod zero_terminated;
pub use zero_terminated::*;

use std::cmp::Ordering;

/// main type: fixed string of size up to N:
#[derive(Copy,Clone,Eq,PartialEq,Hash)]
pub struct fstr<const N:usize>
{
  chrs : [u8;N],
  len : usize,  // length will be <=N
}//fstr
impl<const N:usize> fstr<N>
{
  /// creates a new fstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.  This function is also called by
  /// several others including [fstr::from].
  pub fn make(s:&str) -> fstr<N>
   {
      if (N>65536 || N<1) {panic!("Valid fstr strings are limited to fstr<1> to zstr<65536>");}
      let bytes = s.as_bytes(); // &[u8]
      let blen = bytes.len();
      let mut chars = [0u8; N];
      let mut i = 0;
      for i in 0..blen
      {
        if i<N {chars[i] = bytes[i];} else {break;}
      }
      fstr {
         chrs: chars, len: blen, /* as u16 */
      }
/*
      let mut chars = [0u8; N];
      let mut i = 0;
      for c in s.chars()
      {
         if i<N { chars[i] = c as u8; i+=1; } else {break;}
      }
      fstr {
         chrs: chars,
         len: i,
      }
 */
   }//make

   /// creates an empty string, equivalent to fstr::default()
   pub fn new() -> fstr<N>
   {
     fstr::make("")
   }

   /// length of the string in bytes, which will be up to the maximum size N.
   /// This is a constant-time operation. Note that this value is consistent
   /// with [str::len]
   pub fn len(&self)->usize { self.len }

   /// converts fstr to an owned string
   pub fn to_string(&self) -> String
   {
     self.to_str().to_owned()
     //self.chrs[0..self.len].iter().map(|x|{*x as char}).collect()
   }

   /// allows returns copy of u8 array underneath the fstr
   pub fn as_u8(&self) -> [u8;N]
   {
      self.chrs
   }

   /// converts fstr to &str using [std::str::from_utf8]
   pub fn to_str(&self) -> &str
   {
      std::str::from_utf8(&self.chrs[0..self.len]).unwrap()
   }
   /// alias for [fstr::to_str]
   pub fn as_str(&self) -> &str {self.to_str()}

   /// changes a character at character position i to c.  This function
   /// requires that c is in the same character class (ascii or unicode)
   /// as the char being replaced.  It never shuffles the bytes underneath.
   /// The function returns true if the change was successful.
   pub fn set(&mut self,i:usize, c:char) -> bool
   {
      let ref mut cbuf = [0u8;4];  // characters require at most 4 bytes
      c.encode_utf8(cbuf);
      let clen = c.len_utf8();
      if let Some((bi,rc)) = self.to_str().char_indices().nth(i) {
        if clen==rc.len_utf8() {
           for k in 0..clen {self.chrs[bi+k] = cbuf[k];}
           return true;
        }
      }
      return false;
      //if i<self.len {self.chrs[i]=c as u8; true} else {false}
   }
   /// adds chars to end of current string up to maximum size N of fstr<N>,
   /// returns the portion of the push string that was NOT pushed due to
   /// capacity, so
   /// if "" is returned then all characters were pushed successfully.
   pub fn push<'t>(&mut self,s:&'t str) -> &'t str
   {
      if s.len()<1 {return s;}
      let mut buf = [0u8;4];
      let mut i = self.len();
      let mut sci = 0; // indexes characters in s
      for c in s.chars()
      {
         let clen = c.len_utf8();
         c.encode_utf8(&mut buf);
         if i<=N-clen {
           for k in 0..clen
           {
             self.chrs[i+k] = buf[k];
           }
           i += clen;
         } else  { self.len = i; return &s[sci..];}
         sci += 1;
      }
      self.len=i;
      &s[sci..]
   /*
      let mut i = self.len;
      for c in s.chars()
      {
         if i<N {self.chrs[i] = c as u8; i+=1;} else {self.len=N; return &s[N..];}
      }
      self.len = i;
      if (i<s.len()) {&s[i..]} else {""}
   */      
   }

   /// returns the number of characters in the string regardless of
   /// character class
   pub fn charlen(&self) -> usize
   {
      let v:Vec<_> = self.to_str().chars().collect();  v.len()
   }

   /// returns the nth char of the fstr
   pub fn nth(&self,n:usize) -> Option<char>
   {
      self.to_str().chars().nth(n)   
   }

   /// shortens the fstr in-place (mutates).  If n is greater than the
   /// current length of the string, this operation will have no effect.
   pub fn truncate(&mut self, n:usize)
   {
     if let Some((bi,c)) = self.to_str().char_indices().nth(n) {
        self.chrs[bi] = 0;
     }
     //if n<self.len {self.len = n;}
   }
}//impl fstr<N>

/*
impl<'t, const N:usize> std::convert::Into<&'t str> for fstr<N>
{
  fn into(self) -> &'t str
  {
     std::str::from_utf8(&self.chrs[0..self.len]).unwrap()
  }
}
*/

impl<const N:usize> std::convert::From<&str> for fstr<N>
{
  /// creates a new fstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&str) -> fstr<N>
  {
     fstr::make(s)
  }
}
impl<const N:usize> std::convert::From<&mut str> for fstr<N>
{
  /// creates a new fstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&mut str) -> fstr<N>
  {
     fstr::make(s)
  }
}

impl<const N:usize> std::convert::From<&String> for fstr<N>
{
  fn from(s:&String) -> fstr<N>
  {
     fstr::<N>::make(&s[..])
  }
}
impl<const N:usize> std::convert::From<&mut String> for fstr<N>
{
  fn from(s:&mut String) -> fstr<N>
  {
     fstr::<N>::make(&s[..])
  }
}

impl<const N:usize> std::convert::From<String> for fstr<N>
{
  fn from(s:String) -> fstr<N>
  {
     fstr::<N>::make(&s[..])
  }
}

impl<const N:usize,const M:usize> std::convert::From<zstr<M>> for fstr<N>
{
  fn from(s:zstr<M>) -> fstr<N>
  {
     fstr::<N>::make(&s.to_str())
  }
}
impl<const N:usize,const M:usize> std::convert::From<&zstr<M>> for fstr<N>
{
  fn from(s:&zstr<M>) -> fstr<N>
  {
     fstr::<N>::make(&s.to_str())
  }
}

impl<const N:usize> std::cmp::PartialOrd for fstr<N>
{
   fn partial_cmp(&self, other:&Self) -> Option<Ordering>
   {
      //Some(self.chrs[0..self.len].cmp(other.chrs[0..other.len]))
      Some(self.cmp(other))
   }
}

impl<const N:usize> std::cmp::Ord for fstr<N>
{
   fn cmp(&self, other:&Self) -> Ordering
   {
      self.chrs[0..self.len].cmp(&other.chrs[0..other.len])
   }
}

impl<const M:usize> fstr<M>
{
  /// converts an fstr\<M\> to an fstr\<N\>. If the length of the string being
  /// converted is greater than N, the extra characters will be ignored.
  /// This operation produces a copy (non-destructive).
  /// Example:
  ///```ignore
  ///  let s1:fstr<8> = fstr::from("abcdefg");
  ///  let s2:fstr<16> = s1.resize(); 
  ///```
  pub fn resize<const N:usize>(&self) -> fstr<N>
  {
     let length = if (self.len<N) {self.len} else {N};
     let mut chars = [0u8;N];
     for i in 0..length {chars[i] = self.chrs[i];}
     fstr {
       chrs: chars,
       len: length,
     }
  }//resize
/*
   pub fn cat<const N:usize, const MN:usize>(&self,s:fstr<N>) -> fstr<MN>
   {
      let ab:fstr<MN> = self.resize();
      
      let mut i = ab.len;
      for c in s.chars()
      {
         if i<N {self.chrs[i] = c; i+=1;} else {self.len=N; return &s[N..];}
      }
      self.len = i;
      &s[i..]
   }
   */
}//impl fstr<M>

/*   not compatible
impl<const N:usize> std::convert::From<&str> for fstr<N>
{
  fn from(s:&str) -> fstr<N>
  {
     fstr::<N>::make(s)
  }
}
*/

/// [IntoIterator] struct for fstr
pub struct fstriter<const N:usize>
{
   fs : fstr<N>,
   i : usize,
}
impl<const N:usize> Iterator for fstriter<N>
{
   type Item = char;
   fn next(&mut self) -> Option<char>
   {
      if self.i<self.fs.len {
        self.i+=1;
        Some(self.fs.chrs[self.i-1] as char)
      } else {None}
   }
}
impl<const N:usize> IntoIterator for fstr<N>
{
  type Item = char;
  type IntoIter = fstriter<N>;
  fn into_iter(self) -> fstriter<N>
  {
     fstriter {
       fs : self,
       i : 0,
     }
  }
}//IntoIterator

impl<const N:usize> std::fmt::Display for fstr<N>
{
  fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
     write!(f,"{}",self.to_str()) // need change!
  }
}

impl<const N:usize> PartialEq<&str> for fstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     self==other   // see below
  }//eq
}
impl<const N:usize> PartialEq<&str> for &fstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
      &self.to_str() == other
  }//eq
}
impl<'t, const N:usize> PartialEq<fstr<N>> for &'t str
{
  fn eq(&self, other:&fstr<N>) -> bool
  { &other.to_str()==self }
}
impl<'t, const N:usize> PartialEq<&fstr<N>> for &'t str
{
  fn eq(&self, other:&&fstr<N>) -> bool
  { other==self }
}

/// defaults to empty string
impl<const N:usize> Default for fstr<N>
{
   fn default() -> Self { fstr::<N>::make("") }
}

impl<const N:usize> std::fmt::Debug for fstr<N>
{
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let ds = format!("fstr<{}>:\"{}\"",N,&self.to_str());
          f.pad(&ds)
  }
}  // Debug impl


///Convert fstr to &[u8] slice
impl<IndexType,const N:usize> std::ops::Index<IndexType> for fstr<N>
  where IndexType:std::slice::SliceIndex<[u8]>,
{
  type Output = IndexType::Output;
  fn index(&self, index:IndexType)-> &Self::Output
  {
     &self.chrs[index]
  }
}//impl Index
// couldn't get it to work properly, [char] is not same as &str
// because there's no allocated string!

impl<const N:usize> fstr<N>
{

   /// mimics same function on str
   pub fn chars(&self) -> std::str::Chars<'_>
   { self.to_str().chars() }
   /// mimics same function on str
   pub fn char_indices(&self) ->std::str::CharIndices<'_>
   { self.to_str().char_indices() }

  /// returns a copy of the portion of the string, string could be truncated
  /// if indices are out of range. Similar to slice [start..end]
  pub fn substr(&self,start:usize, end:usize) -> fstr<N>
  {
    let mut chars = [0u8;N];
    let mut inds = self.char_indices();
    let len = self.len();
    if start>=len || end<=start {return fstr{chrs:chars, len:0};}
    let (si,_) = inds.nth(start).unwrap();
    let last = if (end>=len) {len} else {
      match inds.nth(end-start-1) {
        Some((ei,_)) => ei,
        None => len,
      }//match
    };//let last =...
    for i in si..last
    {
      chars[i-si] = self.chrs[i];
    }
    fstr { chrs: chars, len:end-start}

/*
    let mut chars = [0u8;N];
    if start>=self.len || end<=start { return fstr{chrs:chars, len:0}; }
    let mut i = start;
    while i<end && i<self.len
    {
       chars[i-start] = self.chrs[i];
       i += 1;
    }
    fstr { chrs: chars, len:i-start }
*/    
  }//substr
}



/// types for small strings 
pub type str8 = fstr<8>; 
pub type str16 = fstr<16>; 
pub type str32 = fstr<32>; 
pub type str64 = fstr<64>; 
pub type str128 = fstr<128>;
