//! Module for fstr type
#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]

#[cfg(feature = "std")]
extern crate std;
use crate::tiny_internal::*;
use crate::zero_terminated::*;
use core::cmp::{min, Ordering};
use core::ops::Add;
use std::eprintln;
use std::string::String;

/// **This type is only available with the `std` (or `fstr`) feature.**
/// A `fstr<N>` is a string of up to const N bytes, using a separate variable to store the length.
/// This type is not as memory-efficient as some other types such as str4-str256.  This is also the only type of the crate that does not support `no_std`.
#[derive(Copy, Clone, Eq)]
pub struct fstr<const N: usize> {
    chrs: [u8; N],
    len: usize, // length will be <=N
} //fstr
impl<const N: usize> fstr<N> {
    /// creates a new `fstr<N>` with given &str.  If the length of s exceeds
    /// N, the extra characters are ignored and a **warning is sent to stderr**.
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
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// returns maximum capacity in bytes
    pub fn capacity(&self) -> usize {
        N
    }

    /// converts fstr to an owned string
    pub fn to_string(&self) -> String {
        //self.to_str().to_owned()
        String::from(self.to_str())
        //self.chrs[0..self.len].iter().map(|x|{*x as char}).collect()
    }

    /// allows returns copy of u8 array underneath the fstr
    pub fn as_u8(&self) -> [u8; N] {
        self.chrs
    }

    /// converts fstr to &str using [std::str::from_utf8_unchecked].  Since
    /// fstr can only be built from valid utf8 sources, this function
    /// is safe.
    pub fn to_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.chrs[0..self.len]) }
    }
    /// same functionality as [fstr::to_str], but using [std::str::from_utf8]
    /// and may technically panic.
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
        self.push_str(s)
        /*
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
            sci += clen;
        }
        self.len = i;
        &s[sci..]
        */
    } //push

    /// alias for [fstr::push]
    pub fn push_str<'t>(&mut self, src: &'t str) -> &'t str {
        let srclen = src.len();
        let slen = self.len();
        let bytes = &src.as_bytes();
        let length = core::cmp::min(slen + srclen, N);
        let remain = if N >= (slen + srclen) {
            0
        } else {
            (srclen + slen) - N
        };
        let mut i = 0;
        while i < srclen && i + slen < N {
            self.chrs[slen + i] = bytes[i];
            i += 1;
        } //while
        self.len += i;
        &src[srclen - remain..]
    }

    /// pushes a single character to the end of the string, returning
    /// true on success.
    pub fn push_char(&mut self, c: char) -> bool {
        let clen = c.len_utf8();
        if self.len + clen > N {
            return false;
        }
        let mut buf = [0u8; 4]; // char buffer
        let bstr = c.encode_utf8(&mut buf);
        self.push(bstr);
        true
    } // push_char

    /// remove and return last character in string, if it exists
    pub fn pop_char(&mut self) -> Option<char> {
        if self.len() == 0 {
            return None;
        }
        let (ci, lastchar) = self.char_indices().last().unwrap();
        self.len = ci;
        Some(lastchar)
    } //pop

    /// returns the number of characters in the string regardless of
    /// character class
    pub fn charlen(&self) -> usize {
        self.to_str().chars().count()
    }

    /// returns the nth char of the fstr
    pub fn nth(&self, n: usize) -> Option<char> {
        self.to_str().chars().nth(n)
    }

    /// returns the nth byte of the string as a char.  This
    /// function should only be called, for example, on ascii strings.  It
    /// is designed to be quicker than [fstr::nth], and does not check array bounds or
    /// check n against the length of the string. Nor does it check
    /// if the value returned is a valid character.
    pub fn nth_bytechar(&self, n: usize) -> char {
        self.chrs[n] as char
    }
    /// alias for [Self::nth_bytechar] (for backwards compatibility)
    pub fn nth_ascii(&self, n: usize) -> char {
        self.chrs[n] as char
    }

    /// shortens the fstr in-place (mutates).  Note that n indicates
    /// a *character* position to truncate up to, not the byte position.
    //  If n is greater than the
    /// current length of the string in chars, this operation will have no effect.
    pub fn truncate(&mut self, n: usize) {
        if let Some((bi, c)) = self.to_str().char_indices().nth(n) {
            //self.chrs[bi] = 0;
            self.len = bi;
        }
        //if n<self.len {self.len = n;}
    }

    /// truncates string up to *byte* position n.  **Panics** if n is
    /// not on a character boundary, similar to [String::truncate]
    pub fn truncate_bytes(&mut self, n: usize) {
        if (n < self.len) {
            assert!(self.is_char_boundary(n));
            self.len = n
        }
    }

    /// Trims **in-place** trailing ascii whitespaces.  This function
    /// regards all bytes as single chars.  The operation panics if
    /// the resulting string does not end on a character boundary.
    pub fn right_ascii_trim(&mut self) {
        let mut n = self.len;
        while n > 0 && (self.chrs[n - 1] as char).is_ascii_whitespace() {
            //self.chrs[n-1] = 0;
            n -= 1;
        }
        assert!(self.is_char_boundary(n));
        self.len = n;
    } //right_trim

    /// resets string to empty string
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// in-place modification of ascii characters to lower-case, panics if
    /// the string is not ascii.
    pub fn make_ascii_lowercase(&mut self) {
        assert!(self.is_ascii());
        for b in &mut self.chrs[..self.len] {
            if *b >= 65 && *b <= 90 {
                *b |= 32;
            }
        }
    } //make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case, panics if
    /// the string is not ascii.
    pub fn make_ascii_uppercase(&mut self) {
        assert!(self.is_ascii());
        for b in &mut self.chrs[..self.len] {
            if *b >= 97 && *b <= 122 {
                *b -= 32;
            }
        }
    }

    /// Constructs a clone of this fstr but with only upper-case ascii
    /// characters.  This contrasts with [str::to_ascii_uppercase],
    /// which creates an owned String.
    pub fn to_ascii_upper(&self) -> Self {
        let mut cp = self.clone();
        cp.make_ascii_uppercase();
        cp
    }

    /// Constructs a clone of this fstr but with only lower-case ascii
    /// characters.  This contrasts with [str::to_ascii_lowercase],
    /// which creates an owned String.
    pub fn to_ascii_lower(&self) -> Self {
        let mut cp = *self;
        cp.make_ascii_lowercase();
        cp
    }

    /// Tests for ascii case-insensitive equality with another string.
    /// This function does not check if either string is ascii.
    pub fn case_insensitive_eq<TA>(&self, other: TA) -> bool
      where TA : AsRef<str>
      {
        if self.len() != other.as_ref().len() {
            return false;
        }
        let obytes = other.as_ref().as_bytes();
        for i in 0..self.len() {
            let mut c = self.chrs[i];
            if (c > 64 && c < 91) {
                c = c | 32;
            } // make lowercase
            let mut d = obytes[i];
            if (d > 64 && d < 91) {
                d = d | 32;
            } // make lowercase
            if c != d {
                return false;
            }
        } //for
        true
    } //case_insensitive_eq
} //impl fstr<N>

impl<const N: usize> std::ops::Deref for fstr<N> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.to_str()
    }
}

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
//generic, but "conflicts with crate 'core'
impl<const N: usize, TA:AsRef<str>> std::convert::From<TA> for fstr<N> {
    fn from(s: TA) -> fstr<N> {
        fstr::<N>::make(s.as_ref())
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

impl<const N: usize> std::fmt::Display for fstr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(&self.to_str())
    }
} // Debug impl

impl<const N: usize> fstr<N> {
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

////////////// core::fmt::Write trait
/// Usage:
/// ```
///   use fixedstr::*;
///   use std::fmt::Write;
///   let mut s = fstr::<32>::new();
///   let result = write!(&mut s,"hello {}, {}, {}",1,2,3);
///   /* or */
///   let s2 = str_format!(fstr<24>,"hello {}, {}, {}",1,2,3);
///   let s3 = try_format!(fstr::<4>,"hello {}, {}, {}",1,2,3); // returns None
/// ```
impl<const N: usize> core::fmt::Write for fstr<N> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result //Result<(),std::fmt::Error>
    {
        //if s.len() + self.len() > N {return Err(core::fmt::Error::default());}
        //self.push(s);
        let rest = self.push(s);
        if rest.len() > 0 {
            return Err(core::fmt::Error::default());
        }
        Ok(())
    } //write_str
} //core::fmt::Write trait

impl<const N: usize, TA:AsRef<str>> Add<TA> for fstr<N> {
    type Output = fstr<N>;
    fn add(self, other: TA) -> fstr<N> {
        let mut a2 = self;
        a2.push(other.as_ref());
        a2
    }
} //Add &str

impl<const N: usize> Add<&fstr<N>> for &str {
    type Output = fstr<N>;
    fn add(self, other: &fstr<N>) -> fstr<N> {
        let mut a2 = fstr::from(self);
        a2.push(other);
        a2
    }
} //Add &str on left

impl<const N: usize> Add<fstr<N>> for &str {
    type Output = fstr<N>;
    fn add(self, other: fstr<N>) -> fstr<N> {
        let mut a2 = fstr::from(self);
        a2.push(&other);
        a2
    }
} //Add &str on left

impl<const N: usize> core::hash::Hash for fstr<N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
} //hash
  /*  can't adopt because it affects type inference for .resize()
  impl<const N: usize, const M:usize> PartialEq<fstr<M>> for fstr<N> {
      fn eq(&self, other: &fstr<M>) -> bool {
         self.as_ref() == other.as_ref()
      }
  }
  */
impl<const N: usize> PartialEq for fstr<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<const N: usize> core::str::FromStr for fstr<N> {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() <= N {
            Ok(fstr::from(s))
        } else {
            Err("capacity exceeded")
        }
    }
}
