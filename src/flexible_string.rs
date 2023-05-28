//! This module implements **[Flexstr]**, which uses an internal enum
//! to hold either a fixed string of up to a maximum length, or an owned [String].
//! The structure satisfies the following axiom:
//! >   *For N <= 256, a `Flexstr<N>` is represented internally by an
//!     owned String if and only if the length of the string is greater than
//!     or equal to N*.
//!
//! For example, a `Flexstr<16>` will hold a string of up to 15 bytes 
//! in an u8-array of size 16. The first byte of the array holds the length of
//! the string.  If subsequent operations such as [Flexstr::push_str]
//! extends the array past 15 bytes, the representation will switch to an owned
//! String.  Conversely, an operation such as [Flexstr::truncate]
//! may switch the representation back to a fixed string.
//! The default N is 32.  **The largest N for which the axiom holds
//! is 256.**  For all N>256, the internal representation is always an owned
//! string.
//!
//! Example:
//! ```
//!  let mut s:Flexstr<8> = Flexstr::from("abcdef");
//!  assert!(s.is_fixed());
//!  s.push_str("ghijk");
//!  assert!(s.is_owned());
//!  s.truncate(7);
//!  assert!(s.is_fixed());
//! ```
//!
//! The intended use of this datatype is for
//! situations when the lengths of strings are *usually* less than N, with
//! only occasional exceptions that require a different representation.
//! However, unlike the other string types in this crate, a Flexstr cannot
//! be copied and is thus subject to move semantics.  The serde serialization
//! option is also supported (`features serde`).

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
use crate::fstr;
use crate::zstr;
use crate::tstr;
use crate::{str12, str128, str16, str192, str24, str256, str32, str4, str48, str64, str8, str96};
use std::cmp::{min, Ordering};
use std::ops::Add;
use crate::flexible_string::Strunion::*;

/*
#[derive(Copy,Clone, Eq, PartialEq, Hash)]
enum Strunion_fixed
{
  single(tstr<8>),
  double(tstr<16>),
  quad(tstr<32>),
  octo(tstr<64>),
  hexa(tstr<128>),
}
impl Default for Strunion_fixed {
  fn default() -> Self {
    Strunion_fixed::single(tstr::<8>::default())
  }
}
*/

#[derive(Eq, PartialEq, Hash)]
enum Strunion<const N:usize>
{
   fixed(tstr<N>),
   owned(String),
}//Strunion
impl<const N:usize> Clone for Strunion<N> {
  fn clone(&self) -> Self {
    match &self {
      fixed(s) => fixed(*s),
      owned(s) => owned(s.clone()),
    }//match
  }
}//impl Clone

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Flexstr<const N:usize=32>
{
   inner:Strunion<N>,
}
impl<const N:usize> Flexstr<N>
{
  /// Creates a new `Flexstr<N>` with given &str.  If the length of the &str
  /// is less than N and N<=256, the internal representation is an `[u8;N]`
  /// array with the first byte holding the length of the string.  Otherwise,
  /// the internal representation is an owned String.
  pub fn make(s:&str) -> Self
  {
     if s.len()<N && N<=256 {Flexstr{inner:fixed(tstr::<N>::from(s))}}
     else {Flexstr{inner:owned(String::from(s))}}
  }//make

  #[cfg(feature="serde")]
  /// this function is only added for uniformity in serde implementation
  pub fn try_make(s: &str) -> Result<Flexstr<N>, &str> {
       Ok(Flexstr::make(s))
    }
/*
  /// length of the string in bytes. This is a constant-time operation.
  pub fn len(&self) -> usize
  {
    match &self.inner {
      fixed(s) => s.len(),
      owned(s) => s.len(),
    }//match
  }//len
*/
  /// creates an empty string, equivalent to [Flexstr::default]
  pub fn new() -> Self { Self::default() }

  /// length in number of characters as opposed to bytes: this is
  /// not necessarily a constant time operation.
  pub fn charlen(&self) -> usize {
     match &self.inner {
       fixed(s) => s.charlen(),
       owned(s) => {
         let v: Vec<_> = s.chars().collect();
         v.len()
       },
     }//match
  }//charlen

  /// converts fstr to &str, possibly using using [std::str::from_utf8_unchecked].  Since
  /// Flexstr can only be built from valid utf8 sources, this function
  /// is safe.
  pub fn to_str(&self) -> &str
  {
    match &self.inner {
      fixed(s) => s.to_str(),
      owned(s) => &s[..],
    }//match
  }//to_str

    /// same functionality as [Flexstr::to_str], but only uses
    ///[std::str::from_utf8] and may technically panic.
    pub fn as_str(&self) -> &str //{self.to_str()}
    {
       match &self.inner {
         fixed(s) => s.as_str(),
         owned(s) => &s[..],
       }//match        
    }

  /// retrieves a copy of the underlying fixed string, if it is a fixed string.
  /// Note that since the `tstr` type is not exported, this function should
  /// be used in conjunction with one of the public aliases [str4]-[str256].
  /// For example,
  /// ```
  ///   let s = Flexstr::<8>::from("abcd");
  ///   let t:str8 = s.get_str().unwrap();
  /// ```
  pub fn get_str(&self) -> Option<tstr<N>> {
    if let fixed(s) = &self.inner { Some(*s) }
    else {None}
  }//get_str

  /// if the underlying representation of the string is an owned string,
  /// return the owned string, leaving an empty string in its place.
  pub fn take_string(&mut self) -> Option<String>
  {
     if let owned(s) = &mut self.inner {
       let mut temp = fixed(tstr::new());
       std::mem::swap(&mut self.inner, &mut temp);
       if let owned(t) = temp {Some(t)} else {None}
     }
     else {None}
  }//take_owned

  /// this function consumes the Flexstr and returns an owned string
  pub fn to_string(self) -> String 
  {
    match self.inner {
      fixed(s) => s.to_string(),
      owned(s) => s,
    }//match    
  }//to_string

  /// returns the nth char of the string, if it exists
  pub fn nth(&self, n: usize) -> Option<char> {
    self.to_str().chars().nth(n)
  }

  /// returns the nth byte of the string as a char.  This function
  /// is designed to be quicker than [Flexstr::nth] and does not check
  /// for bounds.
  pub fn nth_ascii(&self, n:usize) -> char {
    match &self.inner {
       fixed(s) => s.nth_ascii(n),
       owned(s) => s.as_bytes()[n] as char,
    }
  }//nth_ascii

  /// returns a u8-slice that represents the underlying string. The first
  /// byte of the slice is **not** the length of the string regarless of
  /// the internal representation.
  pub fn as_bytes(&self) -> &[u8] {
    match &self.inner {
      fixed(f) => f.as_bytes(),
      owned(s) => s.as_bytes(),
    }//match
  }

  /// changes a character at character position i to c.  This function
  /// requires that c is in the same character class (ascii or unicode)
  /// as the char being replaced.  It never shuffles the bytes underneath.
  /// The function returns true if the change was successful.
  pub fn set(&mut self, i: usize, c: char) -> bool {
     match &mut self.inner {
       fixed(s) => s.set(i,c),
       owned(s) => unsafe {
        let ref mut cbuf = [0u8; 4];
        c.encode_utf8(cbuf);
        let clen = c.len_utf8();
        if let Some((bi, rc)) = s.char_indices().nth(i) {
            if clen == rc.len_utf8() {
                s.as_bytes_mut()[bi..bi+clen].copy_from_slice(&cbuf[..clen]);
                //self.chrs[bi + 1..bi + clen + 1].copy_from_slice(&cbuf[..clen]);
                //for k in 0..clen {self.chrs[bi+k+1] = cbuf[k];}
                return true;
            }
        }
        return false;
       },
     }//match
  } //set

  /// returns whether the internal representation is a fixed string (tstr)
  pub fn is_fixed(&self) -> bool {
    match &self.inner {
      fixed(_) => true,
      owned(_) => false,
    }
  }//is_fixed

  /// returns whether the internal representation is an owned String
  pub fn is_owned(&self) -> bool { !self.is_fixed() }

  /// applies the destructive closure only if the internal representation
  /// is a fixed string
  pub fn if_fixed<F>(&mut self, f:&mut F) where F:FnMut(&mut tstr<N>)
  {
     if let fixed(s) = &mut self.inner {f(s);}
  }

  /// applies the destructive closure only if the internal representation
  /// is a fixed string
  pub fn if_owned<F>(&mut self, f:&mut F) where F:FnMut(&mut str)
  {
     if let owned(s) = &mut self.inner {f(s);}
  }

  /// applies closure f if the internal representation is a fixed string,
  /// or closure g if the internal representation is an owned string.
  pub fn map_or<F,G,U>(&self, f:&F, g:&G) -> U
    where F:Fn(&tstr<N>)-> U, G:Fn(&str) -> U
  {
     match &self.inner {
       fixed(s) => f(s),
       owned(s) => g(&s[..]),
     }//match
  }//map

  /// version of [Flexstr::map_or] accepting mut-closures
  pub fn map_or_mut<F,G,U>(&mut self, f:&mut F, g:&mut G) -> U
    where F:FnMut(&mut tstr<N>)-> U, G:FnMut(&mut str) -> U
  {
     match &mut self.inner {
       fixed(s) => f(s),
       owned(s) => g(&mut s[..]),
     }//match
  }//map
  
  /// This function will append the Flexstr with the given slice,
  /// switching to the owned-String representation if necessary.  The function
  /// returns true if the resulting string uses a `tstr<N>` type, and
  /// false if the representation is an owned string.
  pub fn push_str(&mut self, s:&str) -> bool {
    match &mut self.inner {
      fixed(fs) if fs.len()+s.len() < N => { fs.push(s); true},
      fixed(fs) => {
        let fss = fs.to_string() + s;
        self.inner = owned(fss);
        false
      },
      owned(ns) => {ns.push_str(s); false},
    }//match
  }//push

/*
  pub fn push(&mut self, c:char) -> bool {
     
  }
*/
  
  /// this function truncates a string, returning true if the truncated
  /// string is fixed, and false if owned.  The operation has no
  /// effect if n is larger than the length of the string
  pub fn truncate(&mut self, n: usize) -> bool {
    match &mut self.inner {
      fixed(fs) if n<fs.len() => { fs.truncate(n); true },
      fixed(_) => {true},
      owned(s) if n<N => {
        self.inner = fixed(tstr::<N>::from(&s[..n]));
        true
      },
      owned(s) => { if n<s.len() {s.truncate(n);} false},
    }//match
  }//truncate

  /// returns string corresponding to slice indices as a copy or clone.
  pub fn substr(&self, start: usize, end: usize) -> Flexstr<N> {
    match &self.inner {
      fixed(s) => Flexstr{inner:fixed(s.substr(start,end))},
      owned(s) => Self::from(&s[start..end]),
    }
  }//substr
} //impl<N>


impl<const N:usize> Default for Flexstr<N> {
  fn default() -> Self { Flexstr {inner:fixed(tstr::<N>::default())} }
}

impl<const N:usize> std::ops::Deref for Flexstr<N>
{
    type Target = str;
    fn deref(&self) -> &Self::Target {
      self.to_str()
    }
}

impl<T: AsRef<str> + ?Sized, const N: usize> std::convert::From<&T> for Flexstr<N> {
    fn from(s: &T) -> Self {
        Self::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> std::convert::From<&mut T> for Flexstr<N> {
    fn from(s: &mut T) -> Self {
        Self::make(s.as_mut())
    }
}

impl<const N: usize> std::cmp::PartialOrd for Flexstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.chrs[0..self.len].cmp(other.chrs[0..other.len]))
        Some(self.cmp(other))
    }
}

impl<const N: usize> std::cmp::Ord for Flexstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
      self.to_str().cmp(other.to_str())
    }
}

impl<const N: usize> std::convert::AsRef<str> for Flexstr<N> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}
impl<const N: usize> std::convert::AsMut<str> for Flexstr<N> {
    fn as_mut(&mut self) -> &mut str {
       match &mut self.inner {
         fixed(f) => f.as_mut(),
         owned(s) => s.as_mut(),
       }//match
    }
}

impl<const N: usize> PartialEq<&str> for Flexstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other // see below
    } //eq
}

impl<const N: usize> PartialEq<&str> for &Flexstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other
    } //eq
}
impl<'t, const N: usize> PartialEq<Flexstr<N>> for &'t str {
    fn eq(&self, other: &Flexstr<N>) -> bool {
        &other.to_str() == self
    }
}
impl<'t, const N: usize> PartialEq<&Flexstr<N>> for &'t str {
    fn eq(&self, other: &&Flexstr<N>) -> bool {
        &other.to_str() == self
    }
}

impl<const N: usize> std::fmt::Debug for Flexstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&self.to_str())
    }
} // Debug impl

impl<const N: usize> std::fmt::Display for Flexstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<const N: usize> std::fmt::Write for Flexstr<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result
    {
        self.push_str(s);
        Ok(())
    } //write_str
} //std::fmt::Write trait


impl<const M: usize> Flexstr<M> {
  /// returns a copy/clone of the string with new fixed capacity N.
  /// Example:
  /// ```
  ///  let a:Flexstr<4> = Flexstr::from("ab");
  ///  let mut b:Flexstr<8> = a.resize();
  ///  b.push_str("cdef");
  ///  assert!(b.is_fixed());
  ///  a.push_str("1234");
  ///  assert!(a.is_owned());
  /// ```
  pub fn resize<const N: usize>(&self) -> Flexstr<N> {
    Flexstr::from(self)
  }
}
