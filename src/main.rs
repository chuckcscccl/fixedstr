#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
//use std::mem;
//mod lib;
//use lib::*;
use fixedstr::*;

fn main()
{
   let s1:fstr<16> = fstr::from("abc");
   let mut s2:fstr<8> = fstr::from("and xyz");
   let s2r = s2.push(" and 1234");
   println!("s1,s2,s2r,s2.len: {}, {}, {}, {}", s1, &s2, &s2r,s2.len());
   println!("{}", &s1=="abc");
   let s3 = s1;     // copied, not moved
   println!("{}", "abc"==&s1);
   println!("{}, {} ", s1==s3, s1==s2.resize());

   let mut s4:fstr<256> = s3.resize();
   s4.push("ccccccccccccccccccccccccccccccccccccccccccccccccccccccz");
   println!("{}, length {}",&s4, s4.len());
   let mut s5:fstr<32> = s4.resize();
   println!("{}, length {}",&s5, s5.len());
   println!("{:?}, length {}",&s5[0..10], s5.len());      
   println!("s2.substr {}", s2.substr(2,6));
   println!("{}", s2.substr(2,6).len());   
let mut s4:fstr<64> = s1.resize();
   let owned_string:String = s4.to_string();
   println!("owned s4: {}",&owned_string);
   let str_slice:&str = s4.to_str();
   println!("as &str: {}",&str_slice[0..2]);
   s4 = s1.resize();
   let s5 = fstr::<8>::new();
   let ss5 = s5.as_str();
   othertests();
   ztests();
   ftests();
   tinytests();
}//main

fn othertests()
{
  let s1:fstr<8> = fstr::from("abcdefg");
  let s2:fstr<16> = s1.resize();
  let s3:fstr<8> = fstr::from("abcdxr");
  println!("cmp test: {}", s3>s1);

//  let s = [65u8, 66,67];
//  let st = &s[..] as &str;

  let chrs = ['a','b','c','\0'];
  // try to coerce into str
  let rawp = (&chrs[..]) as *const [char];
  let raw2 = rawp as *const &str;
  println!("what is raw2? {:?}",raw2); // mem addr
  
  //let string1:&str = mem::transmute::<&[char], &str>(&chrs[0..3]);
  //println!("got str: {:?}",string1);
}//othertests

fn ztests()
{
  let a:zstr<8> = zstr::from("abcdefg"); //creates zstr from &str
  let ab = a.substr(1,5);  // copies, not move substring to new string
  assert_eq!(ab, "bcde");  // can compare equality with &str
  println!("zstr: {}", &a); 
  let mut u:zstr<8> = zstr::from("aλb"); //unicode support
  assert!(u.set(1,'μ'));  // changes a character of the same character class
  assert!(!u.set(1,'c')); // .set returns false on failure
  assert!(u.set(2,'c'));
  assert_eq!(u, "aμc");
  assert_eq!(u.len(),4);  // length in bytes
  assert_eq!(u.charlen(),3);  // length in chars
  let mut ac:zstr<16> = a.resize(); // copies to larger capacity string
  let remainder = ac.push("hijklmnopqrst");  //appends string, returns left over
  assert_eq!(ac.len(),15);
  assert_eq!(remainder, "pqrst");
  ac.truncate(9);
  assert_eq!(&ac,"abcdefghi");  
  println!("ac {}, remainder: {}",&ac, &remainder);
}//ztr tests

fn ftests()
{
  let a:fstr<8> = fstr::from("abcdefg"); //creates fstr from &str
  let a1:fstr<8> = a; // copied, not moved
  let a2:&str = a.to_str();
  let a3:String = a.to_string();
  assert_eq!(a.nth_ascii(2), 'c');
  let ab = a.substr(1,5);  // copies substring to new fstr
  assert!(ab=="bcde" && a1==a);  // can compare with &str and itself
  assert!(a<ab);  // implements Ord trait (and Hash
  let mut u:fstr<8> = fstr::from("aλb"); //unicode support
  for x in u.nth(1) {assert_eq!(x,'λ');} // nth returns Option<char>
  assert!(u.set(1,'μ'));  // changes a character of the same character class
  assert!(!u.set(1,'c')); // .set returns false on failure
  assert!(u.set(2,'c'));
  assert_eq!(u, "aμc");
  assert_eq!(u.len(),4);  // length in bytes
  assert_eq!(u.charlen(),3);  // length in chars
  let mut ac:fstr<16> = a.resize(); // copies to larger capacity string
  let remainder:&str = ac.push("hijklmnopqrst");  //appends string, returns left over
  assert_eq!(ac.len(),16);
  assert_eq!(remainder, "qrst");
  ac.truncate(10); // shortens string in place
  assert_eq!(&ac,"abcdefghij");
  println!("ac {}, remainder: {}",&ac, &remainder);
}//ftr tests

fn tinytests()
{
  println!("starting tstr tests...");
  let a:str8 = str8::from("abcdefg"); //creates zstr from &str
  let ab = a.substr(1,5);  // copies, not move substring to new string
  assert_eq!(ab, "bcde");  // can compare equality with &str
  assert_eq!(ab.len(),4);
  println!("zstr: {}", &a); 
  let mut u:str8 = str8::from("aλb"); //unicode support
  assert!(u.set(1,'μ'));  // changes a character of the same character class
  assert!(!u.set(1,'c')); // .set returns false on failure
  assert!(u.set(2,'c'));
  assert_eq!(u, "aμc");
  assert_eq!(u.len(),4);  // length in bytes
  assert_eq!(u.charlen(),3);  // length in chars
  let mut ac:str16 = a.resize(); // copies to larger capacity string
  let remainder = ac.push("hijklmno");
  assert_eq!(ac.len(),15);
  assert_eq!(remainder, "");
  println!("ac {}, remainder: {}",&ac, &remainder);
  ac.truncate(9);
  assert_eq!(&ac,"abcdefghi");  
  println!("ac {}, remainder: {}",&ac, &remainder);

  let mut s = str8::from("aλc");
  assert_eq!(s.capacity(),7);
  assert_eq!(s.push("1234567"), "4567");
  assert_eq!(s,"aλc123");
  assert_eq!(s.charlen(), 6);
  assert_eq!(s.len(), 7);

  println!("size of str8: {}",std::mem::size_of::<str8>());
  println!("size of zstr<8>: {}",std::mem::size_of::<zstr<8>>());  
  println!("size of &str: {}",std::mem::size_of::<&str>());
  println!("size of &str8: {}",std::mem::size_of::<&str8>());

  let mut toosmall:fstr<8> = fstr::make("abcdefghijkl");
  let mut toosmallz:zstr<8> = zstr::make("abcdefghijkl");
  let mut toosmallt:str8 = str8::make("abcdefghijkl");
  println!("toosmall: {}",toosmall);
  let waytoosmall:fstr<4> = toosmall.resize();
  let way2:zstr<4> = toosmallz.resize();
  let mut way3:str16 = str16::make("abcdefedefsfsdfsd");
  let way4:str8 = way3.resize();
  way3 = way4.resize();
  println!("way3: {}",way3);  
}//tiny tests
