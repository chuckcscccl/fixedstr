Library for strings of fixed maximum lengths that can be copied and
stack-allocated using Rust's const generics feature.  Certain provided
types such as `zstr<8>` and `str8` are smaller in size than a &str.

#### Examples
```
  let a:str8 = str8::from("abcdef"); //a str8 can hold up to 7 bytes
  let a2 = a;  // copied, not moved
  let ab = a.substr(1,5);  // copies, not move substring to new string
  assert_eq!(ab, "bcde");  // can compare equality with &str
  assert_eq!(ab.len(),4);
  println!("str8: {}", &a);  // impls Display
  assert!(a<ab); // and Ord, Hash, Debug, Eq, other common traits
  let astr:&str = a.to_str(); // convert to &str (zero copy)
  let aowned:String = a.to_string(); // convert to owned string
  let afstr:fstr<8> = fstr::from(a); // fstr is another fixedstr crate type
  let azstr:zstr<16> = zstr::from(a); // so is zstr
  let a32:str32 = a.resize(); // same kind of string but with 31-byte capacity  
  let mut u = str8::from("aλb"); //unicode support
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
```

Consult the documentation for details.
