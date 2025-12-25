//! This module implements [zstr], which are zero-terminated strings of
//! fixed maximum lengths.  Each `zstr<N>` is represented underneath with
//! an u8-array of size N.  Compared to [crate::fstr], these strings
//! are more memory efficient but with some of the operations taking slightly
//! longer.  However, *all* bytes of the array following the string
//! are set to zero.  This allows the first zero-byte of the array to
//! be found by binary search, giving an O(log N) length function.
//!
//!Type zstr\<N\> can store strings consisting of up to N-1 bytes
//! whereas fstr\<N\> can store strings consisting of up to N bytes.
//! Also, itztr is assumed that the zstr may carray non-textual data and therefore
//! implements some of the traits differently.

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(feature = "alloc",not(feature = "std")))]
use alloc::string::String;

#[cfg(feature = "std")]
extern crate std;
#[cfg(feature = "std")]
use std::string::String;

#[cfg(feature = "std")]
use crate::fstr;

use crate::tstr;
use core::cmp::{min, Ordering};
use core::ops::Add;

/// `zstr<N>`: utf-8 strings of size up to N bytes. The strings are
/// zero-terminated with a single byte, with the additional requirement that
/// all bytes following the first zero are also zeros in the underlying array.
/// This allows for an O(log N) [zstr::len] function.  Note that
/// [utf8 encodings](https://www.ibm.com/docs/en/db2/11.5?topic=support-unicode-character-encoding)
/// of unicode characters allow single null bytes to be distinguished as
/// end-of-string.
#[derive(Copy, Clone, Eq)]
pub struct zstr<const N: usize> {
    chrs: [u8; N],
} //zstr
impl<const N: usize> zstr<N> {
    /// creates a new `zstr<N>` with given &str.  If the length of s exceeds
    /// N, the extra characters are ignored.
    /// This function is also called by
    /// several others including [zstr::from].
    pub fn make(s: &str) -> zstr<N> {
        let mut chars = [0u8; N];
        let bytes = s.as_bytes(); // &[u8]
        let mut i = 0;
        let limit = if N == 0 { 0 } else { min(N - 1, bytes.len()) };
        chars[..limit].clone_from_slice(&bytes[..limit]);
        zstr { chrs: chars }
    } //make

    /// alias for [zstr::make]
    #[inline]
    pub fn create(s: &str) -> zstr<N> {
        Self::make(s)
    }

    /// version of make that returns the original string in an `Err(_)` if
    /// truncation is requried, or in an `Ok(_)` if no truncation is required
    pub fn try_make(s: &str) -> Result<zstr<N>, &str> {
        if s.len() + 1 > N {
            Err(s)
        } else {
            Ok(zstr::make(s))
        }
    }

    /// creates an empty string, equivalent to zstr::default() but can also
    /// be called in a const context
    pub const fn new() -> zstr<N> {
        zstr {
          chrs: [0;N]
        }
    }

/// const constructor, to be called from const contexts.  However, as
/// const constructors are restricted from using iterators, it's slightly
/// better to call the non-const constructors in non-const contexts.
/// Truncates automatically.
    pub const fn const_make(s:&str) -> zstr<N> {
      let mut t = zstr::<N>::new();
      let mut len = s.len();
      if len+1>N { len = N-1; } // fix max length
      let bytes = s.as_bytes();
      let mut i = 0;
      while i<len {
        t.chrs[i] = bytes[i];
        i += 1;
      }
      t
    }//const_make

    /// version of `const_make` that does not truncate.
    pub const fn const_try_make(s:&str) -> Option<zstr<N>> {
      if s.len()+1>N {None}
      else { Some(zstr::const_make(s)) }
    }

    /// const function that
    /// creates a new `zstr<N>` with given `&[u8]` slice.  If the length of the
    /// slice exceeds N-1, the extra bytes are ignored.  All bytes of the slice
    /// following the first zero-byte are also ignored.
    /// **This operation does not check if the u8 slice is an utf8 source.**
    /// This function is unique to zstr and not available for the
    /// other string types in this crate.
    pub const fn from_raw(s: &[u8]) -> zstr<N> {
        let mut z = zstr { chrs: [0; N] };
        let mut i = 0;
        while i + 1 < N && i < s.len() && s[i] != 0 {
            z.chrs[i] = s[i];
            i += 1;
        }
        z
    } //from_raw

    /// Length of the string in bytes (consistent with [str::len]).
    /// This function uses binary search to find the first zero-byte
    /// and runs in O(log N) time for each `zstr<N>`.  This function
    /// can be called from a const context.
    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.blen()
    }

    /// Length of a `zstr<N>` string in bytes using O(n) linear search,
    /// may be useful when the string is of length n but n is known to be
    /// much smaller than N, or when the underlying array is corrupted.
    /// This function is const, and is unique to the zstr type and
    /// not available for other string types in this crate.
    pub const fn linear_len(&self) -> usize {
        let mut i = 0;
        while self.chrs[i] != 0 {
            i += 1;
        }
        return i;
    } //linear_len

    /// const function that checks that the underlying array of the zstr is
    /// properly zero-terminated, with no non-zero bytes after the first
    /// zero.  Returns false if there's a problem.
    pub const fn check_integrity(&self) -> bool {
        let mut n = self.linear_len();
        if n == N {
            return false;
        }
        while n < N {
            if self.chrs[n] != 0 {
                return false;
            }
            n += 1;
        } //while
        true
    } //check_integrity

    /// Guarantees that the underlying array of the zstr is
    /// properly zero-terminated, with no non-zero bytes after the first zero.
    pub fn clean(&mut self) {
        let mut n = self.linear_len();
        if n == N {
            self.chrs[n - 1] = 0;
        }
        while n < N {
            self.chrs[n] = 0;
            n += 1;
        } //while
    } //clean

    /// returns maximum capacity in bytes
    #[inline(always)]
    pub const fn capacity(&self) -> usize {
        if N == 0 {
            return 0;
        }
        N - 1
    }

    // new blen function uses binary search to find first 0 byte.
    const fn blen(&self) -> usize {
        let (mut min, mut max) = (0, N);
        let mut mid = 0;
        while min < max {
            //mid = (min + max) / 2;
            mid = min + (max-min)/2;   // no overflow, just in case
            if self.chrs[mid] == 0 {
                // go left
                max = mid;
            } else {
                // go right
                min = mid + 1;
            }
        } //while
        min
    } //blen, O(log N)

    /// converts zstr to an owned string
    #[cfg(any(feature = "alloc", feature = "std"))]
    pub fn to_string(&self) -> String {
        String::from(self.to_str())
    }

    /// returns slice of u8 array underneath the zstr, **including the terminating 0**
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.chrs[..self.blen() + 1]
    }

    /// returns mutable slice of the u8 array underneath, including the terminating zero.  **WARNING:** changing a byte to zero in the middle of the string is not enough to zero-terminate the string: the length calculation via binary search will become invalid. All bytes following the first zero must also be zeroed.  Use with care.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let n = self.blen()+1;
        &mut self.chrs[0..n]
    }

    /// returns slice of u8 array underneath the zstr without the terminating zero
    #[inline]
    pub fn as_bytes_non_terminated(&self) -> &[u8] {
        &self.chrs[..self.blen()]
    }

    /// converts zstr to &str using [core::str::from_utf8_unchecked].
    pub fn to_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.chrs[0..self.blen()]) }
    }
    /// checked version of [zstr::to_str], but may panic (calls `unwrap`)
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.chrs[0..self.blen()]).unwrap()
    }
    /// version of [zstr::as_str] that does not call `unwrap`
    pub fn as_str_safe(&self) -> Result<&str,core::str::Utf8Error> {
        core::str::from_utf8(&self.chrs[0..self.blen()])
    }
    

    /// changes a character at *character position* i to c.  This function
    /// requires that c is in the same character class (ascii or unicode)
    /// as the char being replaced.  It never shuffles the bytes underneath.
    /// The function returns true if the change was successful.
    pub fn set(&mut self, i: usize, c: char) -> bool {
        let ref mut cbuf = [0u8; 4];
        c.encode_utf8(cbuf);
        let clen = c.len_utf8();
        if let Some((bi, rc)) = self.to_str().char_indices().nth(i) {
            if clen == rc.len_utf8() {
                self.chrs[bi..bi + clen].clone_from_slice(&cbuf[..clen]);
                return true;
            }
        }
        return false;
    } //set

    /// version of [zstr::set] that assumes that the char is a single byte.
    /// Sets the char at the given *byte* index.
    /// Does not check for index bounds, but does check that the byte set is
    /// not zero. This function is designed to be fast.
    pub const fn set_byte_char(&mut self, i:usize, c:char) {
      if self.chrs[i]!=0 {self.chrs[i] = c as u8;}
    }

    /// adds chars to end of current string up to maximum size N-1 of `zstr<N>`,
    /// returns the portion of the push string that was NOT pushed due to
    /// capacity, so
    /// if "" is returned then all characters were pushed successfully.
    #[inline]
    pub fn push<'t>(&mut self, s: &'t str) -> &'t str {
        self.push_str(s)
    } //push

    /// alias for [zstr::push]
    pub fn push_str<'t>(&mut self, src: &'t str) -> &'t str {
        let srclen = src.len();
        let slen = self.blen();
        let bytes = &src.as_bytes();
        let length = core::cmp::min(slen + srclen, N - 1);
        let remain = if N - 1 >= (slen + srclen) {
            0
        } else {
            (srclen + slen) - N + 1
        };
        let mut i = 0;
        while i < srclen && i + slen + 1 < N {
            self.chrs[slen + i] = bytes[i];
            i += 1;
        } //while
        &src[srclen - remain..]
    } //push_str

    /// pushes a single character to the end of the string, returning
    /// true on success.
    pub fn push_char(&mut self, c: char) -> bool {
        let clen = c.len_utf8();
        let slen = self.len();
        if slen + clen >= N {
            return false;
        }
        let mut buf = [0u8; 4]; // char buffer
        c.encode_utf8(&mut buf);
        for i in 0..clen {
            self.chrs[slen + i] = buf[i];
        }
        self.chrs[slen + clen] = 0;
        true
    } // push_char

    /// remove and return last character in string, if it exists
    pub fn pop_char(&mut self) -> Option<char> {
        if self.chrs[0] == 0 {
            return None;
        } // length zero
        let (ci, lastchar) = self.char_indices().last().unwrap();
        //self.chrs[ci]=0;
        let mut cm = ci;
        while cm < N && self.chrs[cm] != 0 {
            self.chrs[cm] = 0;
            cm += 1;
        }
        Some(lastchar)
    } //pop

    /// returns the number of characters in the string regardless of
    /// character class.  For strings with only single-byte chars,
    /// call [Self::len] instead.
    pub fn charlen(&self) -> usize {
        self.to_str().chars().count()
    }

    /// returns the nth character of the zstr
    pub fn nth(&self, n: usize) -> Option<char> {
        self.to_str().chars().nth(n)
        //if n<self.len() {Some(self.chrs[n] as char)} else {None}
    }

    /// returns the nth byte of the string as a char.  This
    /// function should only be called on, for example, ascii strings.  It
    /// is designed to be quicker than [zstr::nth], and does not check array bounds or
    /// check n against the length of the string. Nor does it check
    /// if the value returned is a valid character.
    pub const fn nth_bytechar(&self, n: usize) -> char {
        self.chrs[n] as char
    }
    /// alias for nth_bytechar (for backwards compatibility)
    pub const fn nth_ascii(&self, n: usize) -> char {
        self.chrs[n] as char
    }

    /// determines if string is an ascii string
    pub fn is_ascii(&self) -> bool {
        self.to_str().is_ascii()
    }

    /// shortens the zstr in-place. Note that n indicates
    /// a *character* position to truncate up to, not the byte position.
    /// If n is greater than the
    /// current character length of the string, this operation will have no effect.
    /// This is not an O(1) operation.
    pub fn truncate(&mut self, n: usize) // n is char position, not binary position
    {
        if let Some((bi, c)) = self.to_str().char_indices().nth(n) {
            let mut bm = bi;
            while bm < N && self.chrs[bm] != 0 {
                self.chrs[bm] = 0;
                bm += 1;
            }
            //self.chrs[bi] = 0;
        }
    }

    /// truncates string up to *byte* position n.  **Panics** if n is
    /// not on a character boundary truncate on owned Strings.
    /// Although faster than [zstr::truncate], this function is still
    /// not O(1) because it zeros the truncated bytes.  This is a calculated
    /// tradeoff with a O(log N) [zstr::len] function, which is expected to
    /// have greater impact.
    pub fn truncate_bytes(&mut self, n: usize) {
        if n < N {
            assert!(self.is_char_boundary(n));
            //self.chrs[n] = 0;
            let mut m = n;
            while m < N && self.chrs[m] != 0 {
                self.chrs[m] = 0;
                m += 1;
            }
        }
    } //truncate_bytes

    /// Trims **in-place** trailing ascii whitespaces.  This function
    /// regards all bytes as single chars.  The operation panics if
    /// the resulting string does not end on a character boundary.
    pub fn right_ascii_trim(&mut self) {
        let mut n = self.blen();
        while n > 0 && (self.chrs[n - 1] as char).is_ascii_whitespace() {
            self.chrs[n - 1] = 0;
            n -= 1;
        }
        assert!(self.is_char_boundary(n));
    } //right_trim

    /// Reverses **in-place** a string where characters are single bytes.
    /// The result of this operation on non single-byte chars is unpredicatable.
    /// This function is only available for the zstr type and not for other
    /// string types in this crate.
    pub fn reverse_bytes(&mut self) {
        let n = self.blen();
        let m = n / 2;
        let mut i = 0;
        while i < m {
            self.chrs.swap(i, n - i - 1);
            i += 1;
        }
    } //reverse_bytes

    /// in-place swap of bytes i and k, returns true on success and
    /// false if indices are out of bounds.  This function is only available
    /// for zstr strings and not for other string types in this crate.
    pub fn swap_bytes(&mut self, i: usize, k: usize) -> bool {
        if i != k && i < N && k < N && self.chrs[i] != 0 && self.chrs[k] != 0 {
            self.chrs.swap(i, k);
            true
        } else {
            false
        }
    } //swap_bytes

    /// resets string to empty string
    pub fn clear(&mut self) {
        self.chrs = [0; N];
        //self.chrs[0]=0;
    }

    /// in-place modification of ascii characters to lower-case, panics
    /// if the string is not ascii.
    pub fn make_ascii_lowercase(&mut self) {
        assert!(self.is_ascii());
        for b in &mut self.chrs {
            if *b == 0 {
                break;
            } else if *b >= 65 && *b <= 90 {
                *b += 32;
            }
        }
    } //make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case, panics if
    /// the string is not ascii.
    pub fn make_ascii_uppercase(&mut self) {
        assert!(self.is_ascii());
        for b in &mut self.chrs {
            if *b == 0 {
                break;
            } else if *b >= 97 && *b <= 122 {
                *b -= 32;
            }
        }
    }

    /// Constructs a clone of this zstr but with only upper-case ascii
    /// characters.  This contrasts with [str::to_ascii_uppercase],
    /// which creates an owned String.
    pub fn to_ascii_upper(&self) -> Self {
        let mut cp = self.clone();
        cp.make_ascii_uppercase();
        cp
    }

    /// Constructs a clone of this zstr but with only lower-case ascii
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
    where
        TA: AsRef<str>,
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

    // new for 0.5.0
    /// converts zstr to a raw pointer to the first byte
    pub const fn to_ptr(&self) -> *const u8 {
        let ptr = &self.chrs[0] as *const u8;
        ptr
        //ptr as *const char
    }

    /// Converts zstr to a mutable pointer to the first byte.  Although
    /// technically not 'unsafe', this function can be used to alter
    /// the underlying representation so that there are non-zero values
    /// after the first zero.  Use with care.
    pub fn to_ptr_mut(&mut self) -> *mut u8 {
        &mut self.chrs[0] as *mut u8
    }

    /// Creates a zstr from a raw pointer by copying bytes until the
    /// first zero is encountered or when maximum capacity (N-1) is reached.
    pub unsafe fn from_ptr(mut ptr: *const u8) -> Self {
        let mut z = zstr::new();
        let mut i = 0;
        while *ptr != 0 && i + 1 < N {
            z.chrs[i] = *ptr;
            ptr = (ptr as usize + 1) as *const u8;
            i += 1;
        } //while
        z.chrs[i] = 0;
        z
    } //unsafe from_raw

    /// Creates a [core::ffi::CStr]
    pub fn to_cstr(&self) -> &core::ffi::CStr {
      &core::ffi::CStr::from_bytes_until_nul(self.chrs.as_slice()).unwrap()
    }

    /// Converts from a [core::ffi::CStr], truncates as needed
    pub fn from_cstr(cstr:&core::ffi::CStr) -> Self {
      Self::from_raw(cstr.to_bytes_with_nul())
    }

    /// Decodes a UTF-16 encodeded slice. If a decoding error is encountered
    /// or capacity exceeded, an `Err(s)` is returned where s is the
    /// the encoded string up to the point of the error.
    pub fn from_utf16(v: &[u16]) -> Result<Self, Self> {
        let mut s = Self::new();
        let mut len = 0; // track length without calling zstr::len
        let mut buf = [0u8; 4];
        for c in char::decode_utf16(v.iter().cloned()) {
            if let Ok(c1) = c {
                let cbytes = c1.encode_utf8(&mut buf);
                let clen = c1.len_utf8();
                len += clen;
                if len + 1 > N {
                    s.chrs[len - clen] = 0;
                    return Err(s);
                } else {
                    s.chrs[len - clen..len].copy_from_slice(&buf[..clen]);
                }
            } else {
                s.chrs[len] = 0;
                return Err(s);
            }
        }
        s.chrs[len] = 0;
        Ok(s)
    } //from_utf16
} //impl zstr<N>

impl<const N: usize> core::ops::Deref for zstr<N> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.to_str()
    }
}

impl<const N: usize> core::convert::AsRef<str> for zstr<N> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}
impl<const N: usize> core::convert::AsMut<str> for zstr<N> {
    fn as_mut(&mut self) -> &mut str {
        let blen = self.blen();
        unsafe { core::str::from_utf8_unchecked_mut(&mut self.chrs[0..blen]) }
    }
}

impl<T: AsRef<str> + ?Sized, const N: usize> core::convert::From<&T> for zstr<N> {
    fn from(s: &T) -> zstr<N> {
        zstr::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> core::convert::From<&mut T> for zstr<N> {
    fn from(s: &mut T) -> zstr<N> {
        zstr::make(s.as_mut())
    }
}

#[cfg(feature = "std")]
impl<const N: usize> std::convert::From<std::string::String> for zstr<N> {
    fn from(s: std::string::String) -> zstr<N> {
        zstr::<N>::make(&s[..])
    }
}
#[cfg(feature = "std")]
impl<const N: usize, const M: usize> std::convert::From<fstr<M>> for zstr<N> {
    fn from(s: fstr<M>) -> zstr<N> {
        zstr::<N>::make(s.to_str())
    }
}

impl<const N: usize, const M: usize> core::convert::From<tstr<M>> for zstr<N> {
    fn from(s: tstr<M>) -> zstr<N> {
        zstr::<N>::make(s.to_str())
    }
}

impl<const N: usize> core::cmp::PartialOrd for zstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.chrs[0..self.blen()].cmp(other.chrs[0..other.blen()]))
        Some(self.cmp(other))
    }
}

impl<const N: usize> core::cmp::Ord for zstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.chrs[0..self.blen()].cmp(&other.chrs[0..other.blen()])
    }
}

impl<const M: usize> zstr<M> {
    /// converts an zstr\<M\> to an zstr\<N\>. If the length of the string being
    /// converted is greater than N-1, the extra characters are ignored.
    /// This operation produces a copy (non-destructive).
    /// Example:
    ///```ignore
    ///  let s1:zstr<8> = zstr::from("abcdefg");
    ///  let s2:zstr<16> = s1.resize();
    ///```
    pub fn resize<const N: usize>(&self) -> zstr<N> {
        let slen = self.blen();
        let length = if slen + 1 < N {
            slen
        } else if N == 0 {
            0
        } else {
            N - 1
        };
        let mut chars = [0u8; N];
        chars[..length].clone_from_slice(&self.chrs[..length]);
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

impl<const N: usize> core::fmt::Display for zstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        //write!(f, "{}", self.to_str())
        f.pad(self.to_str())
    }
}

impl<const N: usize> PartialEq<&str> for zstr<N> {
    fn eq(&self, other: &&str) -> bool {
        self.to_str() == *other // see below
    } //eq
}
impl<const N: usize> PartialEq<&str> for &zstr<N> {
    fn eq(&self, other: &&str) -> bool {
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
    } //eq
}
impl<'t, const N: usize> PartialEq<zstr<N>> for &'t str {
    fn eq(&self, other: &zstr<N>) -> bool {
        &other.to_str() == self
    }
}
impl<'t, const N: usize> PartialEq<&zstr<N>> for &'t str {
    fn eq(&self, other: &&zstr<N>) -> bool {
        &other.to_str() == self
    }
}

/// defaults to empty string
impl<const N: usize> Default for zstr<N> {
    fn default() -> Self {
        zstr::<N>::make("")
    }
}
#[cfg(feature = "std")]
impl<const N: usize, const M: usize> PartialEq<zstr<N>> for fstr<M> {
    fn eq(&self, other: &zstr<N>) -> bool {
        other.to_str() == self.to_str()
    }
}

#[cfg(feature = "std")]
impl<const N: usize, const M: usize> PartialEq<fstr<N>> for zstr<M> {
    fn eq(&self, other: &fstr<N>) -> bool {
        other.to_str() == self.to_str()
    }
}

#[cfg(feature = "std")]
impl<const N: usize, const M: usize> PartialEq<&fstr<N>> for zstr<M> {
    fn eq(&self, other: &&fstr<N>) -> bool {
        other.to_str() == self.to_str()
    }
}

impl<const N: usize> core::fmt::Debug for zstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(&self.to_str())
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
        zstr { chrs: chars }
    } //substr
}

/// [zstr] type aliases for convenience
pub type ztr8 = zstr<8>;
pub type ztr16 = zstr<16>;
pub type ztr32 = zstr<32>;
pub type ztr64 = zstr<64>;
pub type ztr128 = zstr<128>;

////////////// core::fmt::Write trait
/// Usage:
/// ```
///  # use fixedstr::*;
///   use core::fmt::Write;
///   let mut s = zstr::<32>::new();
///   let result = write!(&mut s,"hello {}, {}, {}",1,2,3);
///   /* or */
///   let s2 = str_format!(zstr<16>,"abx{}{}{}",1,2,3);
/// ```
impl<const N: usize> core::fmt::Write for zstr<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result //Result<(),core::fmt::Error>
    {
        if s.len() + self.len() + 1 > N {
            return Err(core::fmt::Error::default());
        }
        self.push(s);
        Ok(())
    } //write_str
} //core::fmt::Write trait

#[cfg(feature = "experimental")]
mod special_index {
    use super::*;
    use core::ops::{Range, RangeFrom, RangeFull, RangeTo};
    use core::ops::{RangeInclusive, RangeToInclusive};

    impl<const N: usize> core::ops::Index<Range<usize>> for zstr<N> {
        type Output = str;
        fn index(&self, index: Range<usize>) -> &Self::Output {
            &self.to_str()[index]
        }
    } //impl Index
    impl<const N: usize> core::ops::Index<RangeTo<usize>> for zstr<N> {
        type Output = str;
        fn index(&self, index: RangeTo<usize>) -> &Self::Output {
            &self.to_str()[index]
        }
    } //impl Index
    impl<const N: usize> core::ops::Index<RangeFrom<usize>> for zstr<N> {
        type Output = str;
        fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
            &self.to_str()[index]
        }
    } //impl Index
    impl<const N: usize> core::ops::Index<RangeInclusive<usize>> for zstr<N> {
        type Output = str;
        fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
            &self.to_str()[index]
        }
    } //impl Index
    impl<const N: usize> core::ops::Index<RangeToInclusive<usize>> for zstr<N> {
        type Output = str;
        fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
            &self.to_str()[index]
        }
    } //impl Index
    impl<const N: usize> core::ops::Index<RangeFull> for zstr<N> {
        type Output = str;
        fn index(&self, index: RangeFull) -> &Self::Output {
            &self.to_str()[index]
        }
    } //impl Index

    // must include above to have the following ..

    ///The implementation of `Index<usize>` for types `zstr<N>` is different
    ///from that of `fstr<N>` and `tstr<N>`, to allow `IndexMut` on a single
    ///byte.  The type returned by this trait is &u8, not &str.  This special
    ///trait is only available with the `experimental` feature.
    impl<const N: usize> core::ops::Index<usize> for zstr<N> {
        type Output = u8;
        fn index(&self, index: usize) -> &Self::Output {
            &self.chrs[index]
        }
    } //impl Index

    /// **This trait is provided with caution**, and only with the
    /// **`experimental`** feature, as it allows arbitrary changes
    /// to the bytes of the string.  In particular, the string can become
    /// corrupted if a premature zero-byte is created using this function,
    /// which invalidates the [Self::len] function.  Several other operations
    /// such as [Self::push] depend on a correct length function.
    impl<const N: usize> core::ops::IndexMut<usize> for zstr<N> {
        fn index_mut(&mut self, index: usize) -> &mut Self::Output {
            let ln = self.blen();
            if index >= ln {
                panic!("index {} out of range ({})", index, ln);
            }
            &mut self.chrs[index]
        }
    } //impl IndexMut
} // special_index submodule (--features experimental)

impl<const N: usize, TA: AsRef<str>> Add<TA> for zstr<N> {
    type Output = zstr<N>;
    fn add(self, other: TA) -> zstr<N> {
        let mut a2 = self;
        a2.push(other.as_ref());
        a2
    }
} //Add &str
  /*
  impl<const N: usize> Add<&str> for zstr<N> {
      type Output = zstr<N>;
      fn add(self, other: &str) -> zstr<N> {
          let mut a2 = self;
          a2.push(other);
          a2
      }
  } //Add &str
  */
impl<const N: usize> Add<&zstr<N>> for &str {
    type Output = zstr<N>;
    fn add(self, other: &zstr<N>) -> zstr<N> {
        let mut a2 = zstr::from(self);
        a2.push(other);
        a2
    }
} //Add &str on left

impl<const N: usize> Add<zstr<N>> for &str {
    type Output = zstr<N>;
    fn add(self, other: zstr<N>) -> zstr<N> {
        let mut a2 = zstr::from(self);
        a2.push(&other);
        a2
    }
} //Add &str on left

impl<const N: usize> core::hash::Hash for zstr<N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
} //hash
  /*
  impl<const N: usize, const M:usize> core::cmp::PartialEq<zstr<M>> for zstr<N> {
      fn eq(&self, other: &zstr<M>) -> bool {
         self.as_ref() == other.as_ref()
      }
  }
  */

impl<const N: usize> core::cmp::PartialEq for zstr<N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl<const N: usize> core::str::FromStr for zstr<N> {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < N {
            Ok(zstr::from(s))
        } else {
            Err("capacity exceeded")
        }
    }
}


/// Iterator over a [zstr]`<N>` in `CS`-size `&[u8]` slices,
/// except for possibly the last slice.  The last slice may also be
/// zero-terminated. 'CS' must be non-zero.
#[cfg(feature = "experimental")]
pub struct ChunkyIter<'t, const N:usize, const CS:usize> {
  bur : &'t [u8;N],
  index : usize,
}
#[cfg(feature = "experimental")]
impl<'t, const N:usize, const CS:usize> Iterator for ChunkyIter<'t,N,CS> {
  type Item = &'t [u8];
  fn next(&mut self) -> Option<Self::Item> {
    if CS==0 || self.index + 1 > N || self.bur[self.index]==0 { None }
    else {
       self.index += CS;
       Some(&self.bur[self.index-CS .. min(N,self.index)])
    }
  }//next
} // impl Iterator for ChunkyIter

#[cfg(feature = "experimental")]
impl<const N:usize> zstr<N> {
  /// Creates a [ChunkyIter] iterator over `&[u8]` slices of fixed size `CS`,
  /// except for the final slice, which may also be zero-terminated.
  pub fn chunky_iter<'t,const CS:usize>(&'t self) -> ChunkyIter<'t,N,CS> {
    ChunkyIter {
      bur : &self.chrs,
      index : 0,
    }
  }//chunk_iter
}
