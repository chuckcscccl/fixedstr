#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
//! fixed strings with circular-queue backing

use core::cmp::{min, Ordering, PartialOrd};
#[cfg(not(feature = "no-alloc"))]
extern crate alloc;
use core::ops::Add;

/// **This type is only available with the `circular-str` option.**
/// A *circular string* is represented underneath by a fixed-size u8
/// array arranged as a circular queue. The string can wrap around
/// either end and thus become internally non-contiguous.
/// This allows for efficient implementations of operations such as
/// push, trim *in front* of the string.  However, `Deref<str>` is not
/// implemented as it cannot be done efficiently.  Instead, the
/// [cstr::to_strs] function returns a pair of string slices, the second
/// of which is non-empty if the string is not contiguous.  Additionally,
/// only single-byte characters are currently allowed, although this might
/// change in the future by using a "ghost vector" at the end of the array.
/// An iterator [cstr::chars] is provided over all single-byte chars, which
/// also forms the foundation of other traits such as Eq, Ord, Hash, etc.
/// The Serialization (serde) and no-std options are both supported.
///
/// Each `cstr<N>` can hold up to N bytes and the maximum N is 65535.
/// **Values of N that are exact powers of 2 are recommended** to speed up
/// the `%` operation for computing indices in a ciruclar queue.
///
/// Examples:
/// ```
///  # use fixedstr::*;
///  let mut cb = cstr::<16>::make("abc123");
///  cb.push_str("xyz");
///  cb.push_front("9876");
///  assert_eq!(cb.pop_char().unwrap(), 'z');
///  assert_eq!(cb.pop_char_front().unwrap(), '9');
///  cb.push_str_front("000");
///  assert_eq!(cb.len(),14);
///  assert!(&cb == "000876abc123xy");
///  cb.truncate_left(10);
///  assert_eq!(&cb,"23xy");
///  cb.push_str("ijklmno  ");
///  cb.push_char_front(' ');
///  assert!(!cb.is_contiguous());
///  cb.trim_whitespaces();
///  assert!("23xyijklmno" == &cb);
///  assert!(&cb < "4abc");   // Ord trait
///  let mut a = cstr8::from("abc");
///  let ba:cstr8 = "123" + a; // concat &str on the left efficiently
///  assert_eq!(ba,"123abc");
/// ```
#[derive(Copy, Clone)]
pub struct cstr<const N: usize = 32> {
    chrs: [u8; N],
    front: u16,
    len: u16,
} //cstr

impl<const N: usize> cstr<N> {
    /// create `cstr` from `&str` with silent truncation; panics if
    /// N is greater than 65535
    pub fn make(src: &str) -> cstr<N> {
        if N < 1 || N > 65535 {
            panic!("cstr strings are limited to a capacity between 1 and 65535");
        }
        let mut m = cstr::<N>::new();
        let length = core::cmp::min(N, src.len());
        m.chrs[..length].copy_from_slice(&src.as_bytes()[..length]);
        m.len = length as u16;
        m
    } //make

    /// version of make that also panics if the input string is not ascii.
    pub fn from_ascii(src: &str) -> cstr<N> {
        if N < 1 || N > 65535 {
            panic!("cstr strings are limited to a maximum capacity of 65535");
        }
        if !src.is_ascii() {
            panic!("cstr string is not ascii");
        }
        let mut m = cstr::<N>::new();
        let length = core::cmp::min(N, src.len());
        m.chrs[..length].copy_from_slice(&src.as_bytes()[..length]);
        m.len = length as u16;
        m
    } //from_ascii

    /// version of make that does not truncate: returns original str slice
    /// as error.  Also checks if N is no greater than 65535 without panic.
    pub fn try_make(src: &str) -> Result<cstr<N>, &str> {
        let length = src.len();
        if length > N || N > 65535 || N < 1 {
            return Err(src);
        }
        let mut m = cstr::new();
        m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
        m.len = length as u16;
        Ok(m)
    } //try_make

    /// version of `try_make` that also checks if the input string is ascii.
    pub fn try_make_ascii(src: &str) -> Option<cstr<N>> {
        let length = src.len();
        if length > N || N > 65535 || N < 1 || !src.is_ascii() {
            return None;
        }
        let mut m = cstr::new();
        m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
        m.len = length as u16;
        Some(m)
    } //try_make

    /// version of make that returns a pair consisting of the made
    /// `cstr` and the remainder `&str` that was truncated; panics if
    /// N is greater than 65535 (but does not check for ascii strings)
    pub fn make_remainder(src: &str) -> (cstr<N>, &str) {
        if N > 65535 || N < 1 {
            panic!("cstr strings are limited to a capacity between 1 and 65535");
        }
        let mut m = cstr::new();
        let length = core::cmp::min(N, src.len());
        m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
        m.len = length as u16;
        (m, &src[length..])
    } //try_make

    /// make from a pair of str slices, does not truncate, and checks that
    /// N is not greater than 65535 without panic.  The returned cstr will
    /// be contiguous.
    pub fn from_pair(left: &str, right: &str) -> Option<cstr<N>> {
        let (llen, rlen) = (left.len(), right.len());
        if llen + rlen > N || N > 65535 || N < 1 {
            return None;
        }
        let mut m = cstr::new();
        m.len = (llen + rlen) as u16;
        m.chrs[..llen].copy_from_slice(&left.as_bytes()[..llen]);
        m.chrs[llen..].copy_from_slice(&right.as_bytes()[llen..]);
        Some(m)
    } //from_pair

    /// checks if the underlying representation of the string is contiguous
    /// (without wraparound).
    #[inline(always)]
    pub fn is_contiguous(&self) -> bool {
        (self.front as usize + self.len as usize) <= N
    }

    /// resets the internal representation of the cstr so that it is
    /// represented contiguously, without wraparound. **Calling this function
    /// has O(n) cost both in terms of speed and memory** as
    /// it requires a secondary buffer as well as copying.**
    pub fn reset(&mut self) {
        if self.front == 0 {
            return;
        }
        let mut mhrs = [0; N];
        for i in 0..self.len as usize {
            mhrs[i] = self.chrs[self.index(i)];
        }
        self.chrs = mhrs;
        self.front = 0;
    } //reset

    /// clears string to empty string
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// resets string to empty string and clears underlying buffer to contain
    /// all zeros.
    pub fn zero(&mut self) {
        self.chrs = [0; N];
        self.front = 0;
        self.len = 0;
    }

    /// guarantees a contiguous underlying representation of the string.
    /// This is an O(n) operation.
    pub fn make_contiguous(&mut self) {
        if !self.is_contiguous() {
            self.reset();
        }
    }

    /// returns the nth char of the fstr.  Since only single-byte characters
    /// are currently supported by the cstr type, this function is the same
    /// as [Self::nth_bytechar] except that n is checked against the length of the
    /// string.
    pub fn nth(&self, n: usize) -> Option<char> {
        if n < self.len as usize {
            Some(self.chrs[self.index(n)] as char)
        } else {
            None
        }
    }

    /// returns the nth byte of the string as a char, does not check n
    /// against length of array
    #[inline]
    pub fn nth_bytechar(&self, n: usize) -> char {
        self.chrs[self.index(n)] as char
    }

    /// sets the nth byte of the string to the supplied character.
    /// the character must fit in a single byte.  Returns true on success.
    pub fn set(&mut self, n: usize, c: char) -> bool {
        if c.len_utf8() > 1 || n >= self.len as usize {
            false
        } else {
            self.chrs[self.index(n)] = c as u8;
            true
        }
    } //set

    /// pushes given string to the end of the string, returns remainder
    pub fn push_str<'t>(&mut self, src: &'t str) -> &'t str {
        let srclen = src.len();
        let slen = self.len as usize;
        let bytes = &src.as_bytes();
        let length = core::cmp::min(slen + srclen, N);
        let remain = if N > (slen + srclen) {
            0
        } else {
            (srclen + slen) - N
        };
        let mut i = 0;
        while i < srclen && i + slen < N {
            self.chrs[self.index(slen + i)] = bytes[i];
            i += 1;
        } //while
        self.len += i as u16;
        &src[srclen - remain..]
    } //push_str

    /// Pushes string to the **front** of the string, returns remainder.
    /// because of the circular-queue backing, this operation has the same
    /// cost as pushing to the back of the string ([Self::push_str]).
    /// This function does not check if the input string is ascii.
    pub fn push_front<'t>(&mut self, src: &'t str) -> &'t str {
        let srclen = src.len();
        let slen = self.len as usize;
        let bytes = &src.as_bytes();
        let length = core::cmp::min(slen + srclen, N);
        let remain = if N >= (slen + srclen) {
            0
        } else {
            (srclen + slen) - N
        };
        let mut i = 0;
        while i < srclen && i + slen < N {
            //self.front =(self.front + (N as u16) -1) % (N as u16);
            self.front = ((self.front as usize + N - 1) % N) as u16;
            self.chrs[self.front as usize] = bytes[srclen - 1 - i];
            i += 1;
        } //while
        self.len += i as u16;
        &src[..remain]
    } //push_front

    /// alias for [Self::push_front]
    pub fn push_str_front<'t>(&mut self, src: &'t str) -> &'t str {
        self.push_front(src)
    }

    /// Pushes a single character to the end of the string, returning
    /// true on success.  This function checks if the given character
    /// occupies a single-byte.
    pub fn push_char(&mut self, c: char) -> bool {
        let clen = c.len_utf8();
        if clen > 1 || self.len as usize + clen > N {
            return false;
        }
        let mut buf = [0u8; 4]; // char buffer
        let bstr = c.encode_utf8(&mut buf);
        self.push_str(bstr);
        true
    } // push_char

    /// Pushes a single character to the front of the string, returning
    /// true on success.  This function checks if the given character
    /// occupies a single-byte.
    pub fn push_char_front(&mut self, c: char) -> bool {
        let clen = c.len_utf8();
        if clen > 1 || self.len as usize + clen > N {
            return false;
        }
        let newfront = (self.front as usize + N - 1) % N;
        self.chrs[newfront] = c as u8;
        self.front = newfront as u16;
        self.len += 1;
        true
    } //push_char_front

    /// remove and return last character in string, if it exists
    pub fn pop_char(&mut self) -> Option<char> {
        if self.len() == 0 {
            return None;
        }
        let lasti = ((self.front + self.len - 1) as usize) % N;
        let firstchar = self.chrs[lasti] as char;
        self.len -= 1;
        Some(firstchar)
        /*
        let (l,r) = self.to_strs();
        let right = if r.len()>0 {r} else {l};
        let (ci,lastchar) = right.char_indices().last().unwrap();
        self.len = if r.len()>0 {(l.len() + ci) as u16} else {ci as u16};
        Some(lastchar)
        */
    } //pop

    /// remove and return first character in string, if it exists
    pub fn pop_char_front(&mut self) -> Option<char> {
        if self.len() == 0 {
            return None;
        }
        let firstchar = self.chrs[self.front as usize] as char;
        self.front = self.index16(1);
        self.len -= 1;
        Some(firstchar)
    } //pop_char_front

    /// alias for [Self::truncate]
    pub fn truncate_right(&mut self, n: usize) {
        if (n < self.len as usize) {
            self.len = n as u16;
        }
    }

    /// right-truncates string up to byte position n. Only the first
    /// n bytes will be kept.
    /// No effect if n is greater than or equal to the length of the string.
    #[inline]
    pub fn truncate(&mut self, n: usize) {
        self.truncate_right(n);
    }

    /// left-truncates string up to byte position n: that is, the first
    /// n bytes will be truncated.  Because of the circular queue backing,
    /// this is an O(1) operation.
    /// No effect if n is greater than the length of the string.
    pub fn truncate_left(&mut self, n: usize) {
        if (n > 0 && n <= self.len as usize) {
            /*
            let (a,b) = self.to_strs();
            if n<a.len() {
              assert!(a.is_char_boundary(n));
            }
            else {
              assert!(b.is_char_boundary(n-a.len()));
            }
            */
            self.front = self.index16(n as u16);
            self.len -= n as u16;
        }
    } //truncate_left

    /// alias for `truncate_left`
    pub fn truncate_front(&mut self, n: usize) {
        self.truncate_left(n);
    }

    /// finds the position of first character that satisfies given predicate
    pub fn find<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(char) -> bool,
    {
        let (a, b) = self.to_strs();
        if let Some(pos) = a.find(|x: char| predicate(x)) {
            Some(pos)
        } else if let Some(pos) = b.find(|x: char| predicate(x)) {
            Some(a.len() + pos)
        } else {
            None
        }
    } //find

    /// finds the position of last character that satisfies given predicate
    pub fn rfind<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(char) -> bool,
    {
        let (a, b) = self.to_strs();
        if let Some(pos) = b.find(|x: char| predicate(x)) {
            Some(a.len() + pos)
        } else if let Some(pos) = a.find(|x: char| predicate(x)) {
            Some(pos)
        } else {
            None
        }
    } //find

    /// finds position of first matching substring
    pub fn find_substr(&self, s: &str) -> Option<usize> {
        let (a, b) = self.to_strs();
        if let Some(pos) = a.find(s) {
            return Some(pos);
        }
        if s.len() > 1 {
            //check middle
            for i in 0..s.len() - 1 {
                let mid = s.len() - i - 1;
                if a.ends_with(&s[..mid]) && b.starts_with(&s[mid..]) {
                    return Some(a.len() - mid);
                }
            } // for each intermediate position
        }
        if let Some(pos) = b.find(s) {
            return Some(a.len() + pos);
        } else {
            None
        }
    } //find_substr

    /// finds position of last matching substring
    pub fn rfind_substr(&self, s: &str) -> Option<usize> {
        let (a, b) = self.to_strs();
        if let Some(pos) = b.find(s) {
            return Some(a.len() + pos);
        }
        if s.len() > 1 {
            // check middle
            for i in 0..s.len() - 1 {
                let mid = s.len() - i - 1;
                if b.starts_with(&s[mid..]) && a.ends_with(&s[..mid]) {
                    return Some(a.len() - mid);
                }
            } //for
        }
        if let Some(pos) = a.find(s) {
            Some(pos)
        } else {
            None
        }
    } //find_substr

    /// **in-place** trimming of white spaces at the front of the string
    pub fn trim_left(&mut self) {
        let (a, b) = self.to_strs();
        let offset;
        if let Some(i) = a.find(|c: char| !c.is_whitespace()) {
            offset = i as u16;
        } else if let Some(k) = b.find(|c: char| !c.is_whitespace()) {
            offset = (a.len() + k) as u16;
        } else {
            offset = (a.len() + b.len()) as u16;
        }
        self.front = self.index16(offset); //((self.front as usize + offset)%N) as u16;
        self.len -= offset;
    } //trim_left

    /// **in-place** trimming of white spaces at the end of the string
    pub fn trim_right(&mut self) {
        let (a, b) = self.to_strs();
        let offset;
        if b.len() == 0 {
            if let Some(k) = a.rfind(|c: char| !c.is_whitespace()) {
                offset = a.len() - k - 1;
            } else {
                offset = a.len();
            }
        }
        //contiguous
        else if let Some(i) = b.rfind(|c: char| !c.is_whitespace()) {
            offset = b.len() - i - 1;
        } else if let Some(k) = a.rfind(|c: char| !c.is_whitespace()) {
            offset = b.len() + (a.len() - k - 1)
        } else {
            offset = a.len() + b.len();
        }
        self.len -= offset as u16;
    } //trim_right

    /// **in-place** trimming of white spaces at either end of the string
    pub fn trim_whitespaces(&mut self) {
        self.trim_left();
        self.trim_right();
    }

    // convenience
    #[inline(always)]
    fn endi(&self) -> usize {
        // index of last value plus 1
        //fastmod(self.front as usize + self.len as usize,N)
        (self.front as usize + self.len as usize) % N
    } // last

    #[inline(always)]
    fn index(&self, i: usize) -> usize {
        (self.front as usize + i) % N
    } // index of ith vale

    /// length of string in bytes
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// construct new, empty string (same as `cstr::default`)
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    } //new

    /// returns a pair of string slices `(left,right)` which, when concatenated,
    /// will yield an equivalent string underneath.  In case of no wraparound,
    /// the right str will be empty.
    pub fn to_strs(&self) -> (&str, &str) {
        let answer;
        if self.len() == 0 {
            answer = ("", "");
        } else if self.is_contiguous() {
            let front = self.front as usize;
            answer = (
                core::str::from_utf8(&self.chrs[front..front + (self.len as usize)]).unwrap(),
                "",
            )
        } else {
            answer = (
                core::str::from_utf8(&self.chrs[self.front as usize..]).unwrap(),
                core::str::from_utf8(&self.chrs[..self.endi()]).unwrap(),
            )
        }
        answer
    } //to_strs

    /// returns iterator over the characters of the string
    pub fn chars<'a>(&'a self) -> CircCharIter<'a> {
        let contig = self.is_contiguous();
        CircCharIter {
            first: if contig {
                &self.chrs[self.front as usize..(self.front + self.len) as usize]
            } else {
                &self.chrs[self.front as usize..]
            },
            second: if contig {
                &[]
            } else {
                &self.chrs[..self.endi()]
            },
            index: 0,
        }
    } //chars

    /// alias for [Self.chars]
    pub fn iter<'a>(&'a self) -> CircCharIter<'a> {
        self.chars()
    }

    /// returns a copy of the same string that is contiguous underneath.
    /// This may call [cstr::reset], which is an O(n) operation.
    pub fn to_contiguous(&self) -> cstr<N> {
        let mut c = *self;
        if !c.is_contiguous() {
            c.reset();
        }
        c
    }

    /// returns a single str slice if the cstr is contiguous underneath,
    /// otherwise panics.
    pub fn force_str(&self) -> &str {
        let (a, b) = self.to_strs();
        if b.len() > 0 {
            panic!("cstr cannot be transformed into a single str slice without calling reset()");
        }
        a
    }

    /// converts cstr to an owned string
    #[cfg(not(feature = "no-alloc"))]
    pub fn to_string(&self) -> alloc::string::String {
        let (a, b) = self.to_strs();
        let mut s = alloc::string::String::from(a);
        if b.len() > 0 {
            s.push_str(b);
        }
        s
    } //to_string

    /// returns a copy of the portion of the string.  Will return empty
    /// string if indices are invalid. The returned string will be contiguous.
    pub fn substr(&self, start: usize, end: usize) -> cstr<N> {
        let mut s = cstr::<N>::default();
        if (end <= start || start as u16 > self.len - 1 || end > self.len as usize) {
            return s;
        }
        for i in start..end {
            s.chrs[i - start] = self.chrs[self.index(i)];
        }
        s.len = (end - start) as u16;
        s
    } //substr

    /// in-place modification of ascii characters to lower-case.
    pub fn make_ascii_lowercase(&mut self) {
        for i in 0..self.len as usize {
            let b = &mut self.chrs[self.index(i)];
            if *b >= 65 && *b <= 90 {
                *b |= 32;
            }
        }
    } //make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case.
    pub fn make_ascii_uppercase(&mut self) {
        for i in 0..self.len as usize {
            let b = &mut self.chrs[self.index(i)];
            if *b >= 97 && *b <= 122 {
                *b -= 32;
            }
        }
    } //make_ascii_uppercase

    /// Tests for ascii case-insensitive equality with another string.
    /// This function does not check if the argument is ascii.
    pub fn case_insensitive_eq<TA>(&self, other: TA) -> bool
    where
        TA: AsRef<str>,
    {
        if self.len() != other.as_ref().len() {
            return false;
        }
        let obytes = other.as_ref().as_bytes();
        for i in 0..self.len() {
            let mut c = self.chrs[(self.front as usize + i) % N];
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

    /// Decodes a UTF-16 encodeded slice. If a decoding error is encountered
    /// or capacity exceeded, an `Err(s)` is returned where s is the
    /// the encoded string up to the point of the error.  The returned
    /// string will be contiguous.
    pub fn from_utf16(v: &[u16]) -> Result<Self, Self> {
        let mut s = Self::new();
        for c in char::decode_utf16(v.iter().cloned()) {
            if let Ok(c1) = c {
                if !s.push_char(c1) {
                    return Err(s);
                }
            } else {
                return Err(s);
            }
        }
        Ok(s)
    } //from_utf16

    /*
    /// returns an str slice representation by possibly calling
    /// [Self::reset] first, which is expensive.
    pub fn force_str(&mut self) -> &str {
      if !self.is_contiguous() {self.reset();}
      let(a,_) = self.to_strs();
      a
    }

    #[cfg(feature="serde")]
    /// for serde only, panics if underlying representation is not contiguous
    pub fn as_str(&self) -> &str {
      let(a,b) = self.to_strs();
      if b.len()>0 {panic!("serialization of cstr is only allowed after reset()");}
      a
    }

    #[inline]
    fn index_of(&self,i:usize) -> usize {
      fastmod(self.front as usize + i,N)
    }
    */

    #[inline(always)]
    fn index16(&self, i: u16) -> u16 {
        (self.front + i) % (N as u16)
        //let n = N as u16;
        //let mask = n-1;
        //if n&mask==0 { (self.front+i) & mask } else {(self.front+i) % n }
    }
} //main impl
  ///////////////////////////////////////////////////////////////

impl<const M: usize> cstr<M> {
    /// converts an `cstr<M>` to an `cstr<N>`. If the length of the string being
    /// converted is greater than N, the extra characters are ignored.
    /// This operation produces a new string that is contiguous underneath.
    pub fn resize<const N: usize>(&self) -> cstr<N> {
        let slen = self.len();
        let length = if (slen < N) { slen } else { N };
        let mut s = cstr::<N>::default();
        let (a, b) = self.to_strs();
        s.chrs[..a.len()].copy_from_slice(a.as_bytes());
        if b.len() > 0 {
            s.chrs[a.len()..].copy_from_slice(b.as_bytes());
        }
        s.len = self.len;
        s
    } //resize

    /// version of resize that does not allow string truncation due to length
    pub fn reallocate<const N: usize>(&self) -> Option<cstr<N>> {
        if self.len() < N {
            Some(self.resize())
        } else {
            None
        }
    }
} //impl cstr<M>

impl<const N: usize> Default for cstr<N> {
    fn default() -> Self {
        cstr {
            chrs: [0; N],
            front: 0,
            len: 0,
        }
    }
} //impl default

impl<const N: usize> core::fmt::Debug for cstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (a, b) = self.to_strs();
        f.pad(a)?;
        f.pad(b)
    }
} // Debug impl

impl<const N: usize> core::fmt::Display for cstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (a, b) = self.to_strs();
        write!(f, "{}{}", a, b)
    }
}

/////////// need Eq, Ord, etc.  and special iterator implementation
impl<const N: usize> PartialEq<&str> for cstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self == other
    } //eq
}

impl<const N: usize> PartialEq<&str> for &cstr<N> {
    fn eq(&self, other: &&str) -> bool {
        let (a, b) = self.to_strs();
        let (alen, blen) = (a.len(), b.len());
        alen + blen == other.len() && a == &other[..alen] && (blen == 0 || b == &other[alen..])
    } //eq
}

impl<const N: usize> PartialEq<cstr<N>> for &str {
    fn eq(&self, other: &cstr<N>) -> bool {
        let (a, b) = other.to_strs();
        let (alen, blen) = (a.len(), b.len());
        alen + blen == self.len() && a == &self[..alen] && (blen == 0 || b == &self[alen..])
    } //eq
}

impl<const N: usize> PartialEq<&cstr<N>> for &str {
    fn eq(&self, other: &&cstr<N>) -> bool {
        let (a, b) = other.to_strs();
        let (alen, blen) = (a.len(), b.len());
        alen + blen == self.len() && a == &self[..alen] && (blen == 0 || b == &self[alen..])
    } //eq
}

/// character interator, returned by [cstr::chars] (available with `circular-str` option along with [cstr])
pub struct CircCharIter<'a> {
    first: &'a [u8],
    second: &'a [u8],
    index: usize,
}
impl<'a> Iterator for CircCharIter<'a> {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.first.len() {
            self.index += 1;
            Some(self.first[self.index - 1] as char)
        } else if self.index - self.first.len() < self.second.len() {
            self.index += 1;
            Some(self.second[self.index - self.first.len() - 1] as char)
        } else {
            None
        }
    } //next
} // impl CircCharIter

/// The implementation of this trait allows comparison between
/// circular strings of different capacity.  This could affect the
/// type inference of the [cstr::resize] function.
impl<const N: usize, const M: usize> PartialEq<cstr<M>> for cstr<N> {
    fn eq(&self, other: &cstr<M>) -> bool {
        if self.len != other.len {
            return false;
        }
        for i in 0..self.len {
            if self.chrs[(self.front + i) as usize % N]
                != other.chrs[(other.front + i) as usize % M]
            {
                return false;
            }
        } //for
        true
        /*
          let mut schars = self.chars();
          let mut ochars = other.chars();
          loop {
              match (schars.next(), ochars.next()) {
                  (None, None) => {
                      break;
                  }
                  (Some(x), Some(y)) if x == y => {}
                  _ => {
                      return false;
                  }
              } //match
          } //loop
          true
        */
    } //eq for Self
} // PartialEq
impl<const N: usize> Eq for cstr<N> {}

impl<const N: usize> Ord for cstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut schars = self.chars();
        let mut ochars = other.chars();
        let mut answer = Ordering::Equal;
        loop {
            match (schars.next(), ochars.next()) {
                (Some(x), Some(y)) if x.cmp(&y) == Ordering::Equal => {}
                (Some(x), Some(y)) => {
                    answer = x.cmp(&y);
                    break;
                }
                (None, None) => {
                    break;
                }
                (None, _) => {
                    answer = Ordering::Less;
                    break;
                }
                (_, None) => {
                    answer = Ordering::Greater;
                    break;
                }
            } //match
        } //loop
        answer
    } //cmp
} //Ord

impl<const N: usize> PartialOrd for cstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
        /*
        let mut schars = self.chars();
        let mut ochars = other.chars();
        let mut answer = Ordering::Equal;
        loop {
          match (schars.next(), ochars.next()) {
            (Some(x), Some(y)) if x.cmp(&y)==Ordering::Equal => {},
            (Some(x), Some(y)) => { answer = x.cmp(&y); break; },
            (None,None) => {break;}
            (None,_) => { answer = Ordering::Less; break; },
            (_,None) => { answer = Ordering::Greater; break; },
          }//match
        }//loop
        Some(answer)
        */
    } //partial_cmp
} // PartialOrd

impl<const N: usize> PartialOrd<&str> for cstr<N> {
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
        let mut schars = self.chars();
        let mut ochars = other.chars();
        let mut answer = Ordering::Equal;
        loop {
            match (schars.next(), ochars.next()) {
                (Some(x), Some(y)) if x.cmp(&y) == Ordering::Equal => {}
                (Some(x), Some(y)) => {
                    answer = x.cmp(&y);
                    break;
                }
                (None, None) => {
                    break;
                }
                (None, _) => {
                    answer = Ordering::Less;
                    break;
                }
                (_, None) => {
                    answer = Ordering::Greater;
                    break;
                }
            } //match
        } //loop
        Some(answer)
    } //partial_cmp
} // PartialOrd

impl<const N: usize> PartialOrd<&str> for &cstr<N> {
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
        let mut schars = self.chars();
        let mut ochars = other.chars();
        let mut answer = Ordering::Equal;
        loop {
            match (schars.next(), ochars.next()) {
                (Some(x), Some(y)) if x.cmp(&y) == Ordering::Equal => {}
                (Some(x), Some(y)) => {
                    answer = x.cmp(&y);
                    break;
                }
                (None, None) => {
                    break;
                }
                (None, _) => {
                    answer = Ordering::Less;
                    break;
                }
                (_, None) => {
                    answer = Ordering::Greater;
                    break;
                }
            } //match
        } //loop
        Some(answer)
    } //partial_cmp
} // PartialOrd

/// Hashing is implemented character-by-character, starting with the
/// last char and ending with the first
impl<const N: usize> core::hash::Hash for cstr<N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        for i in (0..self.len as usize).rev() {
            self.nth_bytechar(i).hash(state);
        }
        //for c in self.chars() { c.hash(state); }
    }
} //hash

impl<T: AsRef<str> + ?Sized, const N: usize> core::convert::From<&T> for cstr<N> {
    fn from(s: &T) -> cstr<N> {
        cstr::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> core::convert::From<&mut T> for cstr<N> {
    fn from(s: &mut T) -> cstr<N> {
        cstr::make(s.as_mut())
    }
}

impl<const N: usize> core::fmt::Write for cstr<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if s.len() + self.len() > N {
            return Err(core::fmt::Error::default());
        }
        self.push_str(s);
        Ok(())
    } //write_str
} //core::fmt::Write trait

impl<const N: usize, TA: AsRef<str>> Add<TA> for cstr<N> {
    type Output = cstr<N>;
    fn add(self, other: TA) -> cstr<N> {
        let mut a2 = self;
        a2.push_str(other.as_ref());
        a2
    }
} //Add AsRef<str>

impl<const N: usize> Add<&cstr<N>> for &str {
    type Output = cstr<N>;
    fn add(self, other: &cstr<N>) -> cstr<N> {
        let mut a2 = *other;
        a2.push_front(self);
        a2
    }
} //Add &str on left

impl<const N: usize> Add<cstr<N>> for &str {
    type Output = cstr<N>;
    fn add(self, mut other: cstr<N>) -> cstr<N> {
        other.push_front(self);
        other
    }
} //Add &str on left

impl<const N: usize> Add for &cstr<N> {
    type Output = cstr<N>;
    fn add(self, other: &cstr<N>) -> cstr<N> {
        let mut a2 = *self;
        let (l, r) = other.to_strs();
        a2.push_str(l);
        if r.len() > 0 {
            a2.push_str(r);
        }
        a2
    }
} //Add &str

impl<const N: usize> Add for cstr<N> {
    type Output = cstr<N>;
    fn add(self, other: cstr<N>) -> cstr<N> {
        let mut a2 = self;
        let (l, r) = other.to_strs();
        a2.push_str(l);
        if r.len() > 0 {
            a2.push_str(r);
        }
        a2
    }
} //Add

impl<const N: usize> core::str::FromStr for cstr<N> {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() <= N {
            Ok(cstr::from(s))
        } else {
            Err("capacity exceeded")
        }
    }
}

/*
////////// fast x % n for n that are powers of 2
#[inline(always)]
fn fastmod(x:usize, n:usize) -> usize {
    x % n
//  let mask = n-1;
//  if n&mask==0 { x & mask } else {x % n}
}//fastmod
*/

////// aliases
/// Convenient aliases for [cstr] using exact powers of 2
pub type cstr1k = cstr<1024>;
pub type cstr8 = cstr<8>;
pub type cstr16 = cstr<16>;
pub type cstr32 = cstr<32>;
pub type cstr64 = cstr<64>;
pub type cstr128 = cstr<128>;
pub type cstr256 = cstr<256>;
pub type cstr512 = cstr<512>;
