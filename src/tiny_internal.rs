//! This module is intended for internal use only.  Only a few
//! type aliases are exported.  A tiny string or tstr<N>, with N<=256,
//! is a version of fixed str that represents the best compromise between
//! memory and runtime efficiency.  Each tstr<N> can hold a string of up to
//! N-1 bytes, with max N=256.  A tstr<N> is represented underneath
//! by a `[u8;N]` with the first byte always representing the length of the
//! string.  A tstr is not necessarily zero-terminated.
//! Because currently Rust does not allow conditions on const generics
//! such as `where N<=256`, this type is not fully exported and one can
//! only use the type aliases.

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]
use crate::fstr;
use crate::zstr;
use std::cmp::Ordering;

/// **THIS STRUCTURE IS NOT EXPORTED.**  It can only be used through the
/// public type aliases [crate::str8] through [crate::str256].
#[derive(Copy,Clone,Eq,PartialEq,Hash)]
pub struct tstr<const N:usize=256> 
{
  chrs : [u8;N],
}//tstr
impl<const N:usize> tstr<N>  
{
  /// creates a new tstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.  This function is also called by
  /// several others including [tstr::from].  This function can now handle
  /// utf8 strings properly.
  pub fn make(s:&str) -> tstr<N>
   {
      if (N>256 || N<1) {panic!("only tstr<1> to tstr<256> are valid");}
      let mut chars = [0u8; N];
      let bytes = s.as_bytes(); // &[u8]
      let blen = bytes.len();
      for i in 0..blen
      {
        if i<N-1 {chars[i+1] = bytes[i];}
        else { chars[0] = i as u8; break; }
      }
      if chars[0]==0 {chars[0]=blen as u8;}
      tstr {
         chrs: chars,
      }
   }//make

   /// creates an empty string, equivalent to tstr::default()
   pub fn new() -> tstr<N>
   {
     tstr::make("")
   }

   /// length of the string in bytes (consistent with [str::len]). This
   /// is a constant-time operation.
   pub fn len(&self)->usize {
      self.chrs[0] as usize
   }

   /// converts tstr to an owned string
   pub fn to_string(&self) -> String
   {
     let vs:Vec<_> = self.chrs[1..self.len()+1].iter().map(|x|{*x}).collect();
     std::string::String::from_utf8(vs).expect("Invalid utf8 string")
   }

   /// returns copy of u8 array underneath the tstr
   pub fn as_bytes(&self) -> &[u8]
   {
      &self.chrs[1..self.len()+1]
   }

   /// converts tstr to &str using [std::str::from_utf8]
   pub fn to_str(&self) -> &str
   {
      std::str::from_utf8(&self.chrs[1..self.len()+1]).unwrap()
   }
   /// alias for [tstr::to_str]
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
           for k in 0..clen {self.chrs[bi+k+1] = cbuf[k];}
           return true;
        }
      }
      return false;
   }//set
   /// adds chars to end of current string up to maximum size N of tstr<N>,
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
         if i<=N-clen-1 {
           for k in 0..clen
           {
             self.chrs[i+k+1] = buf[k];
           }
           i += clen;
         } else { self.chrs[0]=i as u8; return &s[sci..];}
         sci += 1;
      }
      if i<N {self.chrs[0]=i as u8;} // set length
      &s[sci..]
   }//push

   /// returns the number of characters in the string regardless of
   /// character class
   pub fn charlen(&self) -> usize
   {
      let v:Vec<_> = self.to_str().chars().collect();  v.len()
   }

   /// returns the nth char of the tstr
   pub fn nth(&self,n:usize) -> Option<char>
   {
      self.to_str().chars().nth(n)
   }

   /// returns the nth byte of the string as a char.  This
   /// function should only be called on ascii strings.  It
   /// is designed to be quicker than [tstr::nth], and does not check array bounds or
   /// check n against the length of the string. Nor does it check
   /// if the value returned is within the ascii range.
   pub fn nth_ascii(&self,n:usize) -> char
   {
      self.chrs[n+1] as char
   }


   /// shortens the tstr in-place (mutates).  If n is greater than the
   /// current length of the string, this operation will have no effect.
   pub fn truncate(&mut self, n:usize) // n is char position, not binary position
   {
     if let Some((bi,c)) = self.to_str().char_indices().nth(n) {
        self.chrs[0] = bi as u8;
     }
   }

   /// mimics same function on str
   pub fn chars(&self) -> std::str::Chars<'_>
   { self.to_str().chars() }
   /// mimics same function on str
   pub fn char_indices(&self) ->std::str::CharIndices<'_>
   { self.to_str().char_indices() }
}//impl tstr<N>


impl<const N:usize> std::convert::From<&str> for tstr<N>
{
  /// creates a new tstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&str) -> tstr<N>
  {
     tstr::make(s)
  }
}

impl<const N:usize> std::convert::From<&mut str> for tstr<N>
{
  /// creates a new tstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&mut str) -> tstr<N>
  {
     tstr::make(s)
  }
}

impl<const N:usize> std::convert::From<&String> for tstr<N>
{
  fn from(s:&String) -> tstr<N>
  {
     tstr::<N>::make(&s[..])
  }
}
impl<const N:usize> std::convert::From<&mut String> for tstr<N>
{
  fn from(s:&mut String) -> tstr<N>
  {
     tstr::<N>::make(&s[..])
  }
}

impl<const N:usize> std::convert::From<String> for tstr<N>
{
  fn from(s:String) -> tstr<N>
  {
     tstr::<N>::make(&s[..])
  }
}

impl<const N:usize,const M:usize> std::convert::From<fstr<M>> for tstr<N>
{
  fn from(s:fstr<M>) -> tstr<N>
  {
     tstr::<N>::make(s.to_str())
  }
}

impl<const N:usize,const M:usize> std::convert::From<&fstr<M>> for tstr<N>
{
  fn from(s:&fstr<M>) -> tstr<N>
  {
     tstr::<N>::make(&s.to_str())
  }
}

impl<const N:usize,const M:usize> std::convert::From<zstr<M>> for tstr<N>
{
  fn from(s:zstr<M>) -> tstr<N>
  {
     tstr::<N>::make(s.to_str())
  }
}

impl<const N:usize,const M:usize> std::convert::From<&zstr<M>> for tstr<N>
{
  fn from(s:&zstr<M>) -> tstr<N>
  {
     tstr::<N>::make(&s.to_str())
  }
}

impl<const N:usize> std::cmp::PartialOrd for tstr<N>
{
   fn partial_cmp(&self, other:&Self) -> Option<Ordering>
   {
      //Some(self.chrs[0..self.len()].cmp(other.chrs[0..other.len()]))
      Some(self.cmp(other))
   }
}

impl<const N:usize> std::cmp::Ord for tstr<N>
{
   fn cmp(&self, other:&Self) -> Ordering
   {
      self.chrs[1..self.len()+1].cmp(&other.chrs[1..other.len()+1])
   }
}

impl<const M:usize> tstr<M>
{
  /// converts an tstr\<M\> to an tstr\<N\>. If the length of the string being
  /// converted is greater than N, the extra characters will be ignored.
  /// This operation produces a copy (non-destructive).
  /// Example:
  ///```ignore
  ///  let s1:tstr<8> = tstr::from("abcdefg");
  ///  let s2:tstr<16> = s1.resize(); 
  ///```
  pub fn resize<const N:usize>(&self) -> tstr<N>
  {
     let slen = self.len();
     let length = if (slen<N-1) {slen} else {N-1};
     let mut chars = [0u8;N];
     for i in 0..length {chars[i+1] = self.chrs[i+1];}
     chars[0] = (slen) as u8;
     tstr {
       chrs: chars,
     }
  }//resize
}//impl tstr<M>

impl<const N:usize> std::fmt::Display for tstr<N>
{
  fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
     write!(f,"{}",self.to_str()) 
  }
}

impl<const N:usize> PartialEq<&str> for tstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     self.to_str()==*other   // see below
  }//eq
}
impl<const N:usize> PartialEq<&str> for &tstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     &self.to_str() == other
  }//eq
}
impl<'t, const N:usize> PartialEq<tstr<N>> for &'t str
{
  fn eq(&self, other:&tstr<N>) -> bool
  { &other.to_str()==self }
}
impl<'t, const N:usize> PartialEq<&tstr<N>> for &'t str
{
  fn eq(&self, other:&&tstr<N>) -> bool
  { &other.to_str()==self }
}

/// defaults to empty string
impl<const N:usize> Default for tstr<N>
{
   fn default() -> Self { tstr::<N>::make("") }
}

impl<const N:usize,const M:usize> PartialEq<tstr<N>> for fstr<M>
{
  fn eq(&self, other:&tstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize, const M:usize> PartialEq<&tstr<N>> for fstr<M>
{
  fn eq(&self, other:&&tstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize,const M:usize> PartialEq<fstr<N>> for tstr<M>
{
  fn eq(&self, other:&fstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize, const M:usize> PartialEq<&fstr<N>> for tstr<M>
{
  fn eq(&self, other:&&fstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize,const M:usize> PartialEq<zstr<N>> for tstr<M>
{
  fn eq(&self, other:&zstr<N>) -> bool
  { other.to_str()==self.to_str() }
}
impl<const N:usize, const M:usize> PartialEq<&zstr<N>> for tstr<M>
{
  fn eq(&self, other:&&zstr<N>) -> bool
  { other.to_str()==self.to_str() }
}

impl<const N:usize> std::fmt::Debug for tstr<N>
{
fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//      let ds = format!("tstr<{}>:\"{}\"",N,&self.to_str());
          f.pad(&self.to_str())
//        f.debug_struct("tstr")
//         .field("chrs:",&self.to_str())
//         .finish()
  }
}  // Debug impl

/*
///Convert tstr to &[u8] slice
impl<IndexType,const N:usize> std::ops::Index<IndexType> for tstr<N>
  where IndexType:std::slice::SliceIndex<[u8]>,
{
  type Output = IndexType::Output;
  fn index(&self, index:IndexType)-> &Self::Output
  {
     &self.chrs[index+1]
  }
}//impl Index
// couldn't get it to work properly, [char] is not same as &str
// because there's no allocated string!
*/

impl<const N:usize> tstr<N>
{
  /// returns a copy of the portion of the string, string could be truncated
  /// if indices are out of range. Similar to slice [start..end]
  pub fn substr(&self,start:usize, end:usize) -> tstr<N>
  {
    let mut chars = [0u8;N];
    let mut inds = self.char_indices();
    let len = self.len();
    if start>=len || end<=start {return tstr{chrs:chars};}
    chars[0] = (end-start) as u8;
    let (si,_) = inds.nth(start).unwrap();
    let last = if (end>=len) {len} else {
      match inds.nth(end-start-1) {
        Some((ei,_)) => ei,
        None => len,
      }//match
    };//let last =...
    for i in si..last
    {
      chars[i-si+1] = self.chrs[i+1];
    }
    tstr { chrs: chars,}
  }//substr
}

