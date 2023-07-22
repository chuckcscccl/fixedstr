//! This module is intended for internal use only.  Only a few
//! type aliases are exported.  A tiny string or `tstr<N>`, with N<=256,
//! is a version of fixed str that represents the best compromise between
//! memory and runtime efficiency.  Each `tstr<N>` can hold a string of up to
//! N-1 bytes, with max N=256.  A `tstr<N>` is represented underneath
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
#![allow(unused_imports)]
#![allow(dead_code)]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use crate::fstr;

use crate::zstr;
use crate::{str12, str128, str16, str192, str24, str256, str32, str4, str48, str64, str8, str96};
use core::cmp::{min, Ordering};
use core::ops::{Add,Range,Index,IndexMut,RangeFull,RangeFrom,RangeTo};
use core::ops::{RangeInclusive,RangeToInclusive};

/// **This structure is only exported with the `features pub-tstr` option.**
/// Otherwise, it can only be referenced through the
/// public type aliases [crate::str4] through [crate::str256].
/// This type supports `#![no_std]` with
/// the `no-default-features` option.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct tstr<const N: usize = 256> {
    chrs: [u8; N],
} //tstr
impl<const N: usize> tstr<N> {
    /// creates a new `tstr<N>` with given &str.  If the length of s exceeds
    /// N, the extra characters are ignored.  This function is also called by
    /// several others including [tstr::from].  This function can now handle
    /// utf8 strings properly.
    pub fn make(s: &str) -> tstr<N> {
        let mut chars = [0u8; N];
        let bytes = s.as_bytes(); // &[u8]
        let blen = bytes.len();
        /*
        if (blen >= N) {
            eprintln!("!Fixedstr Warning in str::make: length of string literal \"{}\" exceeds the capacity of type str{}; string truncated",s,N);
        }
        */
        let limit = min(N - 1, blen);
        chars[1..limit + 1].copy_from_slice(&bytes[..limit]);
        chars[0] = limit as u8;
        /*
        for i in 0..blen
        {
          if i<N-1 {chars[i+1] = bytes[i];}
          else { chars[0] = i as u8; break; }
        }
        */
        if chars[0] == 0 {
            chars[0] = blen as u8;
        }
        tstr { chrs: chars }
    } //make

    /// alias for [Self::make]
    pub fn create(s: &str) -> tstr<N> {
        let mut chars = [0u8; N];
        let bytes = s.as_bytes();
        let blen = bytes.len();
        let limit = min(N - 1, blen);
        chars[1..limit + 1].copy_from_slice(&bytes[..limit]);
        chars[0] = limit as u8;
        if chars[0] == 0 {
            chars[0] = blen as u8;
        }
        tstr { chrs: chars }
    } //create

    /// version of make that returns the string in an `Err(_)` if
    /// truncation is requried, or in an `Ok(_)` if no truncation is required
    pub fn try_make(s: &str) -> Result<tstr<N>, &str> {
        if s.len() > N - 1 {
            Err(s)
        } else {
            Ok(tstr::make(s))
        }
    }

    /// creates an empty string, equivalent to tstr::default()
    pub fn new() -> tstr<N> {
        tstr::make("")
    }

    /// length of the string in bytes (consistent with [str::len]). This
    /// is a constant-time operation.
    pub fn len(&self) -> usize {
        self.chrs[0] as usize
    }

    /// returns the number of characters in the string regardless of
    /// character class
    pub fn charlen(&self) -> usize {
        self.to_str().chars().count()
    }

    /// returns maximum capacity in bytes
    pub fn capacity(&self) -> usize {
        N - 1
    }

    /// converts tstr to an owned string (not available when no_std enforced)
    #[cfg(feature = "std")]    
    pub fn to_string(&self) -> std::string::String {
        let vs: std::vec::Vec<_> = self.chrs[1..self.len() + 1].iter().map(|x| *x).collect();
        std::string::String::from_utf8(vs).expect("Invalid utf8 string")
    }

    /// returns copy of u8 array underneath the tstr
    pub fn as_bytes(&self) -> &[u8] {
        &self.chrs[1..self.len() + 1]
    }

    /// converts tstr to &str using [core::str::from_utf8_unchecked]
    pub fn to_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.chrs[1..self.len() + 1]) }
    }
    /// checked version of [tstr::to_str], may panic
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.chrs[1..self.len() + 1]).unwrap()
    }

    /// changes a character at character position i to c.  This function
    /// requires that c is in the same character class (ascii or unicode)
    /// as the char being replaced.  It never shuffles the bytes underneath.
    /// The function returns true if the change was successful.
    pub fn set(&mut self, i: usize, c: char) -> bool {
        let ref mut cbuf = [0u8; 4];
        c.encode_utf8(cbuf);
        let clen = c.len_utf8();
        if let Some((bi, rc)) = self.to_str().char_indices().nth(i) {
            if clen == rc.len_utf8() {
                self.chrs[bi + 1..bi + clen + 1].copy_from_slice(&cbuf[..clen]);
                //for k in 0..clen {self.chrs[bi+k+1] = cbuf[k];}
                return true;
            }
        }
        return false;
    } //set
    /// adds chars to end of current string up to maximum size N of `tstr<N>`,
    /// returns the portion of the push string that was NOT pushed due to
    /// capacity, so
    /// if "" is returned then all characters were pushed successfully.
    pub fn push<'t>(&mut self, s: &'t str) -> &'t str {
        if s.len() < 1 {
            return s;
        }
        let mut buf = [0u8; 4];
        let mut i = self.len();
	let mut sci = 0; // length in bytes
        for c in s.chars() {
            let clen = c.len_utf8();
            c.encode_utf8(&mut buf);
            if i+clen+1 <= N {
                self.chrs[i+1 .. i+clen+1].copy_from_slice(&buf[..clen]);
                i += clen;
            } else {
                self.chrs[0] = i as u8;
                return &s[sci..];
            }
	    sci += clen;
        }
        if i < N {
            self.chrs[0] = i as u8;
        } // set length
        &s[sci..]
    } //push

    /// alias for [Self::push]
    pub fn push_str<'t>(&mut self, s: &'t str) -> &'t str {
      self.push(s)
    }

    /// pushes a single character to the end of the string, returning
    /// true on success.
    pub fn push_char(&mut self, c:char) -> bool {
       let clen = c.len_utf8();
       let slen = self.len();
       if slen+clen >= N {return false;}
       let mut buf = [0u8;4]; // char buffer
       c.encode_utf8(&mut buf);
       for i in 0..clen {
         self.chrs[slen+i+1] = buf[i];
       }
       self.chrs[0] = (slen+clen) as u8;
       true
    }// push_char

    /// remove and return last character in string, if it exists
    pub fn pop_char(&mut self) -> Option<char> {
       if self.len()==0 {return None;}
       let (ci,lastchar) = self.char_indices().last().unwrap();
       self.chrs[0]=ci as u8;
       Some(lastchar)
    }//pop


    /// returns the nth char of the tstr
    pub fn nth(&self, n: usize) -> Option<char> {
        self.to_str().chars().nth(n)
    }

    /// returns the nth byte of the string as a char.  This
    /// function should only be called, for example, on ascii strings.  It
    /// is designed to be quicker than [tstr::nth], and does not check array bounds or
    /// check n against the length of the string. Nor does it check
    /// if the value returned is a valid character.
    pub fn nth_bytechar(&self, n: usize) -> char {
        self.chrs[n + 1] as char
    }

    /// alias for [Self::nth_bytechar] (for backwards compatibility)
    pub fn nth_ascii(&self, n: usize) -> char {
        self.chrs[n + 1] as char
    }

    /// determines if string is an ascii string
    pub fn is_ascii(&self) -> bool {
        self.to_str().is_ascii()
    }

    /// shortens the tstr in-place (mutates).  n indicates the number of
    /// *characters* to keep in thestring. If n is greater than the
    /// current character-length ([Self::charlen]) of the string, this operation will have no effect.
    pub fn truncate(&mut self, n: usize) // n is char position, not binary position
    {
        if let Some((bi, c)) = self.to_str().char_indices().nth(n) {
            self.chrs[0] = bi as u8;
        }
    }
    
    /// truncates string up to *byte* position n.  **Panics** if n is
    /// not on a character boundary, similar to truncate on owned Strings.
    pub fn truncate_bytes(&mut self, n: usize) {
       if (n<self.chrs[0] as usize) {
         assert!(self.is_char_boundary(n));
	 self.chrs[0] = n as u8;
       }
    }

/// Trims **in-place** trailing ascii whitespaces.  This function
    /// regards all bytes as single chars.  The operation panics if
    /// the resulting string does not end on a character boundary.
    pub fn right_ascii_trim(&mut self) {
      let mut n = self.chrs[0] as usize;
      while n>0 && (self.chrs[n] as char).is_ascii_whitespace() {
        //self.chrs[n-1] = 0;
        n -= 1;
      }
      assert!(self.is_char_boundary(n));
      self.chrs[0] = n as u8;
    }//right_trim

    /// resets string to empty string
    pub fn clear(&mut self) {
      self.chrs[0]=0;
    }

    /// in-place modification of ascii characters to lower-case
    pub fn make_ascii_lowercase(&mut self) {
      let end = (self.chrs[0] as usize)+1;
      for b in &mut self.chrs[1..end] {
        if *b>=65 && *b<=90 { *b |= 32; }
      }
    }//make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case
    pub fn make_ascii_uppercase(&mut self) {
      let end = (self.chrs[0] as usize)+1;    
      for b in &mut self.chrs[1..end] {
        if *b>=97 && *b<=122 { *b -= 32; }
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

} //impl tstr<N>

impl<const N:usize> core::ops::Deref for tstr<N>
{
    type Target = str;
    fn deref(&self) -> &Self::Target {
      self.to_str()
    }
}

impl<const N: usize> core::convert::AsRef<str> for tstr<N> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}
impl<const N: usize> core::convert::AsMut<str> for tstr<N> {
    fn as_mut(&mut self) -> &mut str {
        let blen = self.len() + 1;
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.chrs[1..blen]) }
    }
}
impl<T: AsRef<str> + ?Sized, const N: usize> core::convert::From<&T> for tstr<N> {
    fn from(s: &T) -> tstr<N> {
        tstr::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> core::convert::From<&mut T> for tstr<N> {
    fn from(s: &mut T) -> tstr<N> {
        tstr::make(s.as_mut())
    }
}

#[cfg(feature = "std")]
impl<const N: usize> core::convert::From<std::string::String> for tstr<N> {
    fn from(s: std::string::String) -> tstr<N> {
        tstr::<N>::make(&s[..])
    }
}

#[cfg(feature = "std")]
impl<const N: usize, const M: usize> std::convert::From<fstr<M>> for tstr<N> {
    fn from(s: fstr<M>) -> tstr<N> {
        tstr::<N>::make(s.to_str())
    }
}

impl<const N: usize, const M: usize> core::convert::From<zstr<M>> for tstr<N> {
    fn from(s: zstr<M>) -> tstr<N> {
        tstr::<N>::make(s.to_str())
    }
}

impl<const N: usize> core::cmp::PartialOrd for tstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.chrs[0..self.len()].cmp(other.chrs[0..other.len()]))
        Some(self.cmp(other))
    }
}

impl<const N: usize> core::cmp::Ord for tstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.chrs[1..self.len() + 1].cmp(&other.chrs[1..other.len() + 1])
    }
}

impl<const M: usize> tstr<M> {
    /// converts an tstr\<M\> to an tstr\<N\>. If the length of the string being
    /// converted is greater than N, the extra characters will be ignored.
    /// This operation produces a copy (non-destructive).
    /// Example:
    ///```ignore
    ///  let s1:tstr<8> = tstr::from("abcdefg");
    ///  let s2:tstr<16> = s1.resize();
    ///```
    pub fn resize<const N: usize>(&self) -> tstr<N> {
        let slen = self.len();
        let length = if (slen < N - 1) { slen } else { N - 1 };
        let mut chars = [0u8; N];
        chars[1..length + 1].copy_from_slice(&self.chrs[1..length + 1]);
        //for i in 0..length {chars[i+1] = self.chrs[i+1];}
        chars[0] = (length) as u8;
        tstr { chrs: chars }
    } //resize

    /// version of resize that does not allow string truncation due to length
    pub fn reallocate<const N: usize>(&self) -> Option<tstr<N>> {
        if self.len() < N {
            Some(self.resize())
        } else {
            None
        }
    } //reallocate
} //impl tstr<M>

impl<const N: usize> core::fmt::Display for tstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<const N: usize> PartialEq<&str> for tstr<N> {
    fn eq(&self, other: &&str) -> bool {
        self.to_str() == *other // see below
    } //eq
}
impl<const N: usize> PartialEq<&str> for &tstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other
    } //eq
}
impl<'t, const N: usize> PartialEq<tstr<N>> for &'t str {
    fn eq(&self, other: &tstr<N>) -> bool {
        &other.to_str() == self
    }
}
impl<'t, const N: usize> PartialEq<&tstr<N>> for &'t str {
    fn eq(&self, other: &&tstr<N>) -> bool {
        &other.to_str() == self
    }
}

/// defaults to empty string
impl<const N: usize> Default for tstr<N> {
    fn default() -> Self {
        tstr::<N>::make("")
    }
}
#[cfg(feature = "std")]
impl<const N: usize, const M: usize> PartialEq<tstr<N>> for fstr<M> {
    fn eq(&self, other: &tstr<N>) -> bool {
        other.to_str() == self.to_str()
    }
}
#[cfg(feature = "std")]
impl<const N: usize, const M: usize> PartialEq<fstr<N>> for tstr<M> {
    fn eq(&self, other: &fstr<N>) -> bool {
        other.to_str() == self.to_str()
    }
}

impl<const N: usize, const M: usize> PartialEq<zstr<N>> for tstr<M> {
    fn eq(&self, other: &zstr<N>) -> bool {
        other.to_str() == self.to_str()
    }
}

impl<const N: usize> core::fmt::Debug for tstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(&self.to_str())
    }
} // Debug impl

/*
///Convert tstr to &str slice
impl<IndexType, const N: usize> core::ops::Index<IndexType> for tstr<N>
where
    IndexType: core::slice::SliceIndex<str>,
{
    type Output = IndexType::Output;
    fn index(&self, index: IndexType) -> &Self::Output {
        &self.to_str()[index]
    }
} //impl Index
*/

impl<const N: usize> tstr<N> {
    /// returns a copy of the portion of the string, string could be truncated
    /// if indices are out of range. Similar to slice [start..end]
    pub fn substr(&self, start: usize, end: usize) -> tstr<N> {
        let mut chars = [0u8; N];
        let mut inds = self.char_indices();
        let len = self.len();
        if start >= len || end <= start {
            return tstr { chrs: chars };
        }
        chars[0] = (end - start) as u8;
        let (si, _) = inds.nth(start).unwrap();
        let last = if (end >= len) {
            len
        } else {
            match inds.nth(end - start - 1) {
                Some((ei, _)) => ei,
                None => len,
            } //match
        }; //let last =...
        chars[1..last - si + 1].copy_from_slice(&self.chrs[si + 1..last + 1]);
        /*
        for i in si..last
        {
          chars[i-si+1] = self.chrs[i+1];
        }
        */
        tstr { chrs: chars }
    } //substr
}

impl Add for str8 {
    type Output = str16;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str16 {
    type Output = str32;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str32 {
    type Output = str64;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str64 {
    type Output = str128;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str128 {
    type Output = str256;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str4 {
    type Output = str8;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str12 {
    type Output = str24;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str24 {
    type Output = str48;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str48 {
    type Output = str96;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

impl Add for str96 {
    type Output = str192;
    fn add(self, other: Self) -> Self::Output {
        let mut cat: Self::Output = self.resize();
        let slen = self.len();
        let olen = other.len();
        cat.chrs[slen + 1..slen + olen + 1].copy_from_slice(&other.chrs[1..olen + 1]);
        cat.chrs[0] = (slen + olen) as u8;
        cat
    }
} //Add

////////////// core::fmt::Write trait
/// Usage:
/// ```
///   use core::fmt::Write;
///   let mut s = str16::new();
///   let result = write!(&mut s,"hello {}, {}, {}",1,2,3);
///   /* or */
///   let s2 = str_format!(str32,"abx{}{}{}",1,2,3);
/// ```
impl<const N: usize> core::fmt::Write for tstr<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result //Result<(),core::fmt::Error>
    {
        if s.len() + self.len() > N - 1 {
            return Err(core::fmt::Error::default());
        }
        self.push(s);
        Ok(())
    } //write_str
} //core::fmt::Write trait
