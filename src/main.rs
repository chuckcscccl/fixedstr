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
   let s1 = str16::from("abc");
   let mut s2 = str8::from("and xyz");
   let s2r = s2.push("...and 1234...");
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
   let s5 = str8::new();
   let ss5 = s5.as_str();
   othertests();
   ztests();
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
  let a:zstr<8> = zstr::from("abcdefg");
  let ab = a.substr(1,5);
  println!("zstr: {}", &a);  
  println!("substring of zstr: {}", &ab);
  let x = "abcd";
  println!("{}", a.substr(0,4));
  println!("{}", x==a.substr(0,4));
  assert_eq!("abcd", a.substr(0,4));
  let os:String = ab.to_string();
  println!("as string: {}",&os);

  let mut uni:ztr8 = zstr::from("aλb");
  println!("unicode string {}, len {}, blen {}", &uni, uni.len(), uni.blen());   
  println!("unicode as str len: {}", uni.to_str().len());
  let u2 = "aλb";
  println!("&str u2 len: {}", u2.len());
  uni.set(1,'μ');
  //uni.set(1,'x');  // no effect
  println!("unicode string {}, len {}, blen {}", &uni, uni.len(), uni.blen());

  let mut b:zstr<16> = ab.resize();
  let br = b.push("xλzabcdefghi");
  b.push("");
  println!("after resize and push: {}, len {}, rest {}, charlen {}, strlen {}",&b,b.len(),&br,b.charlen(),b.to_str().len());

}//ztr tests
