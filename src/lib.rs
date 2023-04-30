//! Library for strings of fixed maximum lengths that can be copied and
//! stack-allocated using const generics.
//!
//! **The structures provided by this crate are [fstr], [zstr]** and tstr.
//! However, tstr is not exported and can only be used through the type
//! aliases [str4], [str8], [str16], through [str256].
//!
//! The size of (std::mem::size_of) types str8 and zstr<8>
//! are 8 bytes, compared to 16 bytes for &str (on 64bit systems), providing more efficient
//! ways of representing very small strings.  Unicode is supported.
//!
//! The three versions of strings implemented are as follows.
//! - A **[fstr]\<N\>**
//! stores a string of up to N bytes.  It is represented underneath using
//! a \[u8;N\] array and a separate usize variable holding the length.
//! - A **[zstr]\<N\>** stores a zero-terminated string, without a separate
//! length variable, and can hold strings of up to N-1 bytes.
//! - The types **[str4]**, **[str8]** through **[str256]** are aliases for internal type
//! tstr<4> through tstr<256> respectively.  These strings are stored
//! in an array of u8 bytes with the first byte holding the length of the
//! string.  Each tstr\<N\> can store strings of up to N-1 bytes, with
//! maximum N=256. tstr
//! combines the best of fstr and zstr in terms of speed
//! and memory efficiency.  However, because Rust does not currently provide
//! a way to specify conditions on const generics at compile time, such as
//! `where N<=256`, the tstr type is not exported and can
//! only be used through the aliases.  These strings implement the same
//! functions and traits as [fstr] and [zstr] so the documentation for
//! these structures also apply to the alias types.
//!
//! **Recent Updates:**
//!
//! Version 0.2.12 includes contribution from
//! [wallefan](https://github.com/wallefan),
//! and added optional serde support for serialization.
//! This feature can be enabled by giving cargo the
//! **`--features serde`** option.
//!
//! Version 0.2.11 impls [std::fmt::Write], thereby enabling the [write!]
//! macro. Also adds new macros [str_format!] and [try_format!].
//!
//! Version 0.2.10 allows str4-str128 strings to be concatenated with
//! the `+` operator, resulting in strings with twice the capacity,
//! str8-str256.  This feature is only implemented for the strN types.
//!
//! Version 0.2.6-0.2.8 impls `AsRef<str>` and `AsMut<str>` traits.
//! Functions try_make and reallocate
//! have been added that do not truncate strings.  str4, str24 and
//! str48 were added.  [str4] can only hold three bytes but is good enough
//! for many types of abbreviations such as those for airports.
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
//! let (upper,lower) = (str8::make("ABC"), str8::make("abc"));
//! assert_eq!(upper, lower.to_ascii_uppercase()); // no owned String needed
//!  
//! let c1 = str8::from("abcdef"); // string concatenation with + for strN types  
//! let c2 = str8::from("xyz123"); // this features is not available for fstr and tstr
//! let c3 = c1 + c2;        // new in Version 0.2.10   
//! assert_eq!(c3,"abcdefxyz123");   
//! assert_eq!(c3.capacity(),15);  // type of c3 is str16
//!
//! // New in Version 0.4.11:
//! let c4 = str_format!(str16,"abc {}{}{}",1,2,3); // impls std::fmt::Write
//! assert_eq!(c4,"abc 123");  // str_format! truncates if capacity exceeded
//! let c5 = try_format!(str8,"abcdef{}","ghijklmn");
//! assert!(c5.is_none());  // try_format! returns None if capacity exceeded
//!```
//!
//![zstr] and the type aliases [str8]...[str256] implement the same functions and traits as [fstr].

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]

//#[macro_use]
//extern crate static_assertions;
//use std::ops::{Add};
//use std::fmt::Write;
pub mod zero_terminated;
pub use zero_terminated::*;
mod tiny_internal;
use std::cmp::{min, Ordering};
use tiny_internal::*;

/// main type: string of size up to const N:
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct fstr<const N: usize> {
    chrs: [u8; N],
    len: usize, // length will be <=N
} //fstr
impl<const N: usize> fstr<N> {
    /// creates a new `fstr<N>` with given &str.  If the length of s exceeds
    /// N, the extra characters are ignored and a **warning is sent to stderr**.
    /// This function is also called by
    /// several others including [fstr::from].
    pub fn make(s: &str) -> fstr<N> {
        let bytes = s.as_bytes(); // &[u8]
        let mut blen = bytes.len();
        if (blen > N) {
            eprintln!("!Fixedstr Warning in fstr::make: length of string literal \"{}\" exceeds the capacity of type fstr<{}>; string truncated",s,N);
            blen = N;
        }
        let mut chars = [0u8; N];
        let mut i = 0;
        let limit = min(N, blen);
        chars[..limit].clone_from_slice(&bytes[..limit]);
        /* //replaced re performance lint
        for i in 0..blen
        {
          if i<N {chars[i] = bytes[i];} else {break;}
        }
        */
        fstr {
            chrs: chars,
            len: blen, /* as u16 */
        }
    } //make

    /// Version of make that does not print warning to stderr.  If the
    /// capacity limit is exceeded, the extra characters are ignored.
    pub fn create(s: &str) -> fstr<N> {
        let bytes = s.as_bytes(); // &[u8]
        let mut blen = bytes.len();
        if (blen > N) {
            blen = N;
        }
        let mut chars = [0u8; N];
        let mut i = 0;
        let limit = min(N, blen);
        chars[..limit].clone_from_slice(&bytes[..limit]);
        fstr {
            chrs: chars,
            len: blen,
        }
    } //create

    /// version of make that does not truncate, if s exceeds capacity,
    /// an Err result is returned containing s
    pub fn try_make(s: &str) -> Result<fstr<N>, &str> {
        if s.len() > N {
            Err(s)
        } else {
            Ok(fstr::make(s))
        }
    }

    /// creates an empty string, equivalent to fstr::default()
    pub fn new() -> fstr<N> {
        fstr::make("")
    }

    /// length of the string in bytes, which will be up to the maximum size N.
    /// This is a constant-time operation. Note that this value is consistent
    /// with [str::len]
    pub fn len(&self) -> usize {
        self.len
    }

    /// returns maximum capacity in bytes
    pub fn capacity(&self) -> usize {
        N
    }

    /// converts fstr to an owned string
    pub fn to_string(&self) -> String {
        self.to_str().to_owned()
        //self.chrs[0..self.len].iter().map(|x|{*x as char}).collect()
    }

    /// allows returns copy of u8 array underneath the fstr
    pub fn as_u8(&self) -> [u8; N] {
        self.chrs
    }

    /// converts fstr to &str using [std::str::from_utf8_unchecked].  Since
    /// fstr can only be build from valid utf8 sources, using this function
    /// is safe.
    pub fn to_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.chrs[0..self.len]) }
    }
    /// same functionality as [fstr::to_str]
    pub fn as_str(&self) -> &str //{self.to_str()}
    {
        std::str::from_utf8(&self.chrs[0..self.len]).unwrap()
    }

    /// changes a character at character position i to c.  This function
    /// requires that c is in the same character class (ascii or unicode)
    /// as the char being replaced.  It never shuffles the bytes underneath.
    /// The function returns true if the change was successful.
    pub fn set(&mut self, i: usize, c: char) -> bool {
        let ref mut cbuf = [0u8; 4]; // characters require at most 4 bytes
        c.encode_utf8(cbuf);
        let clen = c.len_utf8();
        if let Some((bi, rc)) = self.to_str().char_indices().nth(i) {
            if clen == rc.len_utf8() {
                self.chrs[bi..bi + clen].clone_from_slice(&cbuf[..clen]);
                //for k in 0..clen {self.chrs[bi+k] = cbuf[k];}
                return true;
            }
        }
        return false;
    }
    /// adds chars to end of current string up to maximum size N of `fstr<N>`,
    /// returns the portion of the push string that was NOT pushed due to
    /// capacity, so
    /// if "" is returned then all characters were pushed successfully.
    pub fn push<'t>(&mut self, s: &'t str) -> &'t str {
        if s.len() < 1 {
            return s;
        }
        let mut buf = [0u8; 4];
        let mut i = self.len();
        let mut sci = 0; // indexes characters in s
        for c in s.chars() {
            let clen = c.len_utf8();
            c.encode_utf8(&mut buf);
            if i <= N - clen {
                self.chrs[i..i + clen].clone_from_slice(&buf[..clen]);
                /*
                for k in 0..clen
                {
                  self.chrs[i+k] = buf[k];
                }
                */
                i += clen;
            } else {
                self.len = i;
                return &s[sci..];
            }
            sci += 1;
        }
        self.len = i;
        &s[sci..]
    } //push

    /// returns the number of characters in the string regardless of
    /// character class
    pub fn charlen(&self) -> usize {
        let v: Vec<_> = self.to_str().chars().collect();
        v.len()
    }

    /// returns the nth char of the fstr
    pub fn nth(&self, n: usize) -> Option<char> {
        self.to_str().chars().nth(n)
    }

    /// returns the nth byte of the string as a char.  This
    /// function should only be called on ascii strings.  It
    /// is designed to be quicker than [fstr::nth], and does not check array bounds or
    /// check n against the length of the string. Nor does it check
    /// if the value returned is within the ascii range.
    pub fn nth_ascii(&self, n: usize) -> char {
        self.chrs[n] as char
    }

    /// determines if string is an ascii string
    pub fn is_ascii(&self) -> bool {
        self.to_str().is_ascii()
    }

    /// shortens the fstr in-place (mutates).  If n is greater than the
    /// current length of the string in chars, this operation will have no effect.
    pub fn truncate(&mut self, n: usize) {
        if let Some((bi, c)) = self.to_str().char_indices().nth(n) {
            //self.chrs[bi] = 0;
            self.len = bi;
        }
        //if n<self.len {self.len = n;}
    }

    /// in-place modification of ascii characters to lower-case
    pub fn make_ascii_lowercase(&mut self) {
      for b in &mut self.chrs[..self.len] {
        if *b>=65 && *b<=90 { *b |= 32; }
      }
    }//make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case
    pub fn make_ascii_uppercase(&mut self) {
      for b in &mut self.chrs[..self.len] {
        if *b>=97 && *b<=122 { *b -= 32; }
      }      
    }

    /// Constructs a clone of this fstr but with only upper-case ascii
    /// characters.  This contrasts with [str::to_ascii_uppercase],
    /// which creates an owned String. 
    pub fn to_ascii_uppercase(&self) -> Self
    {
      let mut cp = self.clone();
      cp.make_ascii_uppercase();
      cp
    }

    /// Constructs a clone of this fstr but with only lower-case ascii
    /// characters.  This contrasts with [str::to_ascii_lowercase],
    /// which creates an owned String.
    pub fn to_ascii_lowercase(&self) -> Self
    {
      let mut cp = *self;
      cp.make_ascii_lowercase();
      cp
    }

} //impl fstr<N>

#[cfg(feature="serde")]
mod serde_support {
    use serde::{Serialize, Deserialize, Serializer, Deserializer, de::Visitor};
    use super::*;
    macro_rules! generate_impl {
        ($ty: ident, $visitor: ident) => {
            impl<const N: usize> Serialize for $ty<N> {
                fn serialize<S: Serializer>(&self, serializer:S) -> Result<S::Ok, S::Error> {
                    serializer.serialize_str(self.as_str())
                }
            }
            impl<'de, const N: usize> Deserialize<'de> for $ty<N> {
                fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                    deserializer.deserialize_str($visitor)
                }
            }
            struct $visitor<const N: usize>;
            impl<'de, const N: usize> Visitor<'de> for $visitor<N> {
                type Value = $ty<N>;
                fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.write_str("a string")
                }
                fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                    $ty::try_make(s).map_err(|_| E::custom("string too long"))
                }
            }
        }
    }
    generate_impl!(fstr, FstrVisitor);
    generate_impl!(zstr, ZstrVisitor);
    generate_impl!(tstr, TstrVisitor);
}

/*
impl<'t, const N:usize> std::convert::Into<&'t str> for fstr<N>
{
  fn into(self) -> &'t str
  {
     std::str::from_utf8(&self.chrs[0..self.len]).unwrap()
  }
}
*/

impl<T: AsRef<str> + ?Sized, const N: usize> std::convert::From<&T> for fstr<N> {
    fn from(s: &T) -> fstr<N> {
        fstr::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> std::convert::From<&mut T> for fstr<N> {
    fn from(s: &mut T) -> fstr<N> {
        fstr::make(s.as_mut())
    }
}

/*
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

impl<const N:usize,const M:usize> std::convert::From<&zstr<M>> for fstr<N>
{
  fn from(s:&zstr<M>) -> fstr<N>
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
*/

impl<const N: usize> std::convert::From<String> for fstr<N> {
    fn from(s: String) -> fstr<N> {
        fstr::<N>::make(&s[..])
    }
}

impl<const N: usize, const M: usize> std::convert::From<zstr<M>> for fstr<N> {
    fn from(s: zstr<M>) -> fstr<N> {
        fstr::<N>::make(&s.to_str())
    }
}

impl<const N: usize, const M: usize> std::convert::From<tstr<M>> for fstr<N> {
    fn from(s: tstr<M>) -> fstr<N> {
        fstr::<N>::make(&s.to_str())
    }
}

impl<const N: usize> std::cmp::PartialOrd for fstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.chrs[0..self.len].cmp(other.chrs[0..other.len]))
        Some(self.cmp(other))
    }
}

impl<const N: usize> std::cmp::Ord for fstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.chrs[0..self.len].cmp(&other.chrs[0..other.len])
    }
}

impl<const M: usize> fstr<M> {
    /// converts an fstr\<M\> to an fstr\<N\>. If the length of the string being
    /// converted is greater than N, the extra characters are ignored.
    /// This operation produces a copy (non-destructive).
    /// Example:
    ///```ignore
    ///  let s1:fstr<8> = fstr::from("abcdefg");
    ///  let s2:fstr<16> = s1.resize();
    ///```
    pub fn resize<const N: usize>(&self) -> fstr<N> {
        //if (self.len()>N) {eprintln!("!Fixedstr Warning in fstr::resize: string \"{}\" truncated while resizing to fstr<{}>",self,N);}
        let length = if (self.len < N) { self.len } else { N };
        let mut chars = [0u8; N];
        chars[..length].clone_from_slice(&self.chrs[..length]);
        //for i in 0..length {chars[i] = self.chrs[i];}
        fstr {
            chrs: chars,
            len: length,
        }
    } //resize

    /// version of resize that does not allow string truncation due to length
    pub fn reallocate<const N: usize>(&self) -> Option<fstr<N>> {
        if self.len() <= N {
            Some(self.resize())
        } else {
            None
        }
    }
} //impl fstr<M>

/* doesn't work
impl<const M:usize, const N:usize> Add for fstr<M> {
  type Output = fstr<{N+M}>;
  fn add(self, other:fstr<N>) -> Output {
     let mut cat:Output = self.resize();
     cat.push(other);
     cat
  }
}//Add
*/

impl<const N: usize> std::convert::AsRef<str> for fstr<N> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}
impl<const N: usize> std::convert::AsMut<str> for fstr<N> {
    fn as_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(&mut self.chrs[0..self.len]) }
    }
}

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

impl<const N: usize> std::fmt::Display for fstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str()) // need change!
    }
}

/*
impl<T:AsRef<str>+?Sized, const N:usize> PartialEq<&T> for fstr<N>
{
   fn eq(&self, other:&&T)->bool { self.as_ref() == other.as_ref() }
}
*/

impl<const N: usize> PartialEq<&str> for fstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other // see below
    } //eq
}

impl<const N: usize> PartialEq<&str> for &fstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other
    } //eq
}
impl<'t, const N: usize> PartialEq<fstr<N>> for &'t str {
    fn eq(&self, other: &fstr<N>) -> bool {
        &other.to_str() == self
    }
}
impl<'t, const N: usize> PartialEq<&fstr<N>> for &'t str {
    fn eq(&self, other: &&fstr<N>) -> bool {
        &other.to_str() == self
    }
}

/// defaults to empty string
impl<const N: usize> Default for fstr<N> {
    fn default() -> Self {
        fstr::<N>::make("")
    }
}

impl<const N: usize> std::fmt::Debug for fstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ds = format!("fstr<{}>:\"{}\"", N, &self.to_str());
        f.pad(&ds)
    }
} // Debug impl

///Convert fstr to &[u8] slice
impl<IndexType, const N: usize> std::ops::Index<IndexType> for fstr<N>
where
    IndexType: std::slice::SliceIndex<[u8]>,
{
    type Output = IndexType::Output;
    fn index(&self, index: IndexType) -> &Self::Output {
        &self.chrs[index]
    }
} //impl Index

  // couldn't get it to work properly, [char] is not same as &str
  // because there's no allocated string!
/*
  ///Convert fstr to &str slice
  impl<IndexType,const N:usize> std::ops::Index<IndexType> for fstr<N>
    where IndexType:std::slice::SliceIndex<str>,
  {
    type Output = IndexType::Output;
    fn index(&self, index:IndexType)-> &Self::Output
    {
       &self.chrs[index]
    }
  }//impl Index
*/

impl<const N: usize> fstr<N> {
    /// mimics same function on str
    pub fn chars(&self) -> std::str::Chars<'_> {
        self.to_str().chars()
    }
    /// mimics same function on str
    pub fn char_indices(&self) -> std::str::CharIndices<'_> {
        self.to_str().char_indices()
    }

    /// returns a copy of the portion of the string, string could be truncated
    /// if indices are out of range. Similar to slice [start..end]
    pub fn substr(&self, start: usize, end: usize) -> fstr<N> {
        let mut chars = [0u8; N];
        let mut inds = self.char_indices();
        let len = self.len();
        if start >= len || end <= start {
            return fstr {
                chrs: chars,
                len: 0,
            };
        }
        let (si, _) = inds.nth(start).unwrap();
        let last = if (end >= len) {
            len
        } else {
            match inds.nth(end - start - 1) {
                Some((ei, _)) => ei,
                None => len,
            } //match
        }; //let last =...

        chars[0..last - si].clone_from_slice(&self.chrs[si..last]);
        /*
        for i in si..last
        {
          chars[i-si] = self.chrs[i];
        }
        */
        fstr {
            chrs: chars,
            len: end - start,
        }
    } //substr
}

/// types for small strings that use a more efficient representation
/// underneath.  A str8 can hold a string of up to 7 bytes (7 ascii chars).
/// The same functions for [fstr] and [zstr] are provided for these types
/// so the documentation for the other types also applies.
/// The size of str8 is 8 bytes.
///
/// Example:
/// ```
///  let mut s = str8::from("aλc");
///  assert_eq!(s.capacity(),7);
///  assert_eq!(s.push("1234567"), "4567");
///  assert_eq!(s,"aλc123");
///  assert_eq!(s.charlen(), 6);
///  assert_eq!(s.len(), 7);  
/// ```
pub type str8 = tstr<8>;
/// A str16 can hold a string of up to 15 bytes. See docs for [fstr] or [zstr].
/// The size of str16 is 16 bytes, which is the same as for &str on 64bit
/// systems.
pub type str16 = tstr<16>;
/// A str32 can hold a string of up to 31 bytes. See docs for [fstr] or [zstr]
pub type str32 = tstr<32>;
/// A str64 can hold a string of up to 63 bytes. See docs for [fstr] or [zstr]
pub type str64 = tstr<64>;
/// A str28 can hold a string of up to 127 bytes. See docs for [fstr] or [zstr]
pub type str128 = tstr<128>;
/// Each type strN is represented underneath by a `[u8;N]` with N<=256.
/// The first byte of the array always holds the length of the string.
/// Each such type can hold a string of up to N-1 bytes, with max size=255.
/// These types represent the best combination of [fstr] and [zstr] in
/// terms of speed and memory efficiency.  Consult documentation for [fstr]
/// or [zstr] for the same functions and traits.
/// <br>
/// <br>
/// In addition, the str4-str128 types implement [std::ops::Add], allowing for
/// string concatenation of strings of the same type.  For example,
/// two str8 strings will always concatenate to str16, and similarly for
/// all other strN types up to str128.
///```
///  let c1 = str8::from("abcd");
///  let c2 = str8::from("xyz");
///  let c3 = c1 + c2;
///  assert_eq!(c3,"abcdxyz");
///  assert_eq!(c3.capacity(),15);
///```
pub type str256 = tstr<256>;

/// strings of up to three 8-bit chars, good enough to represent abbreviations
/// such as those for states and airports. Each str<4> is exactly 32 bits.
pub type str4 = tstr<4>;
pub type str12 = tstr<12>;
pub type str24 = tstr<24>;
pub type str48 = tstr<48>;
pub type str96 = tstr<96>;
pub type str192 = tstr<192>;

////////////// std::fmt::Write trait
/// Usage:
/// ```
///   use std::fmt::Write;
///   let mut s = fstr::<32>::new();
///   let result = write!(&mut s,"hello {}, {}, {}",1,2,3);
///   /* or */
///   let s2 = str_format(<fstr<24>,"hello {}, {}, {}",1,2,3);
///   let s3 = try_format(<fstr<4>,"hello {}, {}, {}",1,2,3); // returns None
/// ```
impl<const N: usize> std::fmt::Write for fstr<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result //Result<(),std::fmt::Error>
    {
        //if s.len() + self.len() > N {return Err(std::fmt::Error::default());}
        //self.push(s);
        let rest = self.push(s);
        if rest.len() > 0 {
            return Err(std::fmt::Error::default());
        }
        Ok(())
    } //write_str
} //std::fmt::Write trait

/*
fn fstr_write<const N:usize>(args:std::fmt::Arguments) -> fstr<N> {
     use std::fmt::Write;
     let mut fstr0 = fstr::<N>::new();
     //let result = std::fmt::write(&mut fstr0, args);
     let result = fstr0.write_fmt(args);
     fstr0
}
*/

#[macro_export]
/// creates a formated string of given type (by implementing [std::fmt::Write]):
/// ```
///    let s = str_format!(str8,"abc{}{}{}",1,2,3);
/// ```
/// will truncate if capacity exceeded, without warning.
macro_rules! str_format {
  ($ty_size:ty, $($args:tt)*) => {
     {use std::fmt::Write;
     let mut fstr0 = <$ty_size>::new();
     let res=write!(&mut fstr0, $($args)*);
     fstr0}
  };
}

#[macro_export]
/// version of [str_format]! that returns an Option of the given type.
/// ```
///  let s = try_format!(str32,"abcdefg{}","hijklmnop").unwrap();
///  let s2 = try_format!(str8,"abcdefg{}","hijklmnop");
///  assert!(s2.is_none());
/// ```
macro_rules! try_format {
  ($ty_size:ty, $($args:tt)*) => {
     {use std::fmt::Write;
     let mut fstr0 = <$ty_size>::new();
     let result = write!(&mut fstr0, $($args)*);
     if result.is_ok() {Some(fstr0)} else {None}}
  };
}
