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
extern crate alloc;
use alloc::string::String;

//#[cfg(feature="serde")]
//use serde::{Deserialize, Serialize};

/// A *circular string* is represented underneath by fixed-size u8
/// array arranged as a circular
/// queue, allowing for efficient operations such as push, trim *in front*
/// of the string.
/// This type currently **only supports single-byte chars** including ascii strings.
/// Each `cstr<N>` can hold up to N bytes and the maximum N is 65536.
/// The Serialization (serde) and no-std options are both supported.
#[derive(Copy,Clone)]
pub struct cstr<const N : usize=32>
{
  chrs: [u8;N],
  front: u16,
  len: u16,
} //cstr 

impl<const N:usize> cstr<N>
{
   /// create `cstr` from `&str` with silent truncation; panics if
   /// N is greater than 65536
   pub fn make(src:&str) -> cstr<N> {
     if N > 65536 { panic!("cstr strings are limited to a maximum capacity of 65536");}
     let mut m = cstr::<N>::new();
     let length = core::cmp::min(N,src.len());
     m.chrs[..length].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     m
   }//make

   /// version of make that also panics if the input string is not ascii.
   pub fn from_ascii(src:&str) -> cstr<N> {
     if N > 65536 { panic!("cstr strings are limited to a maximum capacity of 65536");}
     if !src.is_ascii() { panic!("cstr string is not ascii");}
     let mut m = cstr::<N>::new();
     let length = core::cmp::min(N,src.len());
     m.chrs[..length].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     m
   }//from_ascii

   /// version of make that does not truncate: returns original str slice
   /// as error.  Also checks if N is no greater than 65536 without panic.
   pub fn try_make(src:&str) -> Result<cstr<N>, &str> {
     let length = src.len();
     if length>N || N>65536 {return Err(src);}
     let mut m = cstr::new();
     m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     Ok(m)
   }//try_make

   /// version of `try_make` that also checks if the input string is ascii.
   pub fn try_make_ascii(src:&str) -> Option<cstr<N>> {
     let length = src.len();
     if length>N || N>65536 || !src.is_ascii() {return None;}
     let mut m = cstr::new();
     m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     Some(m)
   }//try_make

   /// version of make that returns a pair consisting of the made
   /// `cstr` and the remainder `&str` that was truncated; panics if
   /// N is greater than 65536 (but does not check for ascii strings)
   pub fn make_remainder(src:&str) -> (cstr<N>,&str) {
     if N > 65536 { panic!("cstr strings are limited to a maximum capacity of 65536");}   
     let mut m = cstr::new();
     let length = core::cmp::min(N,src.len());
     m.chrs[..].copy_from_slice(&src.as_bytes()[..length]);
     m.len = length as u16;
     (m,&src[length..])
   }//try_make

   // make from a pair of str slices, does not truncate, and checks that
   // N is not greater than 65536 without panic
   pub fn from_pair(left:&str, right:&str) -> Option<cstr<N>> {
     let (llen,rlen) = (left.len(), right.len());
     if llen+rlen > N || N > 65536 { return None; }
     let mut m = cstr::new();
     m.len = (llen+rlen) as u16;
     m.chrs[..llen].copy_from_slice(&left.as_bytes()[..llen]);
     m.chrs[llen..].copy_from_slice(&right.as_bytes()[llen..]);
     Some(m)
   }//from_pair

   /// checks if the underlying representation of the string is contiguous
   /// (without wraparound).
   #[inline(always)]
   pub fn is_contiguous(&self) -> bool {
     (self.front as usize + self.len as usize) <= N
   }

   /// resets the internal representation of the cstr so that it is
   /// represented contiguously, without wraparound. **Calling this function
   /// has non-constant time cost both in terms of speed and memory** as
   /// it requires a secondary buffer as well as copying.**
   pub fn reset(&mut self) {
     if self.front==0 {return;}
     let mut mhrs = [0;N];
     for i in 0..self.len as usize {
       mhrs[i] = self.chrs[self.index(i)];
     }
     self.chrs = mhrs;
     self.front = 0;
   }//reset

   /// pushes given string to the end of the string, returns remainder
   pub fn push_str<'t>(&mut self, src:&'t str) -> &'t str {
     let srclen = src.len();
     let slen = self.len as usize;
     let bytes = &src.as_bytes();
     let length = core::cmp::min(slen+srclen , N);
     let remain = if N>(slen+srclen) {0} else {(srclen+slen)-N};
     let mut i = 0;
     while i<srclen && i+slen<N {
       self.chrs[self.index(slen+i)] = bytes[i];
       i += 1;
     }//while
     self.len += i as u16;
     &src[srclen-remain..]
   }//push_str

   /// Pushes string to the **front** of the string, returns remainder.
   /// because of the circular-queue backing, this operation as the same
   /// cost as pushing to the back of the string ([Self::push_str]).
   /// This function does not check if the input string is ascii.
   pub fn push_front<'t>(&mut self, src:&'t str) -> &'t str {
     let srclen = src.len();
     let slen = self.len as usize;
     let bytes = &src.as_bytes();
     let length = core::cmp::min(slen+srclen , N);
     let remain = if N>=(slen+srclen) {0} else {(srclen+slen)-N};
     let mut i = 0;
     while i<srclen && i+slen<N {
       self.front = (self.front + (N as u16) -1) % (N as u16);
       self.chrs[self.front as usize] = bytes[srclen-1-i];
       i += 1;
     }//while
     self.len += i as u16;     
     &src[..remain]
   }//push_front

   /// alias for [Self::push_front]
   pub fn push_str_front<'t>(&mut self, src:&'t str) -> &'t str {
      self.push_front(src)
   }

    /// Pushes a single character to the end of the string, returning
    /// true on success.  This function checks if the given character
    /// occupies a single-byte.
    pub fn push_char(&mut self, c:char) -> bool {
       let clen = c.len_utf8();
       if clen>1 || self.len as usize + clen > N {return false;}
       let mut buf = [0u8;4]; // char buffer
       let bstr = c.encode_utf8(&mut buf);
       self.push_str(bstr);
       true
    }// push_char

    /// Pushes a single character to the front of the string, returning
    /// true on success.  This function checks if the given character
    /// occupies a single-byte.
    pub fn push_char_front(&mut self, c:char) -> bool {
       let clen = c.len_utf8();
       if clen>1 || self.len as usize + clen > N {return false;}
       let newfront = ((self.front as usize) + N - 1) % N;
       self.chrs[newfront] = c as u8;
       self.front = newfront as u16;
       self.len += 1;
       true
    }//push_char_front

    /// remove and return last character in string, if it exists
    pub fn pop_char(&mut self) -> Option<char> {
       if self.len()==0 {return None;}
       let lasti = ((self.front+self.len-1) as usize) % N;
       let firstchar = self.chrs[lasti] as char;
       self.len-=1;
       Some(firstchar)
       /*
       let (l,r) = self.to_strs();
       let right = if r.len()>0 {r} else {l};
       let (ci,lastchar) = right.char_indices().last().unwrap();
       self.len = if r.len()>0 {(l.len() + ci) as u16} else {ci as u16};
       Some(lastchar)
       */
    }//pop

    /// remove and return first character in string, if it exists
    pub fn pop_char_front(&mut self) -> Option<char> {
       if self.len()==0 {return None;}
       let firstchar = self.chrs[self.front as usize] as char;
       self.front = (self.front+1)%(N as u16);
       self.len -= 1;
       Some(firstchar)
       /*
       let (left,r) = self.to_strs();
       let firstchar = left.chars().next().unwrap();
       let clen = firstchar.len_utf8() as u16;
       self.front = (self.front+clen) % (N as u16) ;
       self.len -= clen;
       Some(firstchar)
       */
    }//pop_char_front


    /// right-truncates string up to byte position n.  No effect
    /// if n is greater than or equal to the length of the string.
    pub fn truncate_bytes(&mut self, n: usize) {
       if (n<self.len as usize) {
         /*
         let (a,b) = self.to_strs();
         if n<a.len() {
           assert!(a.is_char_boundary(n));
         }
         else {
           assert!(b.is_char_boundary(n-a.len()));         
         }
         */
	 self.len = n as u16;
       }
    }

    /// left-truncates string up to byte position n.  No effect
    ///if n is greater than the length of the string.
    pub fn truncate_left(&mut self, n: usize) {
       if (n>0 && n<=self.len as usize) {
         /*
         let (a,b) = self.to_strs();
         if n<a.len() {
           assert!(a.is_char_boundary(n));
         }
         else {
           assert!(b.is_char_boundary(n-a.len()));         
         }
         */
         self.front = ((self.front as usize + n)%N) as u16;
	 self.len -= n as u16;
       }
    }//truncate_left

    /// finds the position of first character that satisfies given predicate
    pub fn find<P>(&self, predicate: P) -> Option<usize>
         where P : Fn(char) -> bool
    {
        let (a,b) = self.to_strs();
        if let Some(pos) = a.find(|x:char|predicate(x)) {
            Some(pos)
        }
        else if let Some(pos) = b.find(|x:char|predicate(x)) {
            Some(a.len() + pos)
        }
        else { None }
    }//find

    /// finds position of first matching substring
    pub fn find_substr(&self, s:&str) -> Option<usize> {
        let (a,b) = self.to_strs();
        if let Some(pos) = a.find(s) {
            Some(pos)
        }
        else if let Some(pos) = b.find(s) {
            Some(a.len() + pos)
        }
        else { None }      
    }//find_substr
    
    /// finds the position of last character that satisfies given predicate
    pub fn rfind<P>(&self, predicate: P) -> Option<usize>
         where P : Fn(char) -> bool
    {
        let (a,b) = self.to_strs();
        if let Some(pos) = b.find(|x:char|predicate(x)) {
            Some(a.len()+pos)
        }
        else if let Some(pos) = a.find(|x:char|predicate(x)) {
            Some(pos)
        }
        else { None }
    }//find

    /// finds position of last matching substring
    pub fn rfind_substr(&self, s:&str) -> Option<usize> {
        let (a,b) = self.to_strs();
        if let Some(pos) = b.find(s) {
            Some(a.len()+pos)
        }
        else if let Some(pos) = a.find(s) {
            Some(pos)
        }
        else { None }      
    }//find_substr

    // **in-place** trimming of white spaces at the front of the string
    pub fn trim_left(&mut self) {
      let (a,b) = self.to_strs();
      let offset;
      if let Some(i) = a.find(|c:char|!c.is_whitespace()) {
         offset = i;
      }
      else if let Some(k) = b.find(|c:char|!c.is_whitespace()) {
         offset = a.len() + k;
      }
      else {
         offset = a.len() + b.len();
      }
      self.front = ((self.front as usize + offset)%N) as u16;
      self.len -= offset as u16;
    }//trim_left

    // **in-place** trimming of white spaces at the end of the string
    pub fn trim_right(&mut self) {
      let (a,b) = self.to_strs();
      let offset;
      if b.len()==0 {
        if let Some(k) = a.rfind(|c:char|!c.is_whitespace()) {
         offset = a.len() - k - 1;
        }
        else {
          offset = a.len();
        }
      }//contiguous
      else if let Some(i) = b.rfind(|c:char|!c.is_whitespace()) {
         offset = b.len() - i - 1;
      }
      else if let Some(k) = a.rfind(|c:char|!c.is_whitespace()) {
         offset = b.len() + (a.len() - k - 1)
      }
      else {
         offset = a.len() + b.len();
      }
      self.len -= offset as u16;
    }//trim_right

    /// **in-place** trimming of white spaces at either end of the string
    pub fn trim_whitespaces(&mut self) {
      self.trim_left();
      self.trim_right();
    }


   // convenience
   #[inline(always)]
   fn endi(&self) -> usize {  // index of last value plus 1
     (self.front as usize + self.len as usize )%N
   }// last

   #[inline(always)]
   fn index(&self, i:usize) -> usize {
     (self.front as usize +i)%N
   } // index of ith vale

///////// doesn't work on non-ascii strings, because of char boundaries

   /// length of string in bytes
   #[inline(always)]
   pub fn len(&self) -> usize  { self.len as usize }

   /// construct new, empty string (same as `cstr::default`)
   #[inline(always)]   
   pub fn new() -> Self {
      Self::default()
   }//new

   /// returns a pair of string slices `(left,right)` which, when concatenated,
   /// will yield an equivalent string underneath.  In case of no wraparound,
   /// the right str will be empty.
   pub fn to_strs(&self) -> (&str,&str) {
     let answer;
     if self.len()==0 {answer = ("","");}
     else if self.is_contiguous() {
       answer = (core::str::from_utf8(&self.chrs[self.front as usize .. self.endi()]).unwrap(),
        "")
     }
     else {
       answer=(core::str::from_utf8(&self.chrs[self.front as usize .. ]).unwrap(),
        core::str::from_utf8(&self.chrs[.. self.endi()]).unwrap())
     }
     answer
   }//to_strs


   /// returns iterator over the characters of the string
   pub fn chars<'a>(&'a self) -> CircCharIter<'a> {
     let contig = self.is_contiguous();
     CircCharIter {
       first : if contig { &self.chrs[self.front as usize .. self.endi()] }
               else { &self.chrs[self.front as usize ..] },
       second: if contig { &[] }
               else { &self.chrs[..self.endi()] },
       index : 0,           
     }
   }//chars

   /// alias for [Self.chars]
   pub fn iter<'a>(&'a self) -> CircCharIter<'a> {
     self.chars()
   }

   /// returns a copy of the same string that is contiguous underneath
   pub fn to_contiguous(&self) -> cstr<N> {
     let mut c = *self;
     if !c.is_contiguous() {c.reset();}
     c
   }

   /// returns a single str slice if the cstr is contiguous underneath,
   /// otherwise panics.
   pub fn force_str(&self) -> &str {
     let (a,b) = self.to_strs();
     if b.len()>0 {panic!("cstr cannot be transformed into a single str slice without calling reset()");}
     a
   }

   /// converts cstr to an owned string
   pub fn to_string(&self) -> String {
     let (a,b) = self.to_strs();
     let mut s = String::from(a);
     if b.len()>0 {s.push_str(b);}
     s
   }//to_string


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
   */
}//main impl
///////////////////////////////////////////////////////////////

impl<const N :usize> Default for cstr<N> {
  fn default() -> Self {
    cstr {
       chrs: [0;N],
       front: 0,
       len:0,
    }
  }
}//impl default

impl<const N: usize> core::fmt::Debug for cstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let (a,b) = self.to_strs();
        f.pad(a)?;
        f.pad(b)
    }
} // Debug impl

/////////// need Eq, Ord, etc.  and special iterator implementation
impl<const N: usize> PartialEq<&str> for cstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self == other
    }//eq
}

impl<const N: usize> PartialEq<&str> for &cstr<N> {
    fn eq(&self, other: &&str) -> bool {
        let (a,b) = self.to_strs();
        let (alen, blen) = (a.len(), b.len());
        alen+blen==other.len() &&
          a == &other[..alen]  &&  (blen==0 || b == &other[alen..]) 
    } //eq
}

/*
impl<T:AsRef<str>, const N: usize> PartialEq<&T> for &cstr<N> {
    fn eq(&self, other: &&T) -> bool {
        let (a,b) = self.to_strs();
        let (alen, blen) = (a.len(), b.len());
        let oref = other.as_ref();
        alen+blen==oref.len() &&
          a == &oref[..alen]  &&  (blen==0 || b == &oref[alen..]) 
    } //eq
}
*/

impl<const N: usize> PartialEq<cstr<N>> for &str {
    fn eq(&self, other: &cstr<N>) -> bool {
        let (a,b) = other.to_strs();
        let (alen, blen) = (a.len(), b.len());
        alen+blen==self.len() &&
          a == &self[..alen]  &&  (blen==0 || b == &self[alen..]) 
    } //eq
}

impl<const N: usize> PartialEq<&cstr<N>> for &str {
    fn eq(&self, other: &&cstr<N>) -> bool {
        let (a,b) = other.to_strs();
        let (alen, blen) = (a.len(), b.len());
        alen+blen==self.len() &&
          a == &self[..alen]  &&  (blen==0 || b == &self[alen..]) 
    } //eq
}

/// character interator, returned by [cstr::chars]
pub struct CircCharIter<'a> {
  first : &'a [u8],
  second: &'a [u8],
  index : usize,
}
impl<'a> Iterator for CircCharIter<'a> {
  type Item = char;
  fn next(&mut self) -> Option<Self::Item> {
    if self.index<self.first.len() {
      self.index += 1;
      Some(self.first[self.index-1] as char)
    }
    else if self.index-self.first.len() < self.second.len() {
      self.index += 1;
      Some(self.second[self.index-self.first.len()-1] as char)
    }
    else { None }
  }//next
}// impl CircCharIter

impl<const N: usize> PartialEq for cstr<N> {
    fn eq(&self, other: &Self) -> bool {
       let mut schars = self.chars();
       let mut ochars = other.chars();
       loop {
         match (schars.next(), ochars.next()) {
           (None,None) => {break;},
           (Some(x), Some(y)) if x==y => {},
           _ => { return false; },
         }//match
       }//loop
       true
    }//eq for Self
}// PartialEq
impl<const N:usize> Eq for cstr<N> {}

impl<const N: usize> Ord for cstr<N> {
  fn cmp(&self, other:&Self) -> Ordering {
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
       answer  
  }//cmp
}//Ord

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
    }//partial_cmp
}// PartialOrd


impl<const N: usize> PartialOrd<&str> for cstr<N> {
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
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
    }//partial_cmp
}// PartialOrd

impl<const N: usize> PartialOrd<&str> for &cstr<N> {
    fn partial_cmp(&self, other: &&str) -> Option<Ordering> {
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
    }//partial_cmp
}// PartialOrd


impl<const N:usize> core::hash::Hash for cstr<N> {
  fn hash<H:core::hash::Hasher>(&self, state:&mut H) {
    for c in self.chars() { c.hash(state); }
  }
}//hash

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
