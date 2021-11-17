//! Library for strings of fixed maximum lengths that can be copied and
//! stack-allocated using Rust's new const generics feature..  Rust will
//! probably provide something equivalent in the future, with even more features,
//! but *just can't wait.*  Been wanting something like this for a long time...

//! Example:
//!
//!```ignore
//!   let s1 = str16::from("abc"); // str16 is alias for fstr<16>
//!   let mut s2 = str32::from("and xyz");
//!   s2.push(" and 1234");  // adds to end of s2
//!   println!("{} {}, {}", s1, &s2, s2.len());  
//!   println!("{}", &s1=="abc");   // can compare with &str
//!   let s3 = s1;     // copied, not moved
//!   println!("{}", "abc"==&s1);
//!   println!("{}, {} ", s1==s3, s1==s2.resize()); 
//!   println!("{}", s2.substr(2,6));
//!   let s4:fstr<64> = s1.resize();  // resize copies to new-capacity fstr
//!   let owned_string = s4.to_string();
//!```

// Fixed, :Copy strings of limited size.  size of each fstr is N or less,
// array chrs is 0-terminated if size of string is less than N:

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(dead_code)]

/// main type: fixed string of size up to N:
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash)]
pub struct fstr<const N:usize>
{
  chrs : [char;N],
  len : usize,  // length will be <=N
}//fstr
impl<const N:usize> fstr<N>
{
  /// creates a new fstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.  This function is also called by
  /// several others including [fstr::from].
  pub fn make(s:&str) -> fstr<N>
   {
      let mut chars = ['\0'; N];
      let mut i = 0;
      for c in s.chars()
      {
         if i<N { chars[i] = c; i+=1; } else {break;}
      }
      fstr {
         chrs: chars,
         len: i,
      }
   }//make

   /// creates an empty string, equivalent to fstr::default()
   pub fn new() -> fstr<N>
   {
     fstr::make("")
   }

   /// length of the string, which will be up to the maximum size N. This
   /// is a constant-time operation.
   pub fn len(&self)->usize { self.len }

   /// converts fstr to an owned string
   pub fn to_string(&self) -> String
   {
     self.chrs[0..self.len].iter().collect()
   }

   /// this will allow non-mutable access to the chars underneath
   pub fn get_chars<'t>(&'t self) -> &'t [char;N] {&self.chrs}

   /// changes a char to c if i is less than the length of the string.
   /// returns true if change was successful.
   pub fn set(&mut self,i:usize, c:char) -> bool
   {
      if i<self.len {self.chrs[i]=c; true} else {false}
   }
   /// adds chars to end of current string up to maximum size N of fstr<N>,
   /// returns the portion of the push string that was NOT pushed due to
   /// capacity, so
   /// if "" is returned then all characters were pushed successfully.
   pub fn push<'t>(&mut self,s:&'t str) -> &'t str
   {
      let mut i = self.len;
      for c in s.chars()
      {
         if i<N {self.chrs[i] = c; i+=1;} else {self.len=N; return &s[N..];}
      }
      self.len = i;
      if (i<s.len()) {&s[i..]} else {""}
   }

   /// returns the nth char of the fstr
   pub fn nth(&self,n:usize) -> Option<char>
   {
      if n<self.len {Some(self.chrs[n])} else {None}
   }

   /// shortens the fstr in-place (mutates).  If n is greater than the
   /// current length of the string, this operation will have no effect.
   pub fn truncate(&mut self, n:usize)
   {
     if n<self.len {self.len = n;}
   }
}//impl fstr<N>

impl<const N:usize> std::convert::From<&str> for fstr<N>
{
  /// creates a new fstr<N> with given &str.  If the length of s exceeds
  /// N, the extra characters are ignored.
  fn from(s:&str) -> fstr<N>
  {
     fstr::make(s)
  }
}

impl<const N:usize> std::convert::From<String> for fstr<N>
{
  fn from(s:String) -> fstr<N>
  {
     fstr::<N>::make(&s[..])
  }
}

impl<const M:usize> fstr<M>
{
  /// converts an fstr\<M\> to an fstr\<N\>. If the length of the string being
  /// converted is greater than N, the extra characters will be ignored.
  /// This operation produces a copy (non-destructive).
  /// Example:
  ///```ignore
  ///  let s1:fstr<8> = fstr::from("abcdefg");
  ///  let s2:fstr<16> = s1.resize(); 
  ///```
  pub fn resize<const N:usize>(&self) -> fstr<N>
  {
     let length = if (self.len<N) {self.len} else {N};
     let mut chars = ['\0';N];
     for i in 0..length {chars[i] = self.chrs[i];}
     fstr {
       chrs: chars,
       len: length,
     }
  }//resize
/*
   pub fn cat<const N:usize, const MN:usize>(&self,s:fstr<N>) -> fstr<MN>
   {
      let ab:fstr<MN> = self.resize();
      
      let mut i = ab.len;
      for c in s.chars()
      {
         if i<N {self.chrs[i] = c; i+=1;} else {self.len=N; return &s[N..];}
      }
      self.len = i;
      &s[i..]
   }
   */
}//impl fstr<M>

/*
impl<const N:usize> std::convert::From<&str> for fstr<N>
{
  fn from(s:&str) -> fstr<N>
  {
     fstr::<N>::make(s)
  }
}
*/

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
      if self.i<self.fs.len {self.i+=1; Some(self.fs.chrs[self.i-1])} else {None}
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

impl<const N:usize> std::fmt::Display for fstr<N>
{
  fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
     write!(f,"{}",self.to_string())
  }
}

impl<const N:usize> PartialEq<&str> for fstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     self==other   // see below
  }//eq
}
impl<const N:usize> PartialEq<&str> for &fstr<N>
{
  fn eq(&self, other:&&str) -> bool
  {
     if other.len()!=self.len {return false;}
     let mut i = 0;
     for c in other.chars()
     {
        if c!=self.chrs[i] {return false;}
        i +=1;
     }
     return true;
  }//eq
}
impl<'t, const N:usize> PartialEq<fstr<N>> for &'t str
{
  fn eq(&self, other:&fstr<N>) -> bool
  { other==self }
}
impl<'t, const N:usize> PartialEq<&fstr<N>> for &'t str
{
  fn eq(&self, other:&&fstr<N>) -> bool
  { other==self }
}

/// defaults to empty string
impl<const N:usize> Default for fstr<N>
{
   fn default() -> Self { fstr::<N>::make("") }
}

///Convert fstr to slice of chars (type [char], not &str)
impl<IndexType,const N:usize> std::ops::Index<IndexType> for fstr<N>
  where IndexType:std::slice::SliceIndex<[char]>,
{
  type Output = IndexType::Output;
  fn index(&self, index:IndexType)-> &Self::Output
  {
     &self.chrs[index]
  }
}//impl Index
// couldn't get it to work properly, [char] is not same as &str
// because there's no allocated string!

impl<const N:usize> fstr<N>
{
  /// returns a copy of the portion of the string, string could be truncated
  /// if indices are out of range. Similar to slice [start..end]
  pub fn substr(&self,start:usize, end:usize) -> fstr<N>
  {
    let mut chars = ['\0';N];
    if start>=self.len || end<=start { return fstr{chrs:chars, len:0}; }
    let mut i = start;
    while i<end && i<self.len
    {
       chars[i-start] = self.chrs[i];
       i += 1;
    }
    fstr { chrs: chars, len:i-start }
  }//substr
}

/// a type for small strings
pub type str8 = fstr<8>; 
pub type str16 = fstr<16>; 
pub type str32 = fstr<32>; 
pub type str64 = fstr<64>; 
pub type str128 = fstr<128>;
