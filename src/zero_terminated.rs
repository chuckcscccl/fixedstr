//! This module implements [zstr], which are zero-terminated strings of
//! fixed maximum lengths.  Compared to [crate::fstr], these strings 
//! are more memory efficient but with some of the operations taking slightly
//! longer. Type zstr\<N\> can store strings consisting of up to N-1 bytes
//! whereas fstr\<N\> can store strings consisting of up to N bytes.
//! Also, it is assumed that the zstr may carray non-textul data and therefore
//! implements some of the traits differently.
//!
//! **`zstr<N>`** also implements the [std::ops::IndexMut] trait for usize.
//! This allows destructive changes to single bytes, such as
//! ```rust
//!   let mut s = <zstr<8>>::from("abcd");
//!   s[0] = b'A';
//!   assert_eq!(&s[0..3],"Abc");
//! ```
//! The consequence of IndexMut is that the buffer may not represent an
//! utf8 string.  In fact, a [zstr::from_raw] method also exists.
//! In constrast, fstr and the alias types str4-str256 do not implement
//! IndexMut.  This distinction of zstr means also means that
//! the [std::ops::Index] traits are separately implemented for the
//! Range types.


#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]
use crate::{fstr, tstr};
use std::cmp::{min, Ordering};
use std::ops::{Range,RangeFull,RangeFrom,RangeTo};
use std::ops::{RangeInclusive,RangeToInclusive};

/// `zstr<N>`: zero-terminated utf8 strings of size up to N bytes.  Note that
/// zstr supports unicode, so that the length of string in characters may
/// be less than N.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct zstr<const N: usize> {
    chrs: [u8; N],
} //zstr
impl<const N: usize> zstr<N> {
    /// creates a new `zstr<N>` with given &str.  If the length of s exceeds
    /// N, the extra characters are ignored and a warning sent to stderr.
    /// This function is also called by
    /// several others including [zstr::from].  This function can now handle
    /// utf8 strings properly.
    pub fn make(s: &str) -> zstr<N> {
        //      if (N>256 || N<1) {panic!("only zstr<1> to zstr<256> are valid");}
        let mut chars = [0u8; N];
        let bytes = s.as_bytes(); // &[u8]
        if (bytes.len() >= N) {
            eprintln!("!Fixedstr Warning in zstr::make: length of string literal \"{}\" exceeds the capacity of type zstr<{}>; string truncated",s,N);
        }
        let mut i = 0;
        let limit = min(N - 1, bytes.len());
        chars[..limit].clone_from_slice(&bytes[..limit]);
        /*
        for i in 0..bytes.len()
        {
          if i<N-1 {chars[i] = bytes[i];} else {break;}
        }
        */
        zstr { chrs: chars }
    } //make

    /// Version of make that does not print warning to stderr.  If the
    /// capacity limit is exceeded, the extra characters are ignored.
    pub fn create(s: &str) -> zstr<N> {
        let mut chars = [0u8; N];
        let bytes = s.as_bytes(); // &[u8]
        let mut i = 0;
        let limit = min(N - 1, bytes.len());
        chars[..limit].clone_from_slice(&bytes[..limit]);
        zstr { chrs: chars }
    } //create

    /// version of make that does not truncate
    pub fn try_make(s: &str) -> Result<zstr<N>, &str> {
        if s.len() > N - 1 {
            Err(s)
        } else {
            Ok(zstr::make(s))
        }
    }

    /// creates an empty string, equivalent to zstr::default()
    pub fn new() -> zstr<N> {
        zstr::make("")
    }


  /// creates a new `zstr<N>` with given u8 slice.  If the length of s exceeds
  /// N, the extra characters are ignored.  The last byte of the array is
  /// is set to 0 to ensure that the string is zero-terminated.  This
  /// operation does not check if the u8 slice is an utf8 source.
  pub fn from_raw(s:&[u8]) -> zstr<N>
  {
     let mut s2 = s;
     if s.len()>N { s2 = &s[..N]; }
     let mut z = zstr {
       chrs: [0;N],
     };
     z.chrs[0..s2.len()].copy_from_slice(s2);
     if (z.chrs.len()>0) {z.chrs[z.chrs.len()-1]=0;}
     z
  }//from_raw



    /// length of the string in bytes (consistent with [str::len]).
    pub fn len(&self) -> usize {
        let mut i = 0;
        while self.chrs[i] != 0 {
            i += 1;
        }
        return i;
        //return std::str::from_utf8(&self.chrs[0..i]).unwrap().len();
    }

    /// returns maximum capacity in bytes
    pub fn capacity(&self) -> usize {
        N - 1
    }

    // returns the byte length of the string, which will be less than N
    fn blen(&self) -> usize {
        let mut i = 0;
        while self.chrs[i] != 0 {
            i += 1;
        }
        return i;
    }

    /// converts zstr to an owned string
    pub fn to_string(&self) -> String {
        let vs: Vec<_> = self.chrs[0..self.blen()].iter().map(|x| *x).collect();
        std::string::String::from_utf8(vs).expect("Invalid utf8 string")
    }

    /// returns slice of u8 array underneath the zstr, including terminating 0
    pub fn as_bytes(&self) -> &[u8] {
        &self.chrs[..self.blen()+1]
    }

    /// converts zstr to &str using [std::str::from_utf8_unchecked].
    /// Since zstr can only be constructed from valid utf8 sources,
    /// this conversion is safe.
    pub fn to_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.chrs[0..self.blen()]) }
    }
    /// checked version of [zstr::to_str], may panic
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.chrs[0..self.blen()]).unwrap()
    }

    /// changes a character at character position i to c.  This function
    /// requires that c is in the same character class (ascii or unicode)
    /// as the char being replaced.  It never shuffles the bytes underneath.
    /// The function returns true if the change was successful.
    pub fn set(&mut self, i: usize, c: char) -> bool {
        let ref mut cbuf = [0u8; 4];
        c.encode_utf8(cbuf);
        let clen = c.len_utf8();
        if let Some((bi, rc)) = self.as_str().char_indices().nth(i) {
            if clen == rc.len_utf8() {
                self.chrs[bi..bi + clen].clone_from_slice(&cbuf[..clen]);
                //for k in 0..clen {self.chrs[bi+k] = cbuf[k];}
                return true;
            }
        }
        return false;
    } //set
    /// adds chars to end of current string up to maximum size N of `zstr<N>`,
    /// returns the portion of the push string that was NOT pushed due to
    /// capacity, so
    /// if "" is returned then all characters were pushed successfully.
    pub fn push<'t>(&mut self, s: &'t str) -> &'t str {
        if s.len() < 1 {
            return s;
        }
        let mut buf = [0u8; 4];
        let mut i = self.blen();
        let mut sci = 0; // indexes characters in s
        for c in s.chars() {
            let clen = c.len_utf8();
            c.encode_utf8(&mut buf);
            if i <= N - clen - 1 {
                self.chrs[i..i + clen].clone_from_slice(&buf[..clen]);
                /*
                for k in 0..clen
                {
                  self.chrs[i+k] = buf[k];
                }
                */
                i += clen;
            } else {
                self.chrs[i] = 0;
                return &s[sci..];
            }
            sci += 1;
        }
        if i < N {
            self.chrs[i] = 0;
        } // zero-terminate
        &s[sci..]
    } //push

    /// returns the number of characters in the string regardless of
    /// character class
    pub fn charlen(&self) -> usize {
        let v: Vec<_> = self.as_str().chars().collect();
        v.len()
    }

    /// returns the nth char of the zstr
    pub fn nth(&self, n: usize) -> Option<char> {
        self.as_str().chars().nth(n)
        //if n<self.len() {Some(self.chrs[n] as char)} else {None}
    }

    /// returns the nth byte of the string as a char.  This
    /// function should only be called on ascii strings.  It
    /// is designed to be quicker than [zstr::nth], and does not check array bounds or
    /// check n against the length of the string. Nor does it check
    /// if the value returned is within the ascii range.
    pub fn nth_ascii(&self, n: usize) -> char {
        self.chrs[n] as char
    }

    /// determines if string is an ascii string
    pub fn is_ascii(&self) -> bool {
        self.as_str().is_ascii()
    }

    /// shortens the zstr in-place (mutates).  If n is greater than the
    /// current length of the string, this operation will have no effect.
    pub fn truncate(&mut self, n: usize) // n is char position, not binary position
    {
        if let Some((bi, c)) = self.as_str().char_indices().nth(n) {
            self.chrs[bi] = 0;
        }
    }
    /*
    /// mimics same function on str
    pub fn chars(&self) -> std::str::Chars<'_> {
        self.as_str().chars()
    }
    /// mimics same function on str
    pub fn char_indices(&self) -> std::str::CharIndices<'_> {
        self.as_str().char_indices()
    }
    */
    
    /// in-place modification of ascii characters to lower-case
    pub fn make_ascii_lowercase(&mut self) {
      for b in &mut self.chrs {
        if *b==0 {break;}
        else if *b>=65 && *b<=90 { *b += 32; }
      }
    }//make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case
    pub fn make_ascii_uppercase(&mut self) {
      for b in &mut self.chrs {
        if *b==0 {break;}
        else if *b>=97 && *b<=122 { *b -= 32; }
      }      
    }

    /// Constructs a clone of this fstr but with only upper-case ascii
    /// characters.  This contrasts with [str::to_ascii_uppercase],
    /// which creates an owned String. 
    pub fn to_ascii_upper(&self) -> Self
    {
      let mut cp = self.clone();
      cp.make_ascii_uppercase();
      cp
    }

    /// Constructs a clone of this fstr but with only lower-case ascii
    /// characters.  This contrasts with [str::to_ascii_lowercase],
    /// which creates an owned String.
    pub fn to_ascii_lower(&self) -> Self
    {
      let mut cp = *self;
      cp.make_ascii_lowercase();
      cp
    }

} //impl zstr<N>

impl<const N:usize> std::ops::Deref for zstr<N>
{
    type Target = str;
    fn deref(&self) -> &Self::Target {
      self.to_str()
    }
}

impl<const N: usize> std::convert::AsRef<str> for zstr<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl<const N: usize> std::convert::AsMut<str> for zstr<N> {
    fn as_mut(&mut self) -> &mut str {
        let blen = self.blen();
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.chrs[0..blen]) }
    }
}

impl<T: AsRef<str> + ?Sized, const N: usize> std::convert::From<&T> for zstr<N> {
    fn from(s: &T) -> zstr<N> {
        zstr::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> std::convert::From<&mut T> for zstr<N> {
    fn from(s: &mut T) -> zstr<N> {
        zstr::make(s.as_mut())
    }
}

/*
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
impl<const N:usize,const M:usize> std::convert::From<&fstr<M>> for zstr<N>
{
  fn from(s:&fstr<M>) -> zstr<N>
  {
     zstr::<N>::make(&s.to_str())
  }
}

impl<const N:usize,const M:usize> std::convert::From<&tstr<M>> for zstr<N>
{
  fn from(s:&tstr<M>) -> zstr<N>
  {
     zstr::<N>::make(&s.to_str())
  }
}

*/


impl<const N: usize> std::convert::From<String> for zstr<N> {
    fn from(s: String) -> zstr<N> {
        zstr::<N>::make(&s[..])
    }
}

impl<const N: usize, const M: usize> std::convert::From<fstr<M>> for zstr<N> {
    fn from(s: fstr<M>) -> zstr<N> {
        zstr::<N>::make(s.to_str())
    }
}

impl<const N: usize, const M: usize> std::convert::From<tstr<M>> for zstr<N> {
    fn from(s: tstr<M>) -> zstr<N> {
        zstr::<N>::make(s.to_str())
    }
}

impl<const N: usize> std::cmp::PartialOrd for zstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.chrs[0..self.blen()].cmp(other.chrs[0..other.blen()]))
        Some(self.cmp(other))
    }
}

impl<const N: usize> std::cmp::Ord for zstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.chrs[0..self.blen()].cmp(&other.chrs[0..other.blen()])
    }
}

impl<const M: usize> zstr<M> {
    /// converts an zstr\<M\> to an zstr\<N\>. If the length of the string being
    /// converted is greater than N, the extra characters are ignored and
    /// a warning sent to stderr.
    /// This operation produces a copy (non-destructive).
    /// Example:
    ///```ignore
    ///  let s1:zstr<8> = zstr::from("abcdefg");
    ///  let s2:zstr<16> = s1.resize();
    ///```
    pub fn resize<const N: usize>(&self) -> zstr<N> {
        let slen = self.blen();
        //if (slen>=N) {eprintln!("!Fixedstr Warning in zstr::resize: string \"{}\" truncated while resizing to zstr<{}>",self,N);}
        let length = if (slen < N - 1) { slen } else { N - 1 };
        let mut chars = [0u8; N];
        chars[..length].clone_from_slice(&self.chrs[..length]);
        //for i in 0..length {chars[i] = self.chrs[i];}
        zstr { chrs: chars }
    } //resize

    /// version of resize that does not allow string truncation due to length
    pub fn reallocate<const N: usize>(&self) -> Option<zstr<N>> {
        if self.len() < N {
            Some(self.resize())
        } else {
            None
        }
    }
} //impl zstr<M>

impl<const N: usize> std::fmt::Display for zstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const N: usize> PartialEq<&str> for zstr<N> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other // see below
    } //eq
}
impl<const N: usize> PartialEq<&str> for &zstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.as_str() == other
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
    } //eq
}
impl<'t, const N: usize> PartialEq<zstr<N>> for &'t str {
    fn eq(&self, other: &zstr<N>) -> bool {
        &other.as_str() == self
    }
}
impl<'t, const N: usize> PartialEq<&zstr<N>> for &'t str {
    fn eq(&self, other: &&zstr<N>) -> bool {
        &other.as_str() == self
    }
}

/// defaults to empty string
impl<const N: usize> Default for zstr<N> {
    fn default() -> Self {
        zstr::<N>::make("")
    }
}

impl<const N: usize, const M: usize> PartialEq<zstr<N>> for fstr<M> {
    fn eq(&self, other: &zstr<N>) -> bool {
        other.as_str() == self.to_str()
    }
}
/*
impl<const N:usize, const M:usize> PartialEq<&zstr<N>> for fstr<M>
{
  fn eq(&self, other:&&zstr<N>) -> bool
  { other.as_str()==self.to_str() }
}
*/
impl<const N: usize, const M: usize> PartialEq<fstr<N>> for zstr<M> {
    fn eq(&self, other: &fstr<N>) -> bool {
        other.to_str() == self.as_str()
    }
}
impl<const N: usize, const M: usize> PartialEq<&fstr<N>> for zstr<M> {
    fn eq(&self, other: &&fstr<N>) -> bool {
        other.to_str() == self.as_str()
    }
}

impl<const N: usize> std::fmt::Debug for zstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ds = format!("zstr<{}>:\"{}\"", N, &self.as_str());
        f.pad(&ds)
        //        f.debug_struct("zstr")
        //         .field("chrs:",&self.to_str())
        //         .finish()
    }
} // Debug impl

impl<const N: usize> zstr<N> {
    /// returns a copy of the portion of the string, string could be truncated
    /// if indices are out of range. Similar to slice [start..end]
    pub fn substr(&self, start: usize, end: usize) -> zstr<N> {
        let mut chars = [0u8; N];
        let mut inds = self.char_indices();
        let len = self.len();
        let blen = self.blen();
        if start >= len || end <= start {
            return zstr { chrs: chars };
        }
        let (si, _) = inds.nth(start).unwrap();
        let last = if (end >= len) {
            blen
        } else {
            match inds.nth(end - start - 1) {
                Some((ei, _)) => ei,
                None => blen,
            } //match
        }; //let last =...
        chars[..last - si].clone_from_slice(&self.chrs[si..last]);
        /*
        for i in si..last
        {
          chars[i-si] = self.chrs[i];
        }
        */
        zstr { chrs: chars }
    } //substr
}

/// types for small strings
pub type ztr8 = zstr<8>;
pub type ztr16 = zstr<16>;
pub type ztr32 = zstr<32>;
pub type ztr64 = zstr<64>;

////////////// std::fmt::Write trait
/// Usage:
/// ```
///   use std::fmt::Write;
///   let mut s = zstr::<32>::new();
///   let result = write!(&mut s,"hello {}, {}, {}",1,2,3);
///   /* or */
///   let s2 = str_format!(zstr<16>,"abx{}{}{}",1,2,3);
/// ```
impl<const N: usize> std::fmt::Write for zstr<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result //Result<(),std::fmt::Error>
    {
        if s.len() + self.len() > N - 1 {
            return Err(std::fmt::Error::default());
        }
        self.push(s);
        Ok(())
    } //write_str
} //std::fmt::Write trait



/*
///Convert zstr to &str
impl<IndexType, const N: usize> std::ops::Index<IndexType> for zstr<N>
where
    IndexType: std::slice::SliceIndex<str>,
{
    type Output = IndexType::Output;
    fn index(&self, index: IndexType) -> &Self::Output {
        &self.as_str()[index]
    }
} //impl Index
*/


///The implementation of `Index<usize>` for types `zstr<N>` is different
///from that of `fstr<N>` and `tstr<N>`, to allow `IndexMut` on a single
///byte.  The type returned by this trait is &u8, not &str.
impl<const N:usize> std::ops::Index<usize> for zstr<N>
{
  type Output = u8;
  fn index(&self, index:usize)-> &Self::Output
  {
     &self.chrs[index]
  }
}//impl Index
impl<const N:usize> std::ops::IndexMut<usize> for zstr<N>
{
  fn index_mut(&mut self, index:usize)-> &mut Self::Output
  {
     &mut self.chrs[index]
  }
}//impl Index


impl<const N:usize> std::ops::Index<Range<usize>> for zstr<N> {
  type Output = str;
  fn index(&self, index:Range<usize>)-> &Self::Output  {
     &self.as_str()[index]
  }
}//impl Index
impl<const N:usize> std::ops::Index<RangeTo<usize>> for zstr<N> {
  type Output = str;
  fn index(&self, index:RangeTo<usize>)-> &Self::Output  {
     &self.as_str()[index]
  }
}//impl Index
impl<const N:usize> std::ops::Index<RangeFrom<usize>> for zstr<N> {
  type Output = str;
  fn index(&self, index:RangeFrom<usize>)-> &Self::Output  {
     &self.as_str()[index]
  }
}//impl Index
impl<const N:usize> std::ops::Index<RangeInclusive<usize>> for zstr<N> {
  type Output = str;
  fn index(&self, index:RangeInclusive<usize>)-> &Self::Output  {
     &self.as_str()[index]
  }
}//impl Index
impl<const N:usize> std::ops::Index<RangeToInclusive<usize>> for zstr<N> {
  type Output = str;
  fn index(&self, index:RangeToInclusive<usize>)-> &Self::Output  {
     &self.as_str()[index]
  }
}//impl Index
impl<const N:usize> std::ops::Index<RangeFull> for zstr<N> {
  type Output = str;
  fn index(&self, index:RangeFull)-> &Self::Output  {
     &self.as_str()[index]
  }
}//impl Index
