//! This module implements **[Flexstr]**, which uses an internal enum
//! to hold either a fixed string of up to a maximum length, or an owned [String].

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use crate::fstr;
use crate::zstr;
use crate::tstr;
use crate::{str12, str128, str16, str192, str24, str256, str32, str4, str48, str64, str8, str96};
use core::cmp::{min, Ordering};
use core::ops::Add;
//use crate::flexible_string::Strunion::*;
use crate::shared_structs::Strunion;
use crate::shared_structs::Strunion::*;
/*
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
*/
/// A `Flexstr<N>` is represented internally as a `tstr<N>` if the length of
/// the string is less than N bytes, and by an owned String otherwise.
/// The structure satisfies the following axiom:
/// >   *For N <= 256, a `Flexstr<N>` is represented internally by an
///     owned String if and only if the length of the string is greater than
///     or equal to N*.
///
/// For example, a `Flexstr<16>` will hold a string of up to 15 bytes 
/// in an u8-array of size 16. The first byte of the array holds the length of
/// the string.  If subsequent operations such as [Flexstr::push_str]
/// extends the string past 15 bytes, the representation will switch to an owned
/// String.  Conversely, an operation such as [Flexstr::truncate]
/// may switch the representation back to a fixed string.
/// The default N is 32.  **The largest N for which the axiom holds
/// is 256.**  For all N>256, the internal representation is always an owned
/// string.
///
/// Example:
/// ```ignore
///  let mut s:Flexstr<8> = Flexstr::from("abcdef");
///  assert!(s.is_fixed());
///  s.push_str("ghijk");
///  assert!(s.is_owned());
///  s.truncate(7);
///  assert!(s.is_fixed());
/// ```
///
/// The intended use of this datatype is for
/// situations when the lengths of strings are *usually* less than N, with
/// only occasional exceptions that require a different representation.
/// However, unlike the other string types in this crate, a Flexstr cannot be
/// copied and is thus subject to **move semantics**.  The serde serialization
/// option is also supported (`features serde`).
///
/// In addition, this type impls the `Add` trait for string concatenation:
///
/// ```ignore
///  let a = flexstr8::from("abcd");
///  let b = &a + "efg";
///  assert_eq!(&b,"abcdefg");
/// ```
///

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Flexstr<const N:usize=32>
{
   inner:Strunion<N>,
}
impl<const N:usize> Flexstr<N>
{
  /// Creates a new `Flexstr<N>` with given &str.
  pub fn make(s:&str) -> Self
  {
     if s.len()<N && N<=256 {Flexstr{inner:fixed(tstr::<N>::from(s))}}
     else {Flexstr{inner:owned(String::from(s))}}
  }//make

  /// Creates a `Flexstr<N>` by consuming a given string.  However, if the
  /// string has length less than N, then a fixed representation will be used.
  pub fn from_string(s:String) -> Self {
    if s.len()<N && N<=256 {Flexstr{inner:fixed(tstr::<N>::from(&s[..]))}}
     else {Flexstr{inner:owned(s)}}
  }

  /// creates a `Flexstr<N>` from a given `tstr<N>`
  pub fn from_tstr(s:tstr<N>) -> Self {
    Flexstr{inner:fixed(s)}
  }


  #[cfg(feature="serde")]
  /// this function is only added for uniformity in serde implementation
  pub fn try_make(s: &str) -> Result<Flexstr<N>, &str> {
       Ok(Flexstr::make(s))
    }

  /// length of the string in bytes. This is a constant-time operation.
  pub fn len(&self) -> usize
  {
    match &self.inner {
      fixed(s) => s.len(),
      owned(s) => s.len(),
    }//match
  }//len

  /// creates an empty string, equivalent to [Flexstr::default]
  pub fn new() -> Self { Self::default() }

  /// length in number of characters as opposed to bytes: this is
  /// not necessarily a constant time operation.
  pub fn charlen(&self) -> usize {
     match &self.inner {
       fixed(s) => s.charlen(),
       owned(s) => {
         s.chars().count()
         //let v: Vec<_> = s.chars().collect();
         //v.len()
       },
     }//match
  }//charlen

  /// converts fstr to &str, possibly using using [core::str::from_utf8_unchecked].  Since
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
    ///[core::str::from_utf8] and may technically panic.
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
  /// ```ignore
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
       core::mem::swap(&mut self.inner, &mut temp);
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
  pub fn nth_bytechar(&self, n:usize) -> char {
    match &self.inner {
       fixed(s) => s.nth_ascii(n),
       owned(s) => s.as_bytes()[n] as char,
    }
  }//nth_bytechar

  /// alias for [Self::nth_bytechar] (for backwards compatibility)
  pub fn nth_ascii(&self, n:usize) -> char { self.nth_bytechar(n) }


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
  pub fn if_fixed<F>(&mut self, f:F) where F:FnOnce(&mut tstr<N>)
  {
     if let fixed(s) = &mut self.inner {f(s);}
  }

  /// applies the destructive closure only if the internal representation
  /// is a fixed string
  pub fn if_owned<F>(&mut self, f:F) where F:FnOnce(&mut str)
  {
     if let owned(s) = &mut self.inner {f(s);}
  }

  /// applies closure f if the internal representation is a fixed string,
  /// or closure g if the internal representation is an owned string.
  pub fn map_or<F,G,U>(&self, f:F, g:G) -> U
    where F:FnOnce(&tstr<N>)-> U, G:FnOnce(&str) -> U
  {
     match &self.inner {
       fixed(s) => f(s),
       owned(s) => g(&s[..]),
     }//match
  }//map

  /// version of [Flexstr::map_or] accepting FnMut closures
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
  }//push_str

  /// appends string with a single character, switching to the String
  /// representation if necessary.  Returns true if resulting string
  /// remains fixed.
  pub fn push(&mut self, c:char) -> bool {
     let clen = c.len_utf8();
     match &mut self.inner {
       owned(s) => { s.push(c); false},
       fixed(s) if s.len()+clen>=N => {
         let mut fss = s.to_string();  fss.push(c);
	 self.inner = owned(fss);
	 false
       },
       fixed(s) => {
         let mut buf = [0u8;4];
	 let bstr = c.encode_utf8(&mut buf);
	 s.push(bstr);
	 true
       }
     }//match
  }//push


  /// remove and return last character in string, if it exists
  pub fn pop(&mut self) -> Option<char> {
    if self.len()==0 {return None;}
    match &mut self.inner {
      fixed(s) => s.pop_char(),
      owned(s) if s.len()>N => s.pop(),
      owned(s) => {  // change representation
        let lastchar = s.pop();
        self.inner = fixed(tstr::from(&s));
        lastchar
      }
    }//match
  }//pop

  /// alias for [Self::pop]
  pub fn pop_char(&mut self) -> Option<char> { self.pop() }
  
  /// this function truncates a string at the indicated byte position,
  /// returning true if the truncated string is fixed, and false if owned.
  /// The operation has no effect if n is larger than the length of the
  /// string.  The operation will **panic** if n is not on a character
  /// boundary, similar to [String::truncate].
  pub fn truncate(&mut self, n: usize) -> bool {
    match &mut self.inner {
      fixed(fs) if n<fs.len() => { fs.truncate_bytes(n); true },
      fixed(_) => {true},
      owned(s) if n<N => {
        assert!(s.is_char_boundary(n));
        self.inner = fixed(tstr::<N>::from(&s[..n]));
        true
      },
      owned(s) => { if n<s.len() {s.truncate(n);} false},
    }//match
  }//truncate

    /// resets string to empty
    pub fn clear(&mut self) {
      match &mut self.inner {
         fixed(s) => {s.clear();},
         owned(s) => { self.inner = fixed(tstr::default());},
      }
    }//clear

  /// returns string corresponding to slice indices as a copy or clone.
  pub fn substr(&self, start: usize, end: usize) -> Flexstr<N> {
    match &self.inner {
      fixed(s) => Flexstr{inner:fixed(s.substr(start,end))},
      owned(s) => Self::from(&s[start..end]),
    }
  }//substr


  /// Splits the string into a `tstr<N>` portion and a String portion.
  /// The structure inherits the fixed part and the String returned will
  /// contain the extra bytes that does not fit.  Example:
  ///
  /// ```
  ///  # use fixedstr::*;
  ///   let mut fs:Flexstr<4> = Flexstr::from("abcdefg");
  ///   let extras = fs.split_off();
  ///   assert!( &fs=="abc" && &extras=="defg" && fs.is_fixed());
  /// ```
  pub fn split_off(&mut self) -> String {
    match &mut self.inner {
      fixed(s) => { String::default() },
      owned(s) => {
	 let answer = String::from(&s[N-1..]);
         self.inner = fixed( tstr::<N>::from(&s[..N-1]) );
	 answer
      }
    }//match
  }//split_off


    /// in-place modification of ascii characters to lower-case
    pub fn make_ascii_lowercase(&mut self) {
      match &mut self.inner {
        fixed(s) => { s.make_ascii_lowercase(); },
        owned(s) => {
           s.as_mut_str().make_ascii_lowercase();
        },
      }//match
    }//make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case
    pub fn make_ascii_uppercase(&mut self) {
      match &mut self.inner {
        fixed(s) => { s.make_ascii_uppercase(); },
        owned(s) => {
           s.as_mut_str().make_ascii_uppercase();
        },
      }//match
    }

} //impl<N>


impl<const N:usize> Default for Flexstr<N> {
  fn default() -> Self { Flexstr {inner:fixed(tstr::<N>::default())} }
}

/*
impl<const N:usize> core::hash::Hash for Flexstr<N>
{
  fn hash<H:core::hash::Hasher>(&self, state:&mut H) {
    self.as_str().hash(state)
  }
}//hash
*/

impl<const N:usize> core::ops::Deref for Flexstr<N>
{
    type Target = str;
    fn deref(&self) -> &Self::Target {
      self.to_str()
    }
}

impl<T: AsRef<str> + ?Sized, const N: usize> core::convert::From<&T> for Flexstr<N> {
    fn from(s: &T) -> Self {
        Self::make(s.as_ref())
    }
}

impl<T: AsMut<str> + ?Sized, const N: usize> core::convert::From<&mut T> for Flexstr<N> {
    fn from(s: &mut T) -> Self {
        Self::make(s.as_mut())
    }
}

impl<const N: usize> core::cmp::PartialOrd for Flexstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        //Some(self.chrs[0..self.len].cmp(other.chrs[0..other.len]))
        Some(self.cmp(other))
    }
}

impl<const N: usize> core::cmp::Ord for Flexstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
      self.to_str().cmp(other.to_str())
    }
}

impl<const N: usize> core::convert::AsRef<str> for Flexstr<N> {
    fn as_ref(&self) -> &str {
        self.to_str()
    }
}
impl<const N: usize> core::convert::AsMut<str> for Flexstr<N> {
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

impl<const N: usize> core::fmt::Debug for Flexstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(&self.to_str())
    }
} // Debug impl

impl<const N: usize> core::fmt::Display for Flexstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<const N: usize> core::fmt::Write for Flexstr<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        self.push_str(s);
        Ok(())
    } //write_str
} //core::fmt::Write trait


impl<const N: usize> core::convert::From<String> for Flexstr<N> {
    /// *will consume owned string and convert it to a fixed
    /// representation if its length is less than N*
    fn from(s: String) -> Self {
        if s.len()>=N {
	  Flexstr{inner:owned(s)}
	} else {
	  Flexstr{inner:fixed(tstr::<N>::from(&s[..]))}
	}
    }
}//from String

impl<const M: usize> Flexstr<M> {
  /// returns a copy/clone of the string with new fixed capacity N.
  /// Example:
  /// ```
  ///  # use fixedstr::Flexstr;
  ///  let mut a:Flexstr<4> = Flexstr::from("ab");
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

/* redundant
impl<const N:usize> Add for &Flexstr<N> {
  type Output = Flexstr<N>;
  fn add(self, other:Self) -> Self::Output {
    match (&self.inner, &other.inner) {
       (owned(a),b) => {
         let mut a2 = a.clone();
         a2.push_str(&other);
         Flexstr{inner:owned(a2)}
       },
       (a,owned(b)) => {
         let mut a2 = self.clone().to_string();
         a2.push_str(&other);
         Flexstr{inner:owned(a2)}         
       },
       (fixed(a), fixed(b)) if a.len() + b.len() >= N => {
         let mut a2 = a.to_string();
         a2.push_str(&b);
         Flexstr{inner:owned(a2)}       
       },
       (fixed(a), fixed(b)) => {
         let mut a2 = *a; //copy
         a2.push(&b);
         Flexstr{inner:fixed(a2)}
       }
    }//match
  }
}//Add
*/

impl<const N:usize> Add<&str> for &Flexstr<N> {
  type Output = Flexstr<N>;
  fn add(self, other:&str) -> Self::Output {
    match (&self.inner, other) {
       (owned(a),b) => {
         let mut a2 = a.clone();
         a2.push_str(other);
         Flexstr{inner:owned(a2)}
       },
       (fixed(a), b) if a.len() + b.len() >= N => {
         let mut a2 = a.to_string();
         a2.push_str(b);
         Flexstr{inner:owned(a2)}       
       },
       (fixed(a), b) => {
         let mut a2 = *a; //copy
         a2.push(b);
         Flexstr{inner:fixed(a2)}
       }
    }//match
  }
}//Add, Rhs = &str

impl<const N:usize> Add<&Flexstr<N>> for &str {
  type Output = Flexstr<N>;
  fn add(self, other:&Flexstr<N>) -> Flexstr<N> {
    let mut a2 = Flexstr::from(self);
    a2.push_str(other);
    a2
  }
}//Add &str on left

impl<const N:usize> Add<Flexstr<N>> for &str {
  type Output = Flexstr<N>;
  fn add(self, other:Flexstr<N>) -> Flexstr<N> {
    let mut a2 = Flexstr::from(self);
    a2.push_str(&other);
    a2
  }
}//Add &str on left


/// convenient type aliases for [Flexstr]
pub type flexstr8 = Flexstr<8>;
pub type flexstr16 = Flexstr<16>;
pub type flexstr32 = Flexstr<32>;
pub type flexstr64 = Flexstr<64>;
pub type flexstr128 = Flexstr<128>;
pub type flexstr256 = Flexstr<256>;
