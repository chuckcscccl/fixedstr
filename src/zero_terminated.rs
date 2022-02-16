//! This module implements [zstr], which are zero-terminated strings of
//! fixed lengths up to 255 characters.  Compared to [crate::fstr], zstr
//! are more memory efficient but with some of the operations taking slightly
//! longer.


#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]
use crate::fstr;
use std::cmp::Ordering;

/// zstr<N>: zero-terminated utf8 strings of size up to N bytes.  Note that
/// zstr supports unicode, so that the length of string in characters may
/// be less than N.
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct zstr<const N:usize>
{
  chrs : [u8;N],
}//zstr
impl<const N:usize> zstr<N>
{
  /// creates a new zstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.  This function is also called by
  /// several others including [zstr::from].  This function can now handle
  /// utf8 strings properly.
  pub fn make(s:&str) -> zstr<N>
   {
      let mut chars = [0u8; N];
      let bytes = s.as_bytes(); // &[u8]
      let mut i = 0;
      for i in 0..bytes.len()
      {
        if i+1<N {chars[i] = bytes[i];} else {break;}
      }
      /*
      for c in s.chars()
      {
         if i<N { chars[i] = c as u8; i+=1; } else {break;}
      }
      */
      zstr {
         chrs: chars,
      }
   }//make

   /// creates an empty string, equivalent to zstr::default()
   pub fn new() -> zstr<N>
   {
     zstr::make("")
   }

   /// length of the string in chars. This
   /// is a constant-time operation.
   pub fn len(&self)->usize {
     let mut i =0;
     while self.chrs[i]!=0 {i+=1;}
     return std::str::from_utf8(&self.chrs[0..i]).unwrap().len();
   }

   /// returns the byte length of the string, which will be less than N
   pub fn blen(&self)->usize {
     let mut i =0;
     while self.chrs[i]!=0 {i+=1;}
     return i;
   }

   /// converts zstr to an owned string
   pub fn to_string(&self) -> String
   {
     let vs:Vec<_> = self.chrs[0..self.blen()].iter().map(|x|{*x}).collect();
     std::string::String::from_utf8(vs).expect("Invalid utf8 string")
   }

   /// returns copy of u8 array underneath the zstr
   pub fn as_bytes(&self) -> &[u8]
   {
      &self.chrs[..self.blen()]
   }

   /// converts zstr to &str using [std::str::from_utf8]
   pub fn to_str(&self) -> &str
   {
      std::str::from_utf8(&self.chrs[0..self.blen()]).unwrap()
   }
   /// alias for [zstr::to_str]
   pub fn as_str(&self) -> &str {self.to_str()}

   /// changes a character at character position i to c.  This function
   /// requires that c is in the same character class (ascii or unicode)
   /// as the char being replaced.  It never shuffles the bytes underneath.
   /// The function returns true if the change was successful.
   pub fn set(&mut self,i:usize, c:char) -> bool
   {
      let ref mut cbuf = [0u8;4];
      c.encode_utf8(cbuf);
      let clen = c.len_utf8();
      if let Some((bi,rc)) = self.to_str().char_indices().nth(i) {
        if clen==rc.len_utf8() {
           for k in 0..clen {self.chrs[bi+k] = cbuf[k];}
           return true;
        }
      }
      return false;
   }//set
   /// adds chars to end of current string up to maximum size N of zstr<N>,
   /// returns the portion of the push string that was NOT pushed due to
   /// capacity, so
   /// if "" is returned then all characters were pushed successfully.
   pub fn push<'t>(&mut self,s:&'t str) -> &'t str
   {
      let mut buf = [0u8;4];
      let mut i = self.blen();
      let mut sci = 0; // indexes characters in s
      for c in s.chars()
      {
         let clen = c.len_utf8();
         c.encode_utf8(&mut buf);
         if i<=N-clen-1 {
           for k in 0..clen
           {
             self.chrs[i+k] = buf[k];
           }
           self.chrs[i+clen] = 0;
         } else { return &s[sci..];}
         sci += 1;
      }
      &s[sci..]
   }//push

   /// returns the nth char of the zstr
   pub fn nth(&self,n:usize) -> Option<char>
   {
      self.to_str().chars().nth(n)
      //if n<self.len() {Some(self.chrs[n] as char)} else {None}
   }

   /// shortens the zstr in-place (mutates).  If n is greater than the
   /// current length of the string, this operation will have no effect.
   pub fn truncate(&mut self, n:usize) // n is char position, not binary position
   {
     if let Some((bi,c)) = self.to_str().char_indices().nth(n) {
        self.chrs[bi] = 0;
     }
   }

   pub fn chars(&self) -> std::str::Chars<'_>
   { self.to_str().chars() }

   pub fn char_indices(&self) ->std::str::CharIndices<'_>
   { self.to_str().char_indices() }
}//impl zstr<N>


impl<const N:usize> std::convert::From<&str> for zstr<N>
{
  /// creates a new zstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&str) -> zstr<N>
  {
     zstr::make(s)
  }
}

impl<const N:usize> std::convert::From<&mut str> for zstr<N>
{
  /// creates a new zstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&mut str) -> zstr<N>
  {
     zstr::make(s)
  }
}

impl<const N:usize> std::convert::From<&String> for zstr<N>
{
  fn from(s:&String) -> zstr<N>
  {
     zstr::<N>::make(&s[..])
  }
}
impl<const N:usize> std::convert::From<&mut String> for zstr<N>
{
  fn from(s:&mut String) -> zstr<N>
  {
     zstr::<N>::make(&s[..])
  }
}

impl<const N:usize> std::convert::From<String> for zstr<N>
{
  fn from(s:String) -> zstr<N>
  {
     zstr::<N>::make(&s[..])
  }
}

impl<const N:usize,const M:usize> std::convert::From<fstr<M>> for zstr<N>
{
  fn from(s:fstr<M>) -> zstr<N>
  {
     zstr::<N>::make(s.to_str())
  }
}

impl<const N:usize> std::cmp::PartialOrd for zstr<N>
{
   fn partial_cmp(&self, other:&Self) -> Option<Ordering>
   {
      //Some(self.chrs[0..self.blen()].cmp(other.chrs[0..other.blen()]))
      Some(self.cmp(other))
   }
}

impl<const N:usize> std::cmp::Ord for zstr<N>
{
   fn cmp(&self, other:&Self) -> Ordering
   {
      self.chrs[0..self.blen()].cmp(&other.chrs[0..other.blen()])
   }
}

impl<const M:usize> zstr<M>
{
  /// converts an zstr\<M\> to an zstr\<N\>. If the length of the string being
  /// converted is greater than N, the extra characters will be ignored.
  /// This operation produces a copy (non-destructive).
  /// Example:
  ///```ignore
  ///  let s1:zstr<8> = zstr::from("abcdefg");
  ///  let s2:zstr<16> = s1.resize(); 
  ///```
  pub fn resize<const N:usize>(&self) -> zstr<N>
  {
     let slen = self.blen();
     let length = if (slen<N-1) {slen} else {N-1};
     let mut chars = [0u8;N];
     for i in 0..length {chars[i] = self.chrs[i];}
     zstr {
       chrs: chars,
     }
  }//resize
}//impl zstr<M>

/*  OUT
/// [IntoIterator] struct for zstr
pub struct zstriter<const N:usize>
{
   fs : zstr<N>,
   len: usize, // char length
   i : usize,
}
impl<const N:usize> Iterator for zstriter<N>
{
   type Item = char;
   fn next(&mut self) -> Option<char>
   {
      if self.i<self.len {
        self.i+=1;
        self.to_str().chars().nth(self.i-1)
      } else {None}
   }
}
impl<const N:usize> IntoIterator for zstr<N>
{
  type Item = char;
  type IntoIter = zstriter<N>;
  fn into_iter(self) -> zstriter<N>
  {
     zstriter {
       fs : self,
       len: self.len(),
       i : 0,
     }
  }
}//IntoIterator
*/


impl<const N:usize> std::fmt::Display for zstr<N>
{
  fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
     write!(f,"{}",self.to_str()) 
  }
}

impl<const N:usize> PartialEq<&str> for zstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     &self.to_str()==other   // see below
  }//eq
}
impl<const N:usize> PartialEq<&str> for &zstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     &self.to_str() == other
/*
     let obytes = other.as_bytes();
     let olen = obytes.len();
     let blen = self.blen();
     if olen!=blen {return false;}
     for i in 0..olen
     {
        if obytes[i] != self.chrs[i] {return false;}
     }
     return true;
   */
  }//eq
}
impl<'t, const N:usize> PartialEq<zstr<N>> for &'t str
{
  fn eq(&self, other:&zstr<N>) -> bool
  { &other.to_str()==self }
}
impl<'t, const N:usize> PartialEq<&zstr<N>> for &'t str
{
  fn eq(&self, other:&&zstr<N>) -> bool
  { &other.to_str()==self }
}

/// defaults to empty string
impl<const N:usize> Default for zstr<N>
{
   fn default() -> Self { zstr::<N>::make("") }
}

impl<const N:usize,const M:usize> PartialEq<zstr<N>> for fstr<M>
{
  fn eq(&self, other:&zstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize, const M:usize> PartialEq<&zstr<N>> for fstr<M>
{
  fn eq(&self, other:&&zstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize,const M:usize> PartialEq<fstr<N>> for zstr<M>
{
  fn eq(&self, other:&fstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize, const M:usize> PartialEq<&fstr<N>> for zstr<M>
{
  fn eq(&self, other:&&fstr<N>) -> bool
  { other.to_str()==self.to_str() }
}


///Convert zstr to &[u8] slice
impl<IndexType,const N:usize> std::ops::Index<IndexType> for zstr<N>
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

impl<const N:usize> zstr<N>
{
  /// returns a copy of the portion of the string, string could be truncated
  /// if indices are out of range. Similar to slice [start..end]
  pub fn substr(&self,start:usize, end:usize) -> zstr<N>
  {
    let mut chars = [0u8;N];
    let mut inds = self.char_indices();
    let len = self.len();
    let blen = self.blen();
    if start>=len || end<=start {return zstr{chrs:chars};}
    let (si,_) = inds.nth(start).unwrap();
    let last = if (end>=len) {blen} else {
      match inds.nth(end-start-1) {
        Some((ei,_)) => ei,
        None => blen,
      }//match
    };//let last =...
    for i in si..last
    {
      chars[i-si] = self.chrs[i];
    }
    zstr { chrs: chars,}
  }//substr
}

/// types for small strings 
pub type ztr8 = zstr<8>; 
pub type ztr16 = zstr<16>; 
pub type ztr32 = zstr<32>; 
pub type ztr64 = zstr<64>; 
