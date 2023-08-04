//! Library for strings of fixed maximum lengths that can be copied and
//! stack-allocated using const generics.
//! 
//! **Important Recent Updates:**
//!
//! >  This crate now supports **`#![no_std]`**, although
//! this feature is not enabled by default.  no_std is enabled with the
//! `--no-default-features` option.
//!
//! **COMPATIBILITY NOTICES**:
//!
//! >  Starting in Version 0.4.0, warnings about
//! capacity being exceeded are only sent to stderr when using the fstr type.
//! For other types, truncation is done silently. Consider using the
//! `try_make` function to change this behavior.
//!
//! >  Starting in Version 0.4.2, the underlying representation of the zero-terminated [zstr]
//! type no longer allows non-zero bytes after the first zero.  In particular,
//! the [zstr::from_raw] function now enforces this rule.
//! The modification allows for the length of a `zstr<N>` string to be found 
//! in O(log N) time.
//!
//!
//! **The structures provided by this crate are [fstr], [zstr], [Flexstr]** and **tstr**.
//! However, tstr is not exported by default and should be referenced through the type
//! aliases [str4], [str8], [str16], ...  [str256], as well as indirectly
//! with [Flexstr].  When cargo is given the `no-default-features` option,
//! which enables `#![no_std]` support, only [zstr] and the alias types for
//! tstr are enabled. 
//!
//! The size of (std::mem::size_of) types str8 and zstr<8>
//! are 8 bytes, compared to 16 bytes for &str (on 64bit systems), providing more efficient
//! ways of representing very small strings.  Unicode is supported.
//!
//! The four versions of strings implemented are as follows.
//! - A **[fstr]\<N\>**
//! stores a string of up to N bytes.  It is represented underneath by
//! a `[u8;N]` array and a separate usize variable holding the length.
//! - A **[zstr]\<N\>** is also represented by a `[u8;N]`, without a separate
//! length field, and can hold zero-terminated strings of up to N-1 bytes.
//! **This type supports `#![no_std]`**.
//! - The types **[str4]**, **[str8]** through **[str256]** are aliases for internal types
//! tstr<4> through tstr<256> respectively.  These strings are stored
//! in an array of u8 bytes with the first byte holding the length of the
//! string.  Each tstr\<N\> can store strings of up to N-1 bytes, with
//! maximum N=256. tstr
//! combines the best of fstr and zstr in terms of speed
//! and memory efficiency.  However, because Rust does not currently provide
//! a way to specify conditions on const generics at compile time, such as
//! `where N<=256`, the tstr type is not exported and can
//! only be used through the aliases.  These strings implement essentially
//! the same functions and traits as fstr\<N\> so **the documentation for [fstr]
//! (or [zstr]) also apply to the alias types**.
//! These types **also support `#![no_std]`**.
//! - A **[Flexstr]\<N\>** uses an internal enum that is either a tstr\<N\>
//!   or an owned String, in case the length of the string exceeds N-1.
//!   This type is designed for situations where strings only 
//!   occasionally exceed the limit of N-1 bytes. This type does not implement
//!   the `Copy` trait.
//!
//! **Optional features:**
//!
//! - *`#![no_std]`*: this feature is enabled by the `--no-default-features`
//! option.  
//! Only the [zstr] and tstr types are available with this option.
//! - *serde* : (`--features serde`); Serialization was initially contributed
//! by [wallefan](https://github.com/wallefan) and adopted to other types.
//! This feature can be combined with `--no-default-features` for
//! no_std support.
//! - *pub-tstr*: (`--features pub-tstr`); this feature will make the tstr type public 
//!
//! For example, to enable both no_std and serde, place the following in your
//! `Cargo.toml`:
//! ```ignore
//!   [dependencies]
//!   fixedstr = {version="0.4", features=["serde"], default-features=false}
//! ```
//!
//! **Recent Updates:**
//!
//! Version 0.4.2 improved the implementation of zstr to require all bytes
//! following the first zero byte to also be zeros, which allows the length
//! of the string to be found by binary search.  
//!
//! Version 0.4.0 introduced no_std support
//!
//! Version 0.3.2 introduced the [Flexstr] type.
//!
//! Version 0.3.1 implements `Deref<Target=str>` and removed
//! some redundant procedures.  The functions `to_ascii_lowercase`
//! and `to_ascii_uppercase` **has been renamed to `to_ascii_lower` and
//! `to_ascii_upper`**, to avoid clash with those from the Deref trait.
//!
//! Version 0.2.12 includes contribution from
//! [wallefan](https://github.com/wallefan),
//! and added optional serde support for serialization.
//! This feature can be enabled by giving cargo the
//! **`--features serde`** option.
//!
//! Version 0.2.11 impls [core::fmt::Write], thereby enabling the [write!]
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

//!  ## Examples
//!
//!```ignore
//! let a:fstr<8> = fstr::from("abcdefg"); //creates fstr from &str
//! let a1:fstr<8> = a; // copied, not moved
//! let a2:&str = a.to_str();
//! let a3:String = a.to_string();
//! assert_eq!(a.nth_ascii(2), 'c');
//! let ab = a.substr(1,5);  // copies substring to new fstr
//! assert_eq!(ab,"bcde");  // can compare with &str
//! assert_eq!(&a[1..4],"bcd"); // implements Index
//! assert!(a<ab);  // implements Ord (and Hash, Debug, Display, other traits)
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
//! // New in Version 0.2.11:
//! let c4 = str_format!(str16,"abc {}{}{}",1,2,3); // impls core::fmt::Write
//! assert_eq!(c4,"abc 123");  // str_format! truncates if capacity exceeded
//! let c5 = try_format!(str8,"abcdef{}","ghijklmn");
//! assert!(c5.is_none());  // try_format! returns None if capacity exceeded
//!
//! // New in Version 0.3.0:
//! let mut s = <zstr<8>>::from("abcd");
//! s[0] = b'A';            // implements IndexMut<usize> (only for zstr)
//! assert_eq!(&s[0..3],"Abc");
//! ```
//!
//![zstr] and the type aliases [str4]...[str256] implement the same functions and traits as [fstr].

#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]

#![no_std]

#[cfg(feature = "std")]
mod full_fixed;
#[cfg(feature = "std")]
pub use full_fixed::*;

#[cfg(feature = "std")]
mod flexible_string;
#[cfg(feature = "std")]
pub use flexible_string::*;

mod zero_terminated;
pub use zero_terminated::*;

mod tiny_internal;
use tiny_internal::*;
#[cfg(feature = "pub_tstr")]
pub use tiny_internal::*;

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
                fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("a string")
                }
                fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
                    $ty::try_make(s).map_err(|_| E::custom("string too long"))
                }
            }
        }
    }
    generate_impl!(zstr, ZstrVisitor);
    generate_impl!(tstr, TstrVisitor);
    #[cfg(feature="std")]
    generate_impl!(fstr, FstrVisitor);
    #[cfg(feature="std")]    
    generate_impl!(Flexstr, FlexstrVisitor);
}



/// types for small strings that use a more efficient representation
/// underneath.  A str8 can hold a string of up to 7 bytes (7 ascii chars).
/// The same functions for [fstr] and [zstr] are provided for these types
/// so the documentation for the other types also applies.
/// The size of str8 is 8 bytes.
///
/// Example:
/// ```
///  # use fixedstr::str8;
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
///<br>
/// In addition, the str4-str128 types implement [core::ops::Add].
/// two str8 strings will always concatenate to str16, and similarly for
/// all other strN types up to str128.
///```
///  # use fixedstr::*;
///  let c1 = str8::from("abcd");
///  let c2 = str8::from("xyz");
///  let c3 = c1 + c2;
///  assert_eq!(c3,"abcdxyz");
///  assert_eq!(c3.capacity(),15);
///```

pub type str256 = tstr<256>;

///
/// <br>strings of up to three 8-bit chars, good enough to represent abbreviations
/// such as those for states and airports. Each str<4> is exactly 32 bits.
/// Alias for internal type `tstr<4>`.
pub type str4 = tstr<4>;
pub type str12 = tstr<12>;
pub type str24 = tstr<24>;
pub type str48 = tstr<48>;
pub type str96 = tstr<96>;
pub type str192 = tstr<192>;

#[macro_export]
/// creates a formated string of given type (by implementing [core::fmt::Write]):
/// ```ignore
///    let s = str_format!(str8,"abc{}{}{}",1,2,3);
/// ```
/// will truncate if capacity exceeded, without warning.
macro_rules! str_format {
  ($ty_size:ty, $($args:tt)*) => {
     {use core::fmt::Write;
     let mut fstr0 = <$ty_size>::new();
     let res=write!(&mut fstr0, $($args)*);
     fstr0}
  };
}

#[macro_export]
/// version of [str_format]! that returns an Option of the given type.
/// ```
///   # use fixedstr::*;
///  let s = try_format!(str32,"abcdefg{}","hijklmnop").unwrap();
///  let s2 = try_format!(str8,"abcdefg{}","hijklmnop");
///  assert!(s2.is_none());
/// ```
macro_rules! try_format {
  ($ty_size:ty, $($args:tt)*) => {
     {use core::fmt::Write;
     let mut fstr0 = <$ty_size>::new();
     let result = write!(&mut fstr0, $($args)*);
     if result.is_ok() {Some(fstr0)} else {None}}
  };
}



///////////////////////////  Testing ...

#[cfg(test)]
mod tests {
 use super::*;
#[test]
fn testmain() {
  nostdtest();
  ztests();

  #[cfg(feature = "std")]
  maintest();
  #[cfg(feature = "std")]  
  flextest();
  #[cfg(feature = "std")]  
  tinytests();
  #[cfg(feature = "std")]  
  poppingtest();    
}//testmain

#[cfg(feature = "std")]  
fn poppingtest() {
  extern crate std;
  use std::println;
  let mut a = Flexstr::<8>::from("abcdef");
  assert_eq!(a.pop_char().unwrap(), 'f');
  println!("a: {}",&a);
  let a = flexstr16::from("abcd");
  let c:flexstr16 = &a + "efg";
  assert_eq!(&c,"abcdefg");
}

fn nostdtest() {
  let a:str8 = str8::from("abcdef"); //a str8 can hold up to 7 bytes
  let a2 = a;  // copied, not moved
  let ab = a.substr(1,5);  // copies substring to new string
  assert_eq!(ab, "bcde");  // compare for equality with &str
  assert_eq!(&a[..3], "abc"); // impls Index for Range types
  assert!(a<ab); // and Ord, Hash, Eq, Debug, Display, other common traits
  let astr:&str = a.to_str(); // convert to &str
  let azstr:zstr<16> = zstr::from(a); // so is zstr
  let a32:str32 = a.resize(); // same kind of string but with 31-byte capacity  
  let mut u = str8::from("aλb"); //unicode support
  assert_eq!(u.nth(1), Some('λ'));  // get nth character
  assert_eq!(u.nth_bytechar(3), 'b');  // get nth byte as ascii character
  assert!(u.set(1,'μ'));  // changes a character of the same character class
  assert!(!u.set(1,'c')); // .set returns false on failure
  assert!(u.set(2,'c'));
  assert_eq!(u, "aμc");
  assert_eq!(u.len(),4);  // length in bytes
  assert_eq!(u.charlen(),3);  // length in chars
  let mut ac:str16 = a.reallocate().unwrap(); //copies to larger capacity type
  let remainder = ac.push("ghijklmnopq"); //append up to capacity, returns remainder
  assert_eq!(ac.len(),15);
  assert_eq!(remainder, "pq");
  ac.truncate(9);  // keep first 9 chars
  assert_eq!(&ac,"abcdefghi");
  let (upper,lower) = (str8::make("ABC"), str8::make("abc"));
  assert_eq!(upper, lower.to_ascii_upper()); // no owned String needed

  let c1 = str8::from("abcd"); // string concatenation with + for strN types  
  let c2 = str8::from("xyz");
  let c3 = c1 + c2;           
  assert_eq!(c3,"abcdxyz");
  assert_eq!(c3.capacity(),15);  // type of c3 is str16

  let c4 = str_format!(str16,"abc {}{}{}",1,2,3); // impls std::fmt::Write
  assert_eq!(c4,"abc 123");  //str_format! truncates if capacity exceeded
  let c5 = try_format!(str8,"abcdef{}","ghijklmn");
  assert!(c5.is_none());  // try_format! returns None if capacity exceeded
}//nostdtest


fn ztests() {
    let a: zstr<8> = zstr::from("abcdefg"); //creates zstr from &str
    let ab = a.substr(1, 5); // copies, not move substring to new string
    assert_eq!(ab, "bcde"); // can compare equality with &str
    //println!("zstr: {}", &a);
    let mut u: zstr<8> = zstr::from("aλb"); //unicode support
    assert!(u.set(1, 'μ')); // changes a character of the same character class
    assert!(!u.set(1, 'c')); // .set returns false on failure
    assert!(u.set(2, 'c'));
    assert_eq!(u, "aμc");
    assert_eq!(u.len(), 4); // length in bytes
    assert_eq!(u.charlen(), 3); // length in chars
    let mut ac: zstr<16> = a.resize(); // copies to larger capacity string
    let remainder = ac.push("hijklmnopqrst"); //appends string, returns left over
    assert_eq!(ac.len(), 15);
    assert_eq!(remainder, "pqrst");
    ac.truncate(10);
    assert_eq!(&ac, "abcdefghij");
    //println!("ac {}, remainder: {}, len {}", &ac, &remainder, &ac.len());
    assert_eq!(ac.len(), 10);
    ac.pop_char(); ac.pop_char();
    assert_eq!(ac.len(), 8);    
    let c4 = str_format!(zstr<32>, "abc {}", 123);
    assert_eq!(c4, "abc 123");

    let b = [65u8,66,67,0,0,68,0,69,0,70,0,71];
    let mut bz:zstr<16> = zstr::from_raw(&b);
    bz.push("abcd   \t \n\n");
    //println!("bz: {}, len {}", &bz, bz.len());
    bz.right_ascii_trim();
    bz.reverse_bytes();
    bz.make_ascii_lowercase();
    //println!("bz after trim, reverse: {}, len {}", &bz, bz.len());    
} //ztr tests


#[cfg(feature = "std")]
fn maintest() {
    extern crate std;
    use std::println;
    use std::fmt::Write;
    use std::string::String;
    let s1: fstr<16> = fstr::from("abc");
    let mut s2: fstr<8> = fstr::from("and xyz");
    let s2r = s2.push(" and 1234");
    println!("s1,s2,s2r,s2.len: {}, {}, {}, {}", s1, &s2, &s2r, s2.len());
    println!("{}", &s1 == "abc");
    let s3 = s1; // copied, not moved
    println!("{}", "abc" == &s1);
    println!("{}, {} ", s1 == s3, s1 == s2.resize());

    let mut s4: fstr<256> = s3.resize();
    s4.push("ccccccccccccccccccccccccccccccccccccccccccccccccccccccz");
    println!("{}, length {}", &s4, s4.len());
    let mut s5: fstr<32> = s4.resize();
    println!("{}, length {}", &s5, s5.len());
    println!("{:?}, length {}", &s5[0..10], s5.len());
    println!("s2.substr {}", s2.substr(2, 6));
    println!("{}", s2.substr(2, 6).len());
    let mut s4: fstr<64> = s1.resize();
    let owned_string: String = s4.to_string();
    println!("owned s4: {}", &owned_string);
    let str_slice: &str = s4.to_str();
    println!("as &str: {}", &str_slice[0..2]);
    s4 = s1.resize();
    let s5 = fstr::<8>::new();
    let ss5 = s5.as_str();

    let mut s6 = fstr::<32>::new();
    let result = write!(&mut s6, "hello {}, {}, {}", 1, 2, 3);
    assert_eq!(s6, "hello 1, 2, 3");
    println!("s6 is {}, result is {:?}", &s6, &result);

    let s7 = str_format!(fstr<32>, "abc {}, {}", 1, 10);
    println!("s7 is {}", &s7);
    let s8 = try_format!(fstr<32>, "abcdefg {}, {}", 1, 10);
    println!("s8 is {}", &s8.unwrap());

    let mut f1 = fstr::<16>::from("abcdefg");
    let f2 = f1.to_ascii_uppercase();
    //f1 = f2; // copy?

    let mut s = <zstr<8>>::from("abcd");
    s[0] = b'A';   // impls IndexMut for zstr (not for fstr nor strN types)
    assert_eq!('A', s.nth_ascii(0));
}//maintest

#[cfg(feature = "std")]
fn ftests() {
    extern crate std;
    use std::{string::String,println};
    let a: fstr<8> = fstr::from("abcdefg"); //creates fstr from &str
    let a1: fstr<8> = a; // copied, not moved
    let a2: &str = a.to_str();
    let a3: String = a.to_string();
    assert_eq!(a.nth_ascii(2), 'c');
    let ab = a.substr(1, 5); // copies substring to new fstr
    assert!(ab == "bcde" && a1 == a); // can compare with &str and itself
    assert!(a < ab); // implements Ord trait (and Hash
    let mut u: fstr<8> = fstr::from("aλb"); //unicode support
    u.nth(1).map(|x| assert_eq!(x, 'λ')); // nth returns Option<char>
                                          //for x in u.nth(1) {assert_eq!(x,'λ');} // nth returns Option<char>
    assert!(u.set(1, 'μ')); // changes a character of the same character class
    assert!(!u.set(1, 'c')); // .set returns false on failure
    assert!(u.set(2, 'c'));
    assert_eq!(u, "aμc");
    assert_eq!(u.len(), 4); // length in bytes
    assert_eq!(u.charlen(), 3); // length in chars
    let mut ac: fstr<16> = a.resize(); // copies to larger capacity string
    let remainder: &str = ac.push("hijklmnopqrst"); //appends string, returns left over
    assert_eq!(ac.len(), 16);
    assert_eq!(remainder, "qrst");
    ac.truncate(10); // shortens string in place
    assert_eq!(&ac, "abcdefghij");
    println!("ac {}, remainder: {}", &ac, &remainder);
} //ftr tests


#[cfg(feature = "std")]
fn flextest() {
  extern crate std;
  use std::println;
  use std::fmt::Write;
  use std::string::String;
    println!("starting Flexstr tests...");
    let mut a:Flexstr<8> = Flexstr::from("abcdef");
    a.truncate(5);
    assert_eq!(a, "abcde"); // can compare equality with &str
    assert_eq!(&a[..3], "abc"); // impls Index
    println!("Flexstr slice: {}", &a[1..4]);
    let ab = Flexstr::<8>::from("bcdefghijklmnop");
    assert!(a.is_fixed());
    assert!(!ab.is_fixed());
    let a2:str8 = a.get_str().unwrap();
    assert!(a < ab); // impls Ord, (and Hash, Debug, Eq, other common traits)
    let astr: &str = a.to_str(); // convert to &str (zero copy)
    let aowned: String = a.to_string(); // convert to owned string
    //let b = a.take_string();
    let mut u = Flexstr::<8>::from("aλb"); //unicode support
    assert_eq!(u.nth(1), Some('λ'));  // get nth character
    assert_eq!(u.nth_ascii(3), 'b');  // get nth byte as ascii character
    assert!(u.set(1, 'μ')); // changes a character of the same character class
    assert!(!u.set(1, 'c')); // .set returns false on failure
    assert!(u.set(2, 'c'));
    assert_eq!(u, "aμc");
    assert_eq!(u.len(), 4); // length in bytes
    assert_eq!(u.charlen(), 3); // length in chars
    let mut v:Flexstr<4> = Flexstr::from("aμcxyz");
    v.set(1,'λ');
    println!("v: {}",&v);

    let mut u2:Flexstr<16> = u.resize();
    u2.push_str("aaaaaaaa");
    println!("{} len {}",&u2,u2.len());
    assert!(u2.is_fixed());

    let mut s:Flexstr<8> = Flexstr::from("abcdef");
    assert!(s.is_fixed());
    s.push_str("ghijk");
    assert!(s.is_owned());
    s.truncate(7);
    assert!(s.is_fixed());
    let ab = Flexstr::<32>::from("bcdefghijklmnop");
    println!("size of ab: {}",std::mem::size_of::<Flexstr<32>>());

    let mut vv = Flexstr::<8>::from("abcd");
    vv.push('e');
    //vv.push('λ');
    println!("vv: {}",&vv);

    vv.push_str("abcdefasdfasdfadfssfs");
    let vvs = vv.split_off();
    println!("vv: {},  vvs: {}",&vv,&vvs);

    let mut fs:Flexstr<4> = Flexstr::from("abcdefg");
    let extras = fs.split_off();
    assert!( &fs=="abc" && &extras=="defg" && fs.is_fixed());
}//flextest

#[cfg(feature = "std")]
fn tinytests() {
  extern crate std;
  use std::println;
  use std::string::String;
  use std::fmt::Write;
    println!("starting tstr tests...");
    let a: str8 = str8::from("abcdef");
    let a2 = a; // copied, not moved
    let ab = a.substr(1, 5); // copies, not move substring to new string
    assert_eq!(ab, "bcde"); // can compare equality with &str
    assert_eq!(&a[..3], "abc"); // impls Index
    assert_eq!(ab.len(), 4);
    println!("str8: {}", &a);
    assert!(a < ab); // impls Ord, (and Hash, Debug, Eq, other common traits)
    let astr: &str = a.to_str(); // convert to &str (zero copy)
    let aowned: String = a.to_string(); // convert to owned string
    let afstr: fstr<8> = fstr::from(a); // fstr is another fixedstr crate type
    let azstr: zstr<16> = zstr::from(a); // so is zstr
    let a32: str32 = a.resize(); // same type of string with 31-byte capacity
    let mut u = str8::from("aλb"); //unicode support
    assert_eq!(u.nth(1), Some('λ'));  // get nth character
    assert_eq!(u.nth_ascii(3), 'b');  // get nth byte as ascii character
    assert!(u.set(1, 'μ')); // changes a character of the same character class
    assert!(!u.set(1, 'c')); // .set returns false on failure
    assert!(u.set(2, 'c'));
    assert_eq!(u, "aμc");
    assert_eq!(u.len(), 4); // length in bytes
    assert_eq!(u.charlen(), 3); // length in chars
    let mut ac: str16 = a.reallocate().unwrap(); //copies to larger capacity type
    let remainder = ac.push("ghijklmnopq"); //append up to capacity, returns remainder
    assert_eq!(ac.len(), 15);
    assert_eq!(remainder, "pq");
    println!("ac {}, remainder: {}", &ac, &remainder);
    ac.truncate(9); // keep first 9 chars
    assert_eq!(&ac, "abcdefghi");
    println!("ac {}, remainder: {}", &ac, &remainder);

    let mut s = str8::from("aλc");
    assert_eq!(s.capacity(), 7);
    assert_eq!(s.push("1234567"), "4567");
    assert_eq!(s, "aλc123");
    assert_eq!(s.charlen(), 6); // length in chars
    assert_eq!(s.len(), 7); // length in bytes

    println!("size of str8: {}", std::mem::size_of::<str8>());
    println!("size of zstr<8>: {}", std::mem::size_of::<zstr<8>>());
    println!("size of &str: {}", std::mem::size_of::<&str>());
    println!("size of &str8: {}", std::mem::size_of::<&str8>());

    let mut toosmall: fstr<8> = fstr::make("abcdefghijkl");
    let mut toosmallz: zstr<8> = zstr::make("abcdefghijkl");
    let mut toosmallt: str8 = str8::make("abcdefghijkl");
    println!("toosmall: {}", toosmall);
    let waytoosmall: fstr<4> = toosmall.resize();
    let way2: zstr<4> = toosmallz.resize();
    let mut way3: str16 = str16::make("abcdefedefsfsdfsd");
    let way4: str8 = way3.resize();
    way3 = way4.resize();
    println!("way3: {}, length {}", way3, way3.len());

    // converting to other fixedstr crate types
    let b: str8 = str8::from("abcdefg");
    let mut b2: fstr<32> = fstr::from(b);
    b2.push("hijklmnop");
    println!("b2 is {}", &b2);
    let mut b3: zstr<300> = zstr::from(b);
    b3.push("hijklmnopqrstuvw");
    println!("b3 is {}", &b3);
    let mut b4 = str128::from(b2);
    b4.push("xyz");
    println!("b4 is {}", &b4);

    let (upper,lower) = (str8::make("ABC"), str8::make("abc"));
    assert_eq!(upper, lower.to_ascii_upper());

    let c1 = str8::from("abcdef");
    let c2 = str8::from("xyz123");
    let c3 = c1 + c2;
    assert_eq!(c3, "abcdefxyz123");
    assert_eq!(c3.capacity(), 15);
    //println!("c3 is {}, capacity {}",&c3, &c3.capacity());

    let c4 = str_format!(str16, "abc {}{}{}", 1, 2, 3);
    assert_eq!(c4, "abc 123");
    //    let c4 = str_format!(str16,"abc {}",&c1);
    //    println!("c4 is {}",&c4);
    //assert_eq!(c4,"abc abcdef");
    let c5 = try_format!(str8, "abc {}{}", &c1, &c2);
    assert!(c5.is_none());
    let s = try_format!(str32, "abcdefg{}", "hijklmnop").unwrap();
    let s2 = try_format!(str8, "abcdefg{}", "hijklmnop");
    assert!(s2.is_none());
} //tiny tests

}//tests mod