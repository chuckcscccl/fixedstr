//! Library for strings of fixed maximum lengths that can be copied and
//! stack-allocated using const generics.
//!
//! **The structures provided by this crate are [fstr], [zstr]** and tstr.
//! However, tstr is not exported and can only be used through the type
//! aliases [str8], [str16], [str32], through [str256].
//!
//! **Version 0.2.x** adds **unicode support** and a module for
//! **zero_terminated strings** in the structure [zstr].
//! These strings are more memory efficient than [fstr] but less efficient
//! in terms of run time.
//! 
//! For version 0.2.2, the [str8] through [str256] type aliases where
//! changed to refer to another, internal type distinct from [fstr] and
//! [zstr].  This type represents strings of up to 255 bytes with a
//! `[u8;N]` underneath where it's assumed that N<=256.  The first
//! byte of the array holds the length of the string in bytes.  This structure
//! represents the best combination of fstr and zstr in terms of speed
//! and memory efficiency.  However, because Rust does not currently provide
//! a why to specify conditions on const generics at compile time, such as
//! `where N<=256`, the internal "tiny string" type *tstr* is not exported and can
//! only be used through the aliases.  These strings implement that same
//! functions and traits as [fstr] and [zstr] so the documentation for
//! these structures also apply to the hidden type.
//!
//! For version 0.2.2  the fsiter construct and direct iterator
//! implmentation for fstr has been removed. Use the [fstr::chars]
//! function instead.

//!  ## Examples
//!
//!```
//! let a:fstr<8> = fstr::from("abcdefg"); //creates fstr from &str
//! let a1:fstr<8> = a; // copied, not moved
//! let a2:&str = a.to_str();
//! let a3:String = a.to_string();
//! assert_eq!(a.nth_ascii(2), 'c');
//! let ab = a.substr(1,5);  // copies substring to new fstr
//! assert_eq!(ab,"bcde");  // can compare with &str
//! assert!(a<ab);  // implements Ord trait (and Hash, Debug, Display)
//! let mut u:fstr<8> = fstr::from("aλb"); //unicode support
//! for x in u.nth(1) {assert_eq!(x,'λ');} // nth returns Option<char>
//! assert!(u.set(1,'μ'));  // changes a character of the same character class
//! assert!(!u.set(1,'c')); // .set returns false on failure
//! assert!(u.set(2,'c'));
//! assert_eq!(u, "aμc");
//! assert_eq!(u.len(),4);  // length in bytes
//! assert_eq!(u.charlen(),3);  // length in chars
//! let mut ac:fstr<16> = a.resize(); // copies to larger capacity string
//! let remainder:&str = ac.push("hijklmnopqrst");  //appends string, returns left over
//! assert_eq!(ac.len(),16);
//! assert_eq!(remainder, "qrst");
//! ac.truncate(10); // shortens string in place
//! assert_eq!(&ac,"abcdefghij");
//!```
//!
//![zstr] and the type aliases [str8]...[str256] implement the same capabilities as [fstr].



#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]

pub mod zero_terminated;
pub use zero_terminated::*;
mod tiny_internal;
use tiny_internal::*;

use std::cmp::Ordering;

/// main type: string of size up to const N:
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
//      if (N>65536 || N<1) {panic!("Valid fstr strings are limited to fstr<1> to zstr<65536>");}
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

   /// returns the nth byte of the string as a char.  This
   /// function should only be called on ascii strings.  It
   /// is designed to be quicker than [fstr::nth], and does not check array bounds or
   /// check n against the length of the string. Nor does it check
   /// if the value returned is within the ascii range.
   pub fn nth_ascii(&self,n:usize) -> char
   {
      self.chrs[n] as char
   }

   /// shortens the fstr in-place (mutates).  If n is greater than the
   /// current length of the string in chars, this operation will have no effect.
   pub fn truncate(&mut self, n:usize)
   {
     if let Some((bi,c)) = self.to_str().char_indices().nth(n) {
        //self.chrs[bi] = 0;
        self.len = bi;
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

impl<const N:usize,const M:usize> std::convert::From<tstr<M>> for fstr<N>
{
  fn from(s:tstr<M>) -> fstr<N>
  {
     fstr::<N>::make(&s.to_str())
  }
}
impl<const N:usize,const M:usize> std::convert::From<&tstr<M>> for fstr<N>
{
  fn from(s:&tstr<M>) -> fstr<N>
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
/*
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
*/

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
     &self.to_str()==other   // see below
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
  { &other.to_str()==self }
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



/// types for small strings that use a more efficient representation
/// underneath.  A str8 can hold a string of up to 7 bytes (7 ascii chars).
/// The same functions for [fstr] and [zstr] are provided for these types.
pub type str8 = tstr<8>;
/// A str16 can hold a string of up to 15 bytes. See docs for [fstr] or [zstr]
pub type str16 = tstr<16>;
/// A str16 can hold a string of up to 31 bytes. See docs for [fstr] or [zstr]
pub type str32 = tstr<32>;
/// A str16 can hold a string of up to 63 bytes. See docs for [fstr] or [zstr]
pub type str64 = tstr<64>;
/// A str16 can hold a string of up to 127 bytes. See docs for [fstr] or [zstr]
pub type str128 = tstr<128>;
/// Each type strN is represented underneath by a `[u8;N]` with N<=256.
/// The first byte of the array always holds the length of the string.
/// Each such type can hold a string of up to N-1 bytes, with max size=255.
/// These types represent the best compromise between [fstr] and [zstr] in
/// terms of speed and memory efficiency.
pub type str256 = tstr<256>;
