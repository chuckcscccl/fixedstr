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
use crate::flex_internal::Strunion::*;

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


#[derive(Clone, Eq, PartialEq, Hash)]
enum Strunion<const N:usize>
{
   fixed(tstr<N>),
   owned(String),
}//Strunion

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Flexstring<const N:usize=32>
{
   inner:Strunion<N>,
}
impl<const N:usize> Flexstring<N>
{
  pub fn make(s:&str) -> Self
  {
     if s.len()<N && N<=256 {Flexstring{inner:fixed(tstr::<N>::from(s))}}
     else {Flexstring{inner:owned(String::from(s))}}
  }//make

  pub fn len(&self) -> usize
  {
    match &self.inner {
      fixed(s) => s.len(),
      owned(s) => s.len(),
    }//match
  }//len

  pub fn new() -> Self { Self::default() }

  pub fn charlen(&self) -> usize {
     match &self.inner {
       fixed(s) => s.charlen(),
       owned(s) => {
         let v: Vec<_> = s.chars().collect();
         v.len()
       },
     }//match
  }//charlen

  pub fn to_str(&self) -> &str
  {
    match &self.inner {
      fixed(s) => s.to_str(),
      owned(s) => &s[..],
    }//match
  }//to_str

  pub fn get_str(&self) -> Option<tstr<N>> {
    if let fixed(s) = &self.inner { Some(*s) }
    else {None}
  }//get_str

  pub fn take_owned(&mut self) -> Option<String>
  {
     if let owned(s) = &mut self.inner {
       let mut temp = String::new();
       std::mem::swap(s,&mut temp);
       self.inner= fixed(tstr::new());
       Some(temp)
     }
     else {None}
  }//take_owned

  pub fn to_string(self) -> String 
  {
    match self.inner {
      fixed(s) => s.to_string(),
      owned(s) => s,
    }//match    
  }//to_owned


  pub fn is_owned(&self) -> bool {
    match &self.inner {
      fixed(_) => false,
      owned(_) => true,
    }
  }//is_owned

  pub fn if_fixed<F>(&mut self, f:&mut F) where F:FnMut(&mut tstr<N>)
  {
     if let fixed(s) = &mut self.inner {f(s);}
  }

  pub fn map_fixed<F,U>(&mut self, f:&mut F) -> Option<U>
    where F:FnMut(&mut tstr<N>)-> U
  {
     if let fixed(s) = &mut self.inner {Some(f(s))} else {None}
  }

  pub fn map_mut<F,U>(&mut self, f:&mut F) -> U
    where F:FnMut(&mut str)-> U
  {
     match &mut self.inner {
       fixed(s) => f(s.as_mut()),
       owned(s) => f(&mut s[..]),
     }//match
  }//map

  pub fn map<F,U>(&self, f:&F) -> U
    where F:Fn(&str)-> U
  {
     match &self.inner {
       fixed(s) => f(s.to_str()),
       owned(s) => f(&s[..]),
     }//match
  }//map


  pub fn map_or<F,G,U>(&self, f:&F, g:&G) -> U
    where F:Fn(&tstr<N>)-> U, G:Fn(&str) -> U
  {
     match &self.inner {
       fixed(s) => f(s),
       owned(s) => g(&s[..]),
     }//match
  }//map

  pub fn map_or_mut<F,G,U>(&mut self, f:&mut F, g:&mut G) -> U
    where F:FnMut(&mut tstr<N>)-> U, G:FnMut(&mut str) -> U
  {
     match &mut self.inner {
       fixed(s) => f(s),
       owned(s) => g(&mut s[..]),
     }//match
  }//map

  

  /*
  pub fn if_owned<F>(&mut self, f:&mut F) where F:FnMut(&mut tstr<N>)
  {
     if let fixed(s) = &mut self.inner {f(s);}
  }
  */
  
  /// this function will append the Flexstring with the given slice,
  /// switching to the owned representation if necessary.  The function
  /// returns true if the representation still uses a `tstr<N>` type, and
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

  pub fn truncate(&mut self, n: usize) {
    match &mut self.inner {
      fixed(fs) if n<fs.len() => { fs.truncate(n); },
      owned(s) if n<N => {
        self.inner = fixed(tstr::<N>::from(&s[..n]));
      },
      owned(s) if n<s.len() => { s.truncate(n); }
      _ => {},
    }//match
  }//truncate

} //impl<N>

impl<const N:usize> Default for Flexstring<N> {
  fn default() -> Self { Flexstring {inner:fixed(tstr::<N>::default())} }
}
