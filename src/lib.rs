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
//! stores a string of up to N bytes.  It is represented underneath using
//! a \[u8;N\] array and a separate usize variable holding the length.
//! - A **[zstr]\<N\>** stores a zero-terminated string, without a separate
//! length variable, and can hold strings of up to N-1 bytes.
//! **This type supports `#![no_std]`**.
//! - The types **[str4]**, **[str8]** through **[str256]** are aliases for internal type
//! tstr<4> through tstr<256> respectively.  These strings are stored
//! in an array of u8 bytes with the first byte holding the length of the
//! string.  Each tstr\<N\> can store strings of up to N-1 bytes, with
//! maximum N=256. tstr
//! combines the best of fstr and zstr in terms of speed
//! and memory efficiency.  However, because Rust does not currently provide
//! a way to specify conditions on const generics at compile time, such as
//! `where N<=256`, the tstr type is not exported and can
//! only be used through the aliases.  These strings implement the same
//! functions and traits as fstr\<N\> so **the documentation for [fstr]
//! (or [zstr]) also apply to the alias types**.
//! These types **also support `#![no_std]`**.
//! - A **[Flexstr]\<N\>** uses an internal enum that is either a tstr\<N\>
//!   or an owned String, in case the length of the string exceeds N-1.
//!   This type is designed for situations where strings only 
//!   occasionally exceed the limit of N-1 bytes.
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
//! ```
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
/// ```
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
