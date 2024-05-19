//! **Library for several alternative string types using const generics.**
//!
//!
//!  - The size of some types such as [str8] and [zstr]\<8\>
//!    are 8 bytes, compared to 16 bytes for `&str` on 64bit systems,
//!    providing more efficient ways of representing small strings.
//!  -  Most types (except the optional [Flexstr] and [Sharedstr]) can be
//!    copied and stack-allocated.
//!  -  `#![no_std]` is supported by all but the optional [fstr] type.
//!     Features that use the alloc crate can also be optionally excluded.
//!  -  Unicode is supported by all but the optional [cstr] type.
//!  -  Serde serialization is supported by all but the optional [Sharedstr] type.
//!  -  Select functions are `const`, including const constructors.
//!
//!
//! **COMPATIBILITY NOTICES**:
//!
//! > **With Version 0.5.0, the default availability of some
//!   string types have changed.**  The default configuration is minimalized.
//!   The `std`, `flex-str` and `shared-str`
//!   options are no longer enabled by default.  The crate now
//!   supports **`#![no_std]`** by default.  The `std` option only enables the
//!   [fstr] type, which prints warnings to stderr. **However,** unless
//!   you require one of the types [fstr], [Flexstr] or [Sharedstr], your
//!   build configurations most likely will work as before: the builds will just be
//!   smaller.  If `default-features=false` is already part of your
//!   configuration, it should also work as before.
//!
//! > Another change that could potentially affect backwards compatibility is that
//!   zstr's `Index<usize>` and `IndexMut<usize>` traits, which allow
//!   arbitrary modifications to underlying bytes, is now only available
//!   with the optional `experimental` feature.  Previously, they were
//!   available as default features.
//!
//! **Other Important Recent Updates:**
//!
//! >  **Version 0.5.1 introduced the new *`no-alloc`* option**.  In addition to support
//!    for no_std (for all but the fstr type), this option disables compilation of
//!    any features that use the alloc crate.  This may make some no_std implementations
//!    easier. The default build is no longer minimal (see below).
//!
//! >  As of Version 0.4.6, all string types except for `fstr` support
//! **`#![no_std]`**.
//!
//! >  Starting in Version 0.4.2, the underlying representation of the zero-terminated [zstr]
//! type no longer allows non-zero bytes after the first zero.  In particular,
//! the [zstr::from_raw] function now enforces this rule.
//!
//! >  Starting in Version 0.4.0, warnings about
//! capacity being exceeded are only sent to stderr when using the fstr type.
//! For other types, truncation is done silently. Consider using the
//! `try_make` function or the [core::str::FromStr] trait.
//!
//! <hr>
//!
//! **CRATE OVERVIEW**
//!
//! The two string types that are always provided by this crate are **[zstr]** and **[tstr]**.
//! However, [tstr] is not public by default and should be referenced
//! through the type aliases [str4], [str8], [str16], ...  [str256].
//!
//! - A **[zstr]\<N\>** is represented by a `[u8;N]` array underneath
//!   and can hold zero-terminated, utf-8 strings of up to N-1 bytes.
//! Furthermore, no non-zero bytes can follow the first zero. This
//! allows the length of a `zstr<N>` string to be found in O(log N) time.
//!
//! - The types **[str4]** through **[str256]** are aliases for internal types
//! [tstr]\<4\> through [tstr]\<256\> respectively.  These strings are stored
//! in `[u8;N]` arrays with the first byte holding the length of the
//! string.  Each `tstr<N>` can store strings of up to N-1 bytes, with
//! maximum N=256. Because Rust does not currently provide
//! a way to specify conditions (or type casts) on const generics at
//! compile time, the tstr type is not public by
//! default and can only be used through the aliases.  The `pub-tstr` option
//! makes the `tstr` type public but is not recommended: any `tstr<N>` with
//! `N>256` is not valid and will result in erroneous behavior.
//!
//! In addition, the following string types are available as options:
//!
//! - A **[fstr]\<N\>** stores a string of up to N bytes.
//! It's represented by a `[u8;N]` array and a separate usize variable
//! holding the length.  This type is **enabled with either the `std` or
//! `fstr` option** and some functions will print warnings to stderr when
//! capacity is exceeded. This is the only type that does not support
//! `no_std`, but serde is supported.
//! - The type **[cstr]**, which is **made available
//! with the `circular-str` option**, uses a fixed u8 array
//! that is arranged as a circular queue (aka ring buffer).  This allows
//! efficient implementations of pushing/triming characters *in front* of
//! the string without additional memory allocation.  The downside of these
//! strings is that the underlying representation can be non-contiguous as it allows
//! wrap-around.  As a result, there is no efficient way to implement
//! `Deref<str>`.  Additionally, cstr is the only string type of the crate
//! that does not support Unicode. **Only single-byte characters** are
//! currently supported. There is, however, an iterator over all characters
//! and most common traits are implemented.  Serde and no-std are both supported.
//! - The **[Flexstr]\<N\>** type becomes available with the **`flex-str` option**.
//!   This type uses an internal enum that is either a tstr\<N\>
//!   or an owned String (alloc::string::String) in case the length of the string exceeds N-1.
//!   This type is designed for situations where strings only
//!   occasionally exceed the limit of N-1 bytes. This type does not implement
//!   the `Copy` trait.  Serde and no_std are supported.
//! - The **[Sharedstr]\<N\>** type becomes available with the **`shared-str`
//!   option**. This type is similar to a [Flexstr]\<N\> but uses a
//!   `Rc<RefCell<..>>` underneath to allow strings to be shared as well as
//!   mutated.  This type does not implement `Copy` but `Clone` is done
//!   in constant time.  no_std is supported but **not serde**.
//!
//! **SUMMARY OF OPTIONAL FEATURES**
//!
//! - ***serde*** : Serialization was initially contributed
//!   by [wallefan](https://github.com/wallefan) and adopted to other types
//!   (except `Sharedstr`).  This feature enables the Serialize/Deserialize
//!   traits.
//! - ***circular-str***: this feature makes available the **[cstr]** type.
//! - ***flex-str***: this feature makes available the **[Flexstr]** type.  
//! - ***shared-str***: this feature makes available the **[Sharedstr]** type.
//! - ***std***: this feature cancels `no_std` by enabling the **[fstr]** type.
//!   An alias for this feature name is 'fstr'.
//! - ***pub-tstr***: this feature will make the tstr type public. It is not
//!   recommended: use instead the type aliases [str4] - [str256], which are
//!   always available.
//! - **no-alloc**: this *anti-feature* disables any features that requires the alloc (or std)
//!   crate.  It will disable *entirely* the fstr, Flexstr and Sharedstr types: using
//!   `no-alloc` together with `flex-str`, for example, will not enable the Flexstr type.
//!   It also disables the features in [tstr], [zstr] and [cstr] that require the
//!   alloc crate, in particular any use of alloc::string::String.  Using this feature
//!   is *stronger than no_std*.  Note that when compiled with the `all-features` option, this feature will be included, which will exclude other features.
//! - ***experimental***: the meaning of this feature may change.  Currently
//!   it implements custom Indexing traits for the zstr type, including
//!   `IndexMut<usize>`, which allows individual bytes to be changed
//!   arbitrarily.  Experimental features are not part of the documentation.
//!
//! None of these features is provided by default, so specifying
//! `default-features=false` has no effect.
//!
//! **SAMPLE BUILD CONFIGURATIONS**
//!
//! The simplest way to install this create is to **`cargo add fixedstr`** in your
//! crate or add `fixedstr = "0.5"` to your dependencies in Cargo.toml.
//! The default build makes available the [zstr] type and the type aliases
//! [str4] - [str256] for [tstr].  Serde is not available with this build
//! but no_std is supported, substituting some std features with those from the
//! alloc crate.
//!
//! For **the smallest possible build**, do **`cargo add fixedstr --features no-alloc`**
//! in your crate or add the following in Cargo.toml.
//! ```ignore
//!   [dependencies]
//!   fixedstr = {version="0.5", features=["no-alloc"]}
//! ```
//!
//! To further enable serde serialization, add the following instead:
//! ```ignore
//!   [dependencies]
//!   fixedstr = {version="0.5", features=["serde","no-alloc"]}
//! ```
//! and to exclude `cstr` but include all other features (except `no-alloc`):
//! ```ignore
//!   [dependencies]
//!   fixedstr = {version="0.5", features=["std","flex-str","shared-str","serde","pub-tstr","experimental"]}
//! ```
//! <br>
//!
//! **Do not** install this crate with the `--all-features` option unless you
//! understand that it would include `no-alloc`, which will disable several
//! types and other features of the crate.
//!
//!  ## Examples
//!
//!```
//! use fixedstr::*;
//! let a = str8::from("abcdefg"); //creates new string from &str
//! let a1 = a; // copied, not moved
//! let a2:&str = a.to_str();
//! let a3:String = a.to_string();
//! assert_eq!(a.nth_ascii(2), 'c');
//! let ab = a.substr(1,5);  // copies substring to new str8
//! assert_eq!(ab,"bcde");  // can compare with &str
//! assert_eq!(&a[1..4],"bcd"); // implements Index
//! assert!(a<ab);  // implements Ord (and Hash, Debug, Display, other traits)
//! let mut u:zstr<8> = zstr::from("aλb"); //unicode support
//! {assert_eq!(u.nth(1).unwrap(),'λ');} // nth returns Option<char>
//! assert!(u.set(1,'μ'));  // changes a character of the same character class
//! assert!(!u.set(1,'c')); // .set returns false on failure
//! assert!(u.set(2,'c'));
//! assert_eq!(u, "aμc");
//! assert_eq!(u.len(),4);  // length in bytes
//! assert_eq!(u.charlen(),3);  // length in chars
//! let mut ac:str16 = a.resize(); // copies to larger capacity string
//! let remainder:&str = ac.push("hijklmnopqrst");  //appends string, returns left over
//! assert_eq!(ac.len(),15);
//! assert_eq!(remainder, "pqrst");
//! ac.truncate(10); // shortens string in place
//! assert_eq!(&ac,"abcdefghij");
//! let (upper,lower) = (str8::make("ABC"), str8::make("abc"));
//! assert_eq!(upper, lower.to_ascii_upper()); // no owned String needed
//!  
//! let c1 = str8::from("abcdef"); // string concatenation with + for strN types  
//! let c2 = str8::from("xyz123");
//! let c3 = c1 + c2;       
//! assert_eq!(c3,"abcdefxyz123");   
//! assert_eq!(c3.capacity(),15);  // type of c3 is str16
//!
//! let c4 = str_format!(str16,"abc {}{}{}",1,2,3); // impls core::fmt::Write
//! assert_eq!(c4,"abc 123");  // str_format! truncates if capacity exceeded
//! let c5 = try_format!(str8,"abcdef{}","ghijklmn");
//! assert!(c5.is_none());  // try_format! returns None if capacity exceeded
//!
//! #[cfg(feature = "shared-str")]
//! #[cfg(not(feature = "no-alloc"))]
//! {
//!   let mut s:Sharedstr<8> = Sharedstr::from("abcd");
//!   let mut s2 = s.clone(); // O(1) cost
//!   s.push_char('e');
//!   s2.set(0,'A');
//!   assert_eq!(s2, "Abcde");
//!   assert!(s==s2 && s.ptr_eq(&s2));
//! }
//!
//! #[cfg(feature = "experimental")]
//! {
//!   let mut s = <zstr<8>>::from("abcd");
//!   s[0] = b'A';       // implements IndexMut<usize> (only for zstr)
//!   assert_eq!(&s[0..3],"Abc");
//! }
//! ```
//!
// #![doc = document_features::document_features!()]

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
#[cfg(not(feature = "no-alloc"))]
mod full_fixed;
#[cfg(feature = "std")]
#[cfg(not(feature = "no-alloc"))]
pub use full_fixed::*;

//#[cfg(feature = "flex-str")]
//mod shared_structs;

#[cfg(not(feature = "no-alloc"))]
#[cfg(any(feature = "shared-str", feature = "flex-str"))]
mod shared_structs;

#[cfg(feature = "flex-str")]
#[cfg(not(feature = "no-alloc"))]
mod flexible_string;
#[cfg(feature = "flex-str")]
#[cfg(not(feature = "no-alloc"))]
pub use flexible_string::*;

#[cfg(feature = "shared-str")]
#[cfg(not(feature = "no-alloc"))]
mod shared_string;
#[cfg(feature = "shared-str")]
#[cfg(not(feature = "no-alloc"))]
pub use shared_string::*;

mod zero_terminated;
pub use zero_terminated::*;

mod tiny_internal;
use tiny_internal::*;
#[cfg(feature = "pub_tstr")]
pub use tiny_internal::*;

#[cfg(feature = "circular-str")]
mod circular_string;
#[cfg(feature = "circular-str")]
pub use circular_string::*;

/*
#[cfg(feature = "compressed-str")]
#[cfg(not(feature = "no-alloc"))]
mod compressed;
#[cfg(feature = "compressed-str")]
#[cfg(not(feature = "no-alloc"))]
pub use compressed::*;
*/


//#[macro_use]
//extern crate static_assertions;





#[cfg(feature = "serde")]
mod serde_support {
    use super::*;
    use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};
    macro_rules! generate_impl {
        ($ty: ident, $visitor: ident) => {
            impl<const N: usize> Serialize for $ty<N> {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
        };
    }
    generate_impl!(zstr, ZstrVisitor);
    generate_impl!(tstr, TstrVisitor);
    #[cfg(feature = "std")]
    #[cfg(not(feature = "no-alloc"))]
    generate_impl!(fstr, FstrVisitor);
    #[cfg(feature = "flex-str")]
    #[cfg(not(feature = "no-alloc"))]
    generate_impl!(Flexstr, FlexstrVisitor);

    #[cfg(feature = "circular-str")]
    impl<const N: usize> Serialize for cstr<N> {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let s = self.to_contiguous(); //self.to_string();
            let (a, _) = s.to_strs();
            serializer.serialize_str(a)
        }
    } //serialize

    #[cfg(feature = "circular-str")]
    struct CstrVisitor<const N: usize>;
    #[cfg(feature = "circular-str")]
    impl<'de, const N: usize> Visitor<'de> for CstrVisitor<N> {
        type Value = cstr<N>;
        fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("a string")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<Self::Value, E> {
            cstr::try_make(s).map_err(|_| E::custom("string too long"))
        }
    }

    #[cfg(feature = "circular-str")]
    impl<'de, const N: usize> Deserialize<'de> for cstr<N> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_str(CstrVisitor)
        }
    }
} //serde

/// Types for small strings that use an efficient representation
/// underneath.  Alias for internal type [tstr]\<8\>.
/// A str8 is 8 bytes and can hold string of up to 7 bytes.
/// See documentation for the aliased [tstr] type.
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
/// terms of speed and memory efficiency.
///<br>
/// In addition, the str4-str128 types implement [core::ops::Add] in a way that
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

/// Alias for internal type `tstr<4>`.
/// <br>Holds strings of up to three single-byte chars, good enough to represent abbreviations
/// such as those for states and airports. Each str<4> is exactly 32 bits.
/// Alias for internal type `tstr<4>`.   See documentation for [tstr].
pub type str4 = tstr<4>;
pub type str12 = tstr<12>;
pub type str24 = tstr<24>;
pub type str48 = tstr<48>;
pub type str96 = tstr<96>;
pub type str192 = tstr<192>;


#[macro_export]
/// creates a formated string of given type (by implementing [core::fmt::Write]):
/// ```
///    # use fixedstr::*;
///    let s = str_format!(str8,"abc{}{}{}",1,2,3);
///    assert_eq!(s,"abc123");
/// ```
/// will truncate if capacity exceeded, without warning. See [try_format!]
/// for version that does not truncate.
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

/*
//////////// to string trait
pub trait ToTstr<const N: usize> {
  fn to_tstr(&self) -> tstr<N>;
}//tostring trait
*/

#[macro_export]
/// Macro for converting any expression that implements the Display trait
/// into the specified type, similar to `to_string` but without necessary
/// heap allocation.  Truncation is automatic and silent. Example:
///```
///  # use fixedstr::*;
///  let fs = to_fixedstr!(str8,-0132*2);
///  assert_eq!(&fs,"-264");
///```
/// For version that does not truncate, use [convert_to_str!].
macro_rules! to_fixedstr {
    ($ty_size:ty, $x:expr) => {{
        use core::fmt::Write;
        let mut fstr0 = <$ty_size>::new();
        let res = write!(&mut fstr0, "{}", $x);
        fstr0
    }};
}

#[macro_export]
/// Version of [to_fixedstr!] that returns None instead of truncating .
///```
///  # use fixedstr::*;
///  let fsopt = convert_to_str!(zstr<16>,0.013128009);
///  assert!(matches!(fsopt.as_deref(),Some("0.013128009")))
///```
macro_rules! convert_to_str {
    ($ty_size:ty, $x:expr) => {{
        use core::fmt::Write;
        let mut fstr0 = <$ty_size>::new();
        let res = write!(&mut fstr0, "{}", $x);
        if res.is_ok() {
            Some(fstr0)
        } else {
            None
        }
    }};
}

/////////////////////////////////////////////////////  Testing ...

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn testmain() {
        nostdtest();
        ztests();

        #[cfg(feature = "std")]
        #[cfg(not(feature = "no-alloc"))]
        maintest();
        #[cfg(all(feature = "flex-str", feature = "std"))]
        #[cfg(not(feature = "no-alloc"))]
        flextest();
        #[cfg(feature = "std")]
        #[cfg(not(feature = "no-alloc"))]
        tinytests();
        #[cfg(all(feature = "std", feature = "flex-str"))]
        #[cfg(not(feature = "no-alloc"))]
        poppingtest();
        #[cfg(all(feature = "std", feature = "shared-str"))]
        #[cfg(not(feature = "no-alloc"))]
        strptrtests();
        #[cfg(feature = "pub-tstr")]
        consttests();
    } //testmain

    #[cfg(feature = "std")]
    #[cfg(feature = "shared-str")]
    #[cfg(not(feature = "no-alloc"))]
    fn strptrtests() {
        extern crate std;
        use std::fmt::Write;
        use std::string::String;
        let mut a = Sharedstr::<8>::from("abc12");
        let mut b = a.clone();
        let mut c = Sharedstr::<8>::from("abc");
        c.push_str("12");
        assert!(a == c);
        assert!(a == "abc12");
        b.push('3');
        assert!(a == "abc123");
        assert!("abc123" == b);
    } //strptrtests

    /// test struct
    struct AB(i32, u32);
    impl core::fmt::Display for AB {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{},{}", self.0, self.1)
        }
    }

    #[cfg(all(feature = "std", feature = "flex-str"))]
    #[cfg(not(feature = "no-alloc"))]
    fn poppingtest() {
        extern crate std;
        use std::println;
        let mut a = Flexstr::<8>::from("abcdef");
        assert_eq!(a.pop_char().unwrap(), 'f');
        println!("a: {}", &a);
        let a = flexstr16::from("abcd");
        let c: flexstr16 = &a + "efg";
        assert_eq!(&c, "abcdefg");

        let mut ab = AB(-5, 22 + 1);
        let abfs = to_fixedstr!(zstr<16>, &ab);
        assert_eq!(&abfs, "-5,23");
        let abfs2 = convert_to_str!(zstr<3>, 10003);
        assert!(abfs2.is_none());
    } //poppingtest

    fn nostdtest() {
        let a: str8 = str8::from("abcdef"); //a str8 can hold up to 7 bytes
        let a2 = a; // copied, not moved
        let ab = a.substr(1, 5); // copies substring to new string
        assert_eq!(ab, "bcde"); // compare for equality with &str
        assert_eq!(&a[..3], "abc"); // impls Deref<str>
        assert!(a < ab); // and Ord, Hash, Eq, Debug, Display, other common traits
        let astr: &str = a.to_str(); // convert to &str
        let azstr: zstr<16> = zstr::from(a); // so is zstr
        let mut a32: str32 = a.resize(); // same kind of string but with 31-byte capacity
        a32 = "abc" + a32;
        let mut u = str8::from("aλb"); //unicode support
        assert_eq!(u.nth(1), Some('λ')); // get nth character
        assert_eq!(u.nth_bytechar(3), 'b'); // get nth byte as ascii character
        assert!(u.set(1, 'μ')); // changes a character of the same character class
        assert!(!u.set(1, 'c')); // .set returns false on failure
        assert!(u.set(2, 'c'));
        assert_eq!(u, "aμc");
        assert_eq!(u.len(), 4); // length in bytes
        assert_eq!(u.charlen(), 3); // length in chars
        let mut ac: str16 = a.reallocate().unwrap(); //copies to larger capacity type
        let remainder = ac.push_str("ghijklmnopq"); //append up to capacity, returns remainder
        assert_eq!(ac.len(), 15);
        assert_eq!(remainder, "pq");
        ac.truncate(9); // keep first 9 chars
        assert_eq!(&ac, "abcdefghi");
        let (upper, lower) = (str8::make("ABC"), str8::make("abc"));
        assert_eq!(upper, lower.to_ascii_upper()); // no owned String needed

        let c1 = str8::from("abcd"); // string concatenation with + for strN types
        let c2 = str8::from("xyz");
        assert!(c2.case_insensitive_eq("XyZ"));
        let c2b = str16::from("xYz");
        assert!(c2.case_insensitive_eq(&c2b));
        let mut c3 = c1 + c2;
        assert_eq!(c3, "abcdxyz");
        assert_eq!(c3.capacity(), 15); // type of c3 is str16
        c3 = "00" + c3 + "."; // cat with &str on left or right
        assert_eq!(c3, "00abcdxyz.");

        let c4 = str_format!(str16, "abc {}{}{}", 1, 2, 3); // impls std::fmt::Write
        assert_eq!(c4, "abc 123"); //str_format! truncates if capacity exceeded
        let c5 = try_format!(str8, "abcdef{}", "ghijklmn");
        assert!(c5.is_none()); // try_format! returns None if capacity exceeded

        let fs = to_fixedstr!(str8, -0132);
        assert_eq!(&fs, "-132");

        // testing for constants
        const C:str16 = str16::const_make("abcd");
        //const C:zstr<8> = zstr::const_make("abcd");
        let xarray = [0u8;C.len()];
        assert_eq!(C,"abcd");
        assert_eq!(xarray.len(),4);

        //cstr tests
        #[cfg(feature = "circular-str")]
        {
            use crate::circular_string::*;
            let mut cb = cstr::<16>::make("abc123");
            assert!(cb.is_contiguous());
            cb.push_str("xyz");
            cb.push_front("9876");
            assert_eq!(cb.pop_char().unwrap(), 'z');
            assert_eq!(cb.pop_char_front().unwrap(), '9');
            cb.push_str_front("000");
            assert_eq!(cb.len(), 14);
            assert!(&cb == "000876abc123xy");
            cb.truncate_left(10);
            assert_eq!(&cb, "23xy");
            cb.push_str("ijklmno  ");
            cb.push_char_front(' ');
            assert!(&cb == " 23xyijklmno  ");
            assert!(!cb.is_contiguous());
            //  cb.trim_left();
            //  assert!(&cb == "23xyijklmno ");
            //  cb.trim_right();
            cb.trim_whitespaces();
            assert!("23xyijklmno" == &cb);
            assert!(&cb < "4abc");

            let mut a = cstr::<8>::make("12345678");
            assert_eq!(a.len(), 8);
            a.truncate_front(4);
            assert_eq!(a.len(), 4);
            assert!(a.is_contiguous());
            assert!(&a == "5678");
            a.push_str("abc");
            assert!(&a == "5678abc");
            let mut findopt = a.find_substr("8abc");
            assert_eq!(findopt.unwrap(), 3);
            findopt = a.rfind_substr("678abc");
            assert_eq!(findopt.unwrap(), 1);
            let mut rem = a.push_str("123456");
            assert_eq!(rem, "23456");
            a.truncate_left(4);
            assert_eq!(&a, "abc1");
            rem = a.push_front("qrstuvw");
            assert_eq!(&a, "tuvwabc1");
            assert_eq!(rem, "qrs");
            rem = a.push_str("");
            assert_eq!(&a, "tuvwabc1");
            assert_eq!(rem, "");
            a.truncate(5);
            let mut ba = "123" + a;
            assert_eq!(ba, "123tuvwa");
            ba.truncate_left(4);
            a.truncate_left(1);
            assert_eq!(a, ba);

            #[cfg(feature = "std")]
            {
                let bb = cstr::<8>::from("qgg");
                extern crate std;
                use std::collections::HashSet;
                let mut hh = HashSet::new();
                hh.insert(bb);
                assert!(hh.get(&bb).is_some());
            }
        } //cstr tests
    } //nostdtest

    fn ztests() {
        let a: zstr<8> = zstr::from("abcdefg"); //creates zstr from &str
        let ab = a.substr(1, 5); // copies, not move substring to new string
        assert_eq!(ab, "bcde"); // can compare equality with &str
        assert!(ab.case_insensitive_eq("bCdE"));
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
        ac.pop_char();
        ac.pop_char();
        assert_eq!(ac.len(), 8);
        let mut c4 = str_format!(zstr<16>, "abc {}", 123);
        assert_eq!(c4, "abc 123");
        let rem = c4.push_str("123456789abcdef");
        assert_eq!(c4, "abc 12312345678");
        assert_eq!(rem, "9abcdef");

        let b = [65u8, 66, 67, 0, 0, 68, 0, 69, 0, 70, 0, 71];
        let mut bz: zstr<16> = zstr::from_raw(&b);
        bz.push("abcd   \t \n\n");
        //println!("bz: {}, len {}", &bz, bz.len());
        bz.right_ascii_trim();
        bz.reverse_bytes();
        bz.make_ascii_lowercase();
        //println!("bz after trim, reverse: {}, len {}", &bz, bz.len());
    } //ztr tests

    #[cfg(feature = "std")]
    #[cfg(not(feature = "no-alloc"))]
    fn maintest() {
        extern crate std;
        use std::fmt::Write;
        use std::println;
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

        #[cfg(feature = "experimental")]
        {
            let mut s = <zstr<8>>::from("abcd");
            s[0] = b'A'; // impls IndexMut for zstr (not for fstr nor strN types)
            assert_eq!('A', s.nth_ascii(0));
        }

        use std::collections::HashMap;
        let mut hm = HashMap::new();
        hm.insert(str8::from("abc"), 1);
        assert!(hm.contains_key(&str8::from("abc")));

        let mut a: fstr<8> = fstr::from("abcdef");
        let rem = a.push("g");
        assert!(rem == "" && &a == "abcdefg");

        ftests();
    } //maintest

    #[cfg(feature = "std")]
    #[cfg(not(feature = "no-alloc"))]
    fn ftests() {
        extern crate std;
        use std::{println, string::String, format};
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

        assert_eq!(ac.pop_char().unwrap(), 'j');
        assert_eq!(ac, "abcdefghi");

        let ac2: fstr<16> = fstr::make("abcd");
        ac.truncate(4);
        assert_eq!(ac, ac2);

        let mut z8 = zstr::<16>::from("abc12");
        let z8o = str_format!(zstr<16>,"xxx {}3",z8);
        assert_eq!(z8o, "xxx abc123");
        let zoo = format!("xx{}yy",z8o);
        assert_eq!(zoo,"xxxxx abc123yy");
    } //ftr tests

    #[cfg(all(feature = "std", feature = "flex-str"))]
    #[cfg(not(feature = "no-alloc"))]
    fn flextest() {
        extern crate std;
        use std::fmt::Write;
        use std::println;
        use std::string::String;
        println!("starting Flexstr tests...");
        let mut a: Flexstr<8> = Flexstr::from("abcdef");
        a.truncate(5);
        assert_eq!(a, "abcde"); // can compare equality with &str
        assert_eq!(&a[..3], "abc"); // impls Index
        println!("Flexstr slice: {}", &a[1..4]);
        let ab = Flexstr::<8>::from("bcdefghijklmnop");
        assert!(a.is_fixed());
        assert!(!ab.is_fixed());
        let a2: str8 = a.get_str().unwrap();
        assert!(a < ab); // impls Ord, (and Hash, Debug, Eq, other common traits)
        let astr: &str = a.to_str(); // convert to &str (zero copy)
        let aowned: String = a.to_string(); // convert to owned string
                                            //let b = a.take_string();
        let mut u = Flexstr::<8>::from("aλb"); //unicode support
        assert_eq!(u.nth(1), Some('λ')); // get nth character
        assert_eq!(u.nth_ascii(3), 'b'); // get nth byte as ascii character
        assert!(u.set(1, 'μ')); // changes a character of the same character class
        assert!(!u.set(1, 'c')); // .set returns false on failure
        assert!(u.set(2, 'c'));
        assert_eq!(u, "aμc");
        assert_eq!(u.len(), 4); // length in bytes
        assert_eq!(u.charlen(), 3); // length in chars
        let mut v: Flexstr<4> = Flexstr::from("aμcxyz");
        v.set(1, 'λ');
        println!("v: {}", &v);

        let mut u2: Flexstr<16> = u.resize();
        u2.push_str("aaaaaaaa");
        println!("{} len {}", &u2, u2.len());
        assert!(u2.is_fixed());

        let mut s: Flexstr<8> = Flexstr::from("abcdef");
        assert!(s.is_fixed());
        s.push_str("ghijk");
        assert!(s.is_owned());
        s.truncate(7);
        assert!(s.is_fixed());
        let ab = Flexstr::<32>::from("bcdefghijklmnop");
        println!("size of ab: {}", std::mem::size_of::<Flexstr<32>>());

        let mut vv = Flexstr::<8>::from("abcd");
        vv.push('e');
        //vv.push('λ');
        println!("vv: {}", &vv);

        vv.push_str("abcdefasdfasdfadfssfs");
        let vvs = vv.split_off();
        println!("vv: {},  vvs: {}", &vv, &vvs);

        let mut fs: Flexstr<4> = Flexstr::from("abcdefg");
        let extras = fs.split_off();
        assert!(&fs == "abc" && &extras == "defg" && fs.is_fixed());

        let fss = fs.to_string();
        assert!(&fss == "abc");
    } //flextest

    #[cfg(feature = "std")]
    #[cfg(not(feature = "no-alloc"))]
    fn tinytests() {
        extern crate std;
        use std::fmt::Write;
        use std::println;
        use std::string::String;
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
        assert_eq!(u.nth(1), Some('λ')); // get nth character
        assert_eq!(u.nth_ascii(3), 'b'); // get nth byte as ascii character
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

        let (upper, lower) = (str8::make("ABC"), str8::make("abc"));
        assert_eq!(upper, lower.to_ascii_upper());

        let c1 = str8::from("abcdef");
        let c2 = str8::from("xyz123");
        let c3 = c1 + c2 + "999";
        assert_eq!(c3, "abcdefxyz123999");
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

        let mut c4b = str16::from("abc 12345");
        c4b.truncate(7);
        assert_eq!(c4, c4b);

        let zb = ztr8::from("abc");
        let mut zc = ztr8::from("abcde");
        zc.truncate(3);
        assert_eq!(zb, zc);
    } //tiny tests

    #[cfg(feature = "pub-tstr")]
    fn consttests() {
       let ls = tstr::<{tstr_limit(258)}>::from("abcd");
       assert_eq!(ls.capacity(),255);
    }//consttests
} //tests mod
