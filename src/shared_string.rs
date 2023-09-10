#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
//extern crate std;
//use std::string::String;
use crate::fstr;
use crate::zstr;
use crate::tstr;
use crate::{str12, str128, str16, str192, str24, str256, str32, str4, str48, str64, str8, str96};
use core::cmp::{min, Ordering};
use core::ops::Add;
use crate::shared_structs::Strunion;
use crate::shared_structs::Strunion::*;

////////////////////////////////////////////////////////////////////////
///////////// RC experiments
extern crate alloc;
use alloc::rc::Rc;
use alloc::string::String;
use core::cell::RefCell;

/// This type uses [Rc] and [RefCell] underneath to allow pointers to a [crate::Flexstr]
/// to be shared, and thus cloning is always done in constant time.
/// Similar to `Flexstr`, a `Sharedstr<N>` is represented either by a
/// `tstr<N>` or by an owned string if its length is greater than N-1, for N up
/// to 256.
///
/// **WARNING:**  this type uses certain unsafe features and allows mutations of
/// a shared string from multiple pointers, while allowing continued use of the
/// pointers after mutation. This violates a basic principle of the Rust borrow
/// checker.  Use with caution.
/// Example:
/// ```
///  # use fixedstr::*;
///  let mut a = Sharedstr::<8>::from("abc12");
///  let mut b = a.clone();
///  b.push('3');
///  assert!( a == "abc123" );
///  assert!( a==b && a.ptr_eq(&b) );
/// ```
/// Note that `==` always compares the contents while `ptr_eq` compares for
/// pointer-equality.
/// This type **does not support serde**, as expected of shared
/// pointers.  Convert to another type of string for serialization.
#[derive(Eq, PartialEq, Clone)]
pub struct Sharedstr<const N:usize=32>
{
   inner: Rc<RefCell<Strunion<N>>>,
}
impl<const N:usize> Sharedstr<N>
{
  /// Creates a new `Sharedstr<N>` with given &str.
  pub fn make(s:&str) -> Self
  {
     if s.len()<N && N<=256 {
       Sharedstr{inner:Rc::new(RefCell::new(fixed(tstr::<N>::from(s))))}
     }
     else {Sharedstr{inner:Rc::new(RefCell::new(owned(String::from(s))))}}
  }//make


  /// Creates a `Sharedstr<N>` by consuming a given string.  However, if the
  /// string has length less than N, then a fixed representation will be used.
  pub fn from_string(s:String) -> Self {
    if s.len()<N && N<=256 {Sharedstr{inner:Rc::new(RefCell::new(fixed(tstr::<N>::from(&s[..]))))}}
     else {Sharedstr{inner:Rc::new(RefCell::new(owned(s)))}}
  }

  /// creates a `Sharedstr<N>` from a given `tstr<N>`
  pub fn from_tstr(s:tstr<N>) -> Self {
    Sharedstr{inner:Rc::new(RefCell::new(fixed(s)))}
  }

/*
  #[cfg(feature="serde")]
  /// this function is only added for uniformity in serde implementation
  pub fn try_make(s: &str) -> Result<Sharedstr<N>, &str> {
       Ok(Sharedstr::make(s))
    }
*/

  /// length of the string in bytes. This is a constant-time operation.
  pub fn len(&self) -> usize
  {
    match &*self.inner.borrow() {
      fixed(s) => s.len(),
      owned(s) => s.len(),
    }//match
  }//len
  

    /// converts to `&str` type (may technically panic).
    pub fn as_str(&self) -> &str
    {  unsafe {
         match self.inner.as_ptr().as_ref().unwrap() {
           fixed(s) => s.as_str(),
           owned(s) => s.as_str(),
         }//match
       }//unsafe
    }

  /// creates an empty string, equivalent to [Sharedstr::default]
  pub fn new() -> Self { Self::default() }

  /// length in number of characters as opposed to bytes: this is
  /// not necessarily a constant time operation.
  pub fn charlen(&self) -> usize {
     match &*self.inner.borrow() {
       fixed(s) => s.charlen(),
       owned(s) => {
         s.chars().count()
       },
     }//match
  }//charlen

  /// alias for [Self::as_str]
  pub fn to_str(&self) -> &str
  {
     self.as_str()
  }//to_str

  /// retrieves a copy of the underlying fixed string, if it is a fixed string.
  /// Note that since the `tstr` type is not exported, this function should
  /// be used in conjunction with one of the public aliases [str4]-[str256].
  /// For example,
  /// ```ignore
  ///   let s = Sharedstr::<8>::from("abcd");
  ///   let t:str8 = s.get_str().unwrap();
  /// ```
  pub fn get_str(&self) -> Option<tstr<N>> {
    if let fixed(s) = &*self.inner.borrow() { Some(*s) }
    else {None}
  }//get_str

  /// if the underlying representation of the string is an owned string,
  /// return the owned string, leaving an empty string in its place.
  pub fn take_string(&mut self) -> Option<String>
  {
     if let (ss@owned(_)) = &mut *self.inner.borrow_mut() {
       let mut temp = fixed(tstr::new());
       
       core::mem::swap(ss, &mut temp);
       if let owned(t) = temp {Some(t)} else {None}
     }
     else {None}
  }//take_owned

  /// this function returns a possibly cloned string
  pub fn to_string(self) -> String 
  {
    match &*self.inner.borrow() {
      fixed(s) => s.to_string(),
      owned(s) => s.clone(),
    }//match    
  }//to_string

  /// returns the nth char of the string, if it exists
  pub fn nth(&self, n: usize) -> Option<char> {
    self.to_str().chars().nth(n)
  }

  /// returns the nth byte of the string as a char.  This function
  /// is designed to be quicker than [Sharedstr::nth] and does not check
  /// for bounds.
  pub fn nth_bytechar(&self, n:usize) -> char {
    match &*self.inner.borrow() {
       fixed(s) => s.nth_ascii(n),
       owned(s) => s.as_bytes()[n] as char,
    }
  }//nth_bytechar

  /// alias for [Self::nth_bytechar]
  pub fn nth_ascii(&self, n:usize) -> char { self.nth_bytechar(n) }

  /// returns a u8-slice that represents the underlying string. The first
  /// byte of the slice is **not** the length of the string regarless of
  /// the internal representation.
  pub fn as_bytes(&self) -> &[u8] {
    self.as_str().as_bytes()
  }//as_bytes


  /// changes a character at character position i to c.  This function
  /// requires that c is in the same character class (ascii or unicode)
  /// as the char being replaced.  It never shuffles the bytes underneath.
  /// The function returns true if the change was successful.
  pub fn set(&mut self, i: usize, c: char) -> bool {
     match &mut *self.inner.borrow_mut() {
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
    match &*self.inner.borrow() {
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
     if let fixed(s) = &mut *self.inner.borrow_mut() {f(s);}
  }

  /// applies the destructive closure only if the internal representation
  /// is a fixed string
  pub fn if_owned<F>(&mut self, f:F) where F:FnOnce(&mut str)
  {
     if let owned(s) = &mut *self.inner.borrow_mut() {f(s);}
  }

  /// applies closure f if the internal representation is a fixed string,
  /// or closure g if the internal representation is an owned string.
  pub fn map_or<F,G,U>(&self, f:F, g:G) -> U
    where F:FnOnce(&tstr<N>)-> U, G:FnOnce(&str) -> U
  {
     match &*self.inner.borrow() {
       fixed(s) => f(s),
       owned(s) => g(&s[..]),
     }//match
  }//map

  /// version of [Sharedstr::map_or] accepting FnMut closures
  pub fn map_or_mut<F,G,U>(&mut self, f:&mut F, g:&mut G) -> U
    where F:FnMut(&mut tstr<N>)-> U, G:FnMut(&mut str) -> U
  {
     match &mut *self.inner.borrow_mut() {
       fixed(s) => f(s),
       owned(s) => g(&mut s[..]),
     }//match
  }//map
  
  /// appends the Sharedstr with the given slice,
  /// switching to the owned-String representation if necessary.  The function
  /// returns true if the resulting string uses a `tstr<N>` type, and
  /// false if the representation is an owned string.
  pub fn push_str(&mut self, s:&str) -> bool {
    let mut replacer = None;
    let answer;
    match &mut *self.inner.borrow_mut() {
      fixed(fs) if fs.len()+s.len() < N => { fs.push(s); answer=true;},
      fixed(fs) => {
        let fss = fs.to_string() + s;
        replacer = Some(owned(fss));
        //self.inner.replace(owned(fss));
        answer=false;
      },
      owned(ns) => {ns.push_str(s); answer=false;},
    }//match
    if let Some(r) = replacer {self.inner.replace(r);}
    answer
  }//push_str

   /// appends string with a single character, switching to the String
  /// representation if necessary.  Returns true if resulting string
  /// remains fixed.
  pub fn push(&mut self, c:char) -> bool {
     let clen = c.len_utf8();
     let answer;
     let mut replacer = None;
     match &mut *self.inner.borrow_mut() {
       owned(s) => { s.push(c); answer=false;},
       fixed(s) if s.len()+clen>=N => {
         let mut fss = s.to_string();  fss.push(c);
         replacer = Some(owned(fss));
	 //self.inner.replace(owned(fss));
	 answer=false;
       },
       fixed(s) => {
         let mut buf = [0u8;4];
	 let bstr = c.encode_utf8(&mut buf);
	 s.push(bstr);
	 answer=true;
       }
     }//match
     if let Some(r) = replacer {self.inner.replace(r);}
     answer
  }//push

  /// alias for push
  pub fn push_char(&mut self, c:char) -> bool { self.push(c) }

  /// remove and return last character in string, if it exists
  pub fn pop(&mut self) -> Option<char> {
    if self.len()==0 {return None;}
    let answer;
    let mut replacer = None;
    match &mut *self.inner.borrow_mut() {
      fixed(s) => {answer=s.pop_char();},
      owned(s) if s.len()>N => {answer=s.pop();},
      owned(s) => {  // change representation
        answer = s.pop();
        replacer = Some(fixed(tstr::from(&s)));
      },
    }//match
    if let Some(r) = replacer {self.inner.replace(r);}
    answer
  }//pop

  /// alias for [Self::pop]
  pub fn pop_char(&mut self) -> Option<char> { self.pop() }

  /// this function truncates a string at the indicated byte position,
  /// returning true if the truncated string is fixed, and false if owned.
  /// The operation has no effect if n is larger than the length of the
  /// string.  The operation will **panic** if n is not on a character
  /// boundary, similar to [String::truncate].
  pub fn truncate(&mut self, n: usize) -> bool {
    let mut replacer = None;
    let answer;
    match &mut *self.inner.borrow_mut() {
      fixed(fs) if n<fs.len() => { fs.truncate_bytes(n); answer=true;},
      fixed(_) => {answer=true;},
      owned(s) if n<N => {
        assert!(s.is_char_boundary(n));
        replacer = Some(fixed(tstr::<N>::from(&s[..n])));
        answer=true;
      },
      owned(s) => { if n<s.len() {s.truncate(n);} answer=false;},
    }//match
    if let Some(r) = replacer {
      self.inner.replace(r);
    }
    answer
  }//truncate

  /// resets string to empty
  pub fn clear(&mut self) {
      let mut replacer = None;
      match &mut *self.inner.borrow_mut() {
         fixed(s) => {s.clear();},
         owned(s) => { replacer=Some(fixed(tstr::default()));},
      }
      if let Some(r)=replacer {self.inner.replace(r);}
  }//clear

  /// returns string corresponding to slice indices as a copy or clone.
  pub fn substr(&self, start: usize, end: usize) -> Sharedstr<N> {
    match &*self.inner.borrow() {
      fixed(s) => Sharedstr{inner:Rc::new(RefCell::new(fixed(s.substr(start,end))))},
      owned(s) => Self::from(&s[start..end]),
    }
  }//substr

  /// Splits the string into a `tstr<N>` portion and a String portion.
  /// The structure inherits the fixed part and the String returned will
  /// contain the extra bytes that does not fit.  Example:
  ///
  /// ```
  ///  # use fixedstr::*;
  ///   let mut fs:Sharedstr<4> = Sharedstr::from("abcdefg");
  ///   let extras = fs.split_off();
  ///   assert!( &fs=="abc" && &extras=="defg" && fs.is_fixed());
  /// ```
  pub fn split_off(&mut self) -> String {
    let answer;
    let mut replacer = None;
    match &mut *self.inner.borrow_mut() {
      fixed(s) => { answer = String::default(); }
      owned(s) => {
	 answer = String::from(&s[N-1..]);
         replacer = Some(fixed( tstr::<N>::from(&s[..N-1])));
      },
    }//match
    if let Some(r)=replacer {self.inner.replace(r);}
    answer
  }//split_off


  /// tests strings for content equality
  pub fn equals<T:AsRef<str>>(&self, other:&T) -> bool {
    self == other.as_ref()
  }

  /// tests if two instances of Sharedstr point to the same location,
  /// contrasts with `==`, which always tests for content-equality
  pub fn ptr_eq(&self, other:&Self) -> bool {
    Rc::ptr_eq(&self.inner,&other.inner)
  }

  /// creates a new, non-shared copy of the string
  pub fn deep_clone(&self) -> Self {
    Sharedstr::from(self.as_str())
  }

  /// returns the number of shared references to this string (strong `Rc`
  /// pointers)
  pub fn ptr_count(&self) -> usize {
    Rc::strong_count(&self.inner)
  }


    /// in-place modification of ascii characters to lower-case
    pub fn make_ascii_lowercase(&mut self) {
      match &mut *self.inner.borrow_mut() {
        fixed(s) => { s.make_ascii_lowercase(); },
        owned(s) => {
           s.as_mut_str().make_ascii_lowercase();
        },
      }//match
    }//make_ascii_lowercase

    /// in-place modification of ascii characters to upper-case
    pub fn make_ascii_uppercase(&mut self) {
      match &mut *self.inner.borrow_mut() {
        fixed(s) => { s.make_ascii_uppercase(); },
        owned(s) => {
           s.as_mut_str().make_ascii_uppercase();
        },
      }//match
    }


}//impl Sharedstr


impl<const N:usize> Default for Sharedstr<N> {
  fn default() -> Self { Sharedstr {inner:Rc::new(RefCell::new(fixed(tstr::<N>::default())))} }
}


#[cfg(feature = "flex-str")]
use crate::Flexstr;
#[cfg(feature = "flex-str")]
impl<const N:usize> Sharedstr<N> {
 /// converts and consumes Sharedstr into a Flexstr *if* there is
 /// exactly one strong reference to the Sharedstr.  On failure,
 /// the same Sharedstr is returned as a error.  This function is only available
 /// with the `flex-str` option.
 pub fn to_flexstr(self) -> Result<Flexstr<N>,Sharedstr<N>> {
    extern crate std;
    use std::rc::Rc;
    use crate::shared_structs::Strunion::*;
    match Rc::try_unwrap(self.inner) {
      Ok(x) => { match x.into_inner() {
                   fixed(s) => Ok(Flexstr::from_tstr(s)),
                   owned(s) => Ok(Flexstr::from_string(s)),
                 }//inner match
               },
      Err(r) => {
        Err(Sharedstr{inner:r})
      }
    }//outer match
 }//to_flexstr
}//impl Sharestr


impl<const N:usize> core::ops::Deref for Sharedstr<N>
{
    type Target = str;
    fn deref(&self) -> &Self::Target {
      self.as_str()
    }
}

impl<const N:usize> core::hash::Hash for Sharedstr<N>
{
  fn hash<H:core::hash::Hasher>(&self, state:&mut H) {
    (&*self.inner.borrow()).hash(state)
  }
}//hash

impl<T: AsRef<str> + ?Sized, const N: usize> core::convert::From<&T> for Sharedstr<N> {
    fn from(s: &T) -> Self {
        Self::make(s.as_ref())
    }
}
impl<T: AsMut<str> + ?Sized, const N: usize> core::convert::From<&mut T> for Sharedstr<N> {
    fn from(s: &mut T) -> Self {
        Self::make(s.as_mut())
    }
}


impl<const N: usize> core::cmp::PartialOrd for Sharedstr<N> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
    }
}

impl<const N: usize> core::cmp::Ord for Sharedstr<N> {
    fn cmp(&self, other: &Self) -> Ordering {
      self.as_str().cmp(other.as_str())
    }
}


impl<const N: usize> core::convert::AsRef<str> for Sharedstr<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl<const N: usize> core::convert::AsMut<str> for Sharedstr<N> {
    fn as_mut(&mut self) -> &mut str {
       unsafe {
         match self.inner.as_ptr().as_mut().unwrap() {
           fixed(f) => f.as_mut(),
           owned(s) => s.as_mut(),
         }//match
       }//unsafe
    }//as_mut
}

impl<const N: usize> PartialEq<&str> for Sharedstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other // see below
    } //eq
}

impl<const N: usize> PartialEq<&str> for &Sharedstr<N> {
    fn eq(&self, other: &&str) -> bool {
        &self.to_str() == other
    } //eq
}
impl<'t, const N: usize> PartialEq<Sharedstr<N>> for &'t str {
    fn eq(&self, other: &Sharedstr<N>) -> bool {
        &other.to_str() == self
    }
}
impl<'t, const N: usize> PartialEq<&Sharedstr<N>> for &'t str {
    fn eq(&self, other: &&Sharedstr<N>) -> bool {
        &other.to_str() == self
    }
}

impl<const N: usize> core::fmt::Debug for Sharedstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(&self.to_str())
    }
} // Debug impl


impl<const N: usize> core::fmt::Display for Sharedstr<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

impl<const N: usize> core::fmt::Write for Sharedstr<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result
    {
        self.push_str(s);
        Ok(())
    } //write_str
} //core::fmt::Write trait


impl<const N: usize> core::convert::From<String> for Sharedstr<N> {
    /// *will consume owned string and convert it to a fixed
    /// representation if its length is less than N*
    fn from(s: String) -> Self {
        if s.len()>=N {
	  Sharedstr{inner:Rc::new(RefCell::new(owned(s)))}
	} else {
	  Sharedstr{inner:Rc::new(RefCell::new(fixed(tstr::<N>::from(&s[..]))))}
	}
    }
}//from String


impl<const M: usize> Sharedstr<M> {
  /// returns a copy/clone of the string with new fixed capacity N.
  /// Example:
  /// ```
  ///  # use fixedstr::Sharedstr;
  ///  let mut a:Sharedstr<4> = Sharedstr::from("ab");
  ///  let mut b:Sharedstr<8> = a.resize();
  ///  b.push_str("cdef");
  ///  assert!(b.is_fixed());
  ///  a.push_str("1234");
  ///  assert!(a.is_owned());
  /// ```
  pub fn resize<const N: usize>(&self) -> Sharedstr<N> {
    Sharedstr::from(self)
  }
}

impl<const N:usize> Add<&str> for &Sharedstr<N> {
  type Output = Sharedstr<N>;
  fn add(self, other:&str) -> Self::Output {
    match (&*self.inner.borrow(), other) {
       (owned(a),b) => {
         let mut a2 = a.clone();
         a2.push_str(other);
         Sharedstr{inner:Rc::new(RefCell::new(owned(a2)))}
       },
       (fixed(a), b) if a.len() + b.len() >= N => {
         let mut a2 = a.to_string();
         a2.push_str(b);
         Sharedstr{inner:Rc::new(RefCell::new(owned(a2)))}
       },
       (fixed(a), b) => {
         let mut a2 = *a; //copy
         a2.push(b);
         Sharedstr{inner:Rc::new(RefCell::new(fixed(a2)))}
       }
    }//match
  }
}//Add, Rhs = &str

impl<const N:usize> Add<&Sharedstr<N>> for &str {
  type Output = Sharedstr<N>;
  fn add(self, other:&Sharedstr<N>) -> Sharedstr<N> {
    let mut a2 = Sharedstr::from(self);
    a2.push_str(other);
    a2
  }
}//Add &str on left


/// convenient type aliases for [Sharedstr]
pub type sharedstr8 = Sharedstr<8>;
pub type sharedstr16 = Sharedstr<16>;
pub type sharedstr32 = Sharedstr<32>;
pub type sharedstr64 = Sharedstr<64>;
pub type sharedstr128 = Sharedstr<128>;
pub type sharedstr256 = Sharedstr<256>;
