#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]

// BWT + RLE Compression.

pub struct compactstr<const N : usize>
{
   chrs : [u8; N],
   olen : usize,  // original length
   clen : usize,  // compact length, including a zero
   // eof : usize,  or, position of first zero
} // compactstr

impl<const N:usize> compactstr<N>
{

  pub fn new() -> Self {
    compactstr {
      chrs : [0;N],
      olen : 0,
      clen : 0,
    }
  }
  fn repeats(&self) -> usize {
    let mut ax = 0;
    let mut counts = [0u8;N];
    for i in 0..self.clen {
      let uc = self.chrs[i] as usize;
      if uc != 0 { ax += counts[uc] as usize; counts[uc] += 1; }
    }//for
    ax
  }// counts the number of character repetitions found

  fn rle(s:&str) -> Self {
    let bytes = s.as_bytes();
    let blen = bytes.len();
    let mut compact = compactstr::new();
    if blen==0 {return compact;}
    compact.chrs[0] = bytes[0];
    compact.chrs[1] = 1; // at least one of these bytes
    let mut ci = 1; // indexes compact.chrs
    for bi in 1..blen { // bi indexes bytes
      if bytes[bi] == bytes[bi-1] {
        compact.chrs[ci] += 1;
      }
      else if ci+2 >= N {break;}    
      else {
        compact.chrs[ci+1] = bytes[bi];
        compact.chrs[ci+2] = 1;
        ci += 2;
      }
    } //for
    compact
  }//rle compression, character followed by instances

  fn inverse_rle<const M:usize>(&self) -> [u8;M] { // truncates
    let mut ax = [0u8;M];
    let mut i = 0; // indexes self.chrs
    let mut k = 0; // indexes ax
    while i+1<self.clen && k<M {
      for _ in 0..self.chrs[i+1] {
        if k<M {ax[k] = self.chrs[i];} else {break;}
        k += 1;
      }// for j
      i += 2;
    }//while
    ax
  }//inverse_rle to another array

} // impl compactstr

/// Counts the number of repeated *bytes* in a string slice.  This function
/// uses a stack-allocated `[u8;256]` array and runs in linear time.
pub fn count_repeats(s:&str) -> usize {
    let bytes = s.as_bytes();
    let mut ax = 0;
    let mut counts = [0u8;256];
    for i in 0..s.len() {
      let uc = bytes[i] as usize;
      ax += counts[uc] as usize;
      counts[uc] += 1;
    }//for
    ax
}//count_repeats

/// calculates the *compression ratio* of a string as the number of repeated
/// characters divided by its length.
pub fn compression_ratio(s:&str) -> f32 {
  let reps = count_repeats(s) as f32;
  if s.len()==0 {0.0} else { reps / (s.len() as f32) }
}

/*
// first attempt, uses type vector plus direct recursion
///// SA-IS algorithm (Nong), false for S, true for L
///// Also identifies locations of LMS substrings as (usize:usize)
///// where the second index is the last position in SA of the bucket for
///// the char, and the first index allows us to fill the bucket from
///// left to right.
fn construct_vector<const M:usize>(t:&[u8], len:usize)
   -> ([bool;M], [(usize,usize);M], usize) {
  assert!(len<M);
  let mut types = [false;M];   // false means S-type
  // assume length of array includes the sentinel 0.
  let mut LMS = [(0,0);M];
  let mut i = len-1;
  //types[i] = false;  // type of sentinel is S (default)
  LMS[0] = (i,len);  
  let mut k = 1; // indexes LMS, gives number of LMS substrings found
  let mut lmsend = len;
  let mut counts = [0usize;256];
  counts[0] = 1; // unique sentinel
  let mut sa = [0usize;M];
  sa[0] = len-1; // smallest suffix is always sentinel by itself
  while i>0 {
    counts[t[i-1] as usize] +=1; 
    // determine types[i-1]
    if t[i-1] > t[i] || (t[i-1]==t[i] && types[i]) { types[i-1] = true; }
    // else stays false by default (S)
    if !types[i-1] && !types[i] {   // if SS
      lmsend = i;  // one past i-1
    }
    else if !types[i-1] && types[i] { // if SL, found new LMS substring
      LMS[k] = (i,lmsend);
      k += 1;
      lmsend = i; // LMS substring starts with the L suffix
    }
    else if types[i-1] && !types[i]  {// LS, found LMS char
      // bucket sort the LMS chars - store where? - chars could be same??
    }
    //else if (LL) do nothing
    i -= 1;  
  }//while i>0
  (types,LMS,k)
}//construct_type_vector
// this runs in O(n) by memoization of types vector
*/

// Section 3.2
// K is always 256
fn rename<const M:usize>(t:&mut [u8;M], sa:&mut [usize;M],n:usize,) -> [usize;M]
{
  assert!(M>=256 && n<M);
  let mut tp = [0usize;M];  // T' - output
  let mut counts = [0usize;256];
  for bi in 0..n { // iterates through t[bi]
    counts[t[bi] as usize] += 1;
  }//for index bi in t
  // prefix sum loop
  let mut sum = 0;
  for k in 0..256 {  // reusing sa
    if counts[k]>0 {
      sa[k] = sum;
      sum += counts[k];
      if sum>=n {break;}
    }
  }//prefix sum loop, meaning of counts[k] changed
  // First loop will assume that all chars are type L, will change later
  for bi in 0..n {
     //set tp[bi] to be start index of tp[bi] char's bucket
     tp[bi] = sa[t[bi] as usize];
  }
  // each char is also first char of a unique suffix
  // can inferr the "buckets" array from counts
  // Second loop will change S type chars to be index of end of buckets
  
  let mut chartype = false; // previous type of sentinel, false=S, true=L
  let mut i = n-1;
  tp[n-1] = 0; //sa[t[n-1] as usize]; // + counts[t[i] as usize] - 1;
  while i>0 {
    chartype = t[i-1]>t[i] || (t[i-1]==t[i] && chartype);
    if !chartype { // t[i-1] is of S type
      tp[i-1] = sa[t[i-1] as usize] + counts[t[i-1] as usize] - 1;
    }
    i -= 1;
  }// while bi>0

  // next step : sort all LMS characters. left-most S type chars
  // Goal is to place the correct, sorted index of each LMS char
  // at the end of the corresponding bucket in SA.
  //Right now, sa stores the starting location of buckets by byte value
  // We'll do it using more memory at first.

  // 3.3 2, (1)
  use SAEntry::*;
  let mut SA = [Counter(0);M];
  SA[0] = Index(n-1);
  let mut prevtype = false; // type of sentinel
  chartype = false;
  i = n-1;
  let mut lmschars = 0; // counter
  while i>0 {
    prevtype = t[i-1]>t[i] || (t[i-1]==t[i] && chartype);
    if !chartype && prevtype { //found LMS char at position i (tp[i] is LMS)
      match SA[tp[i]] {
        Counter(n) => { SA[tp[i]] = Counter(n+1); },
        _ => {},
      }//match
      lmschars += 1;
    }//found LMS char
    chartype = prevtype;
    i -= 1;
  }//while i>0
  // (2)
  chartype = false; // type of sentinel
  i = n-1;
  while i>0 {
    prevtype = t[i-1]>t[i] || (t[i-1]==t[i] && chartype);
    if !chartype && prevtype { //found LMS char at position i (tp[i] is LMS)
      match SA[tp[i]] {
        Counter(1) => {
          SA[tp[i]] = Index(i);
        }
        Counter(count) if count>1 => {
          SA[tp[i]-count+1] = Index(i);
          SA[tp[i]] = Counter(count-1);
        },
        _ => {},
      }//match
    }//found LMS char
    chartype = prevtype;
    i -= 1;
  }//while i>0

  println!("SA: {:?}",&SA[..n]);

  // 3.4 induce sort all LMS-substrings (prefixes) from sorted LMS chars
  i = n-1; // this is S type
  chartype = false;
  while i>0 {
    chartype = t[i-1]>t[i] || (t[i-1]==t[i] && chartype);
    if chartype {  // t[i-1] is L-type
      if let Counter(c) = SA[tp[i]] {
        SA[tp[i]] = Counter(c+1);
      }
    }
    i -= 1;
  }//while i
  SA[0] = Index(n-1);
  for i in 1..n {  // scan SA left to right, SA[0] already correct
    match SA[i] {
      Index(ind) => {
        let j = ind-1;
        //determine if suf(j) is L-type
        if j+1<n && t[j]>t[j+1] || (t[j]==t[j+1] && !SA[j].is_index()) { //L
          let mut lfentry = tp[j];
          while let Index(_) = SA[lfentry] {lfentry += 1;} // expensive!
          SA[lfentry] = Index(j);
        }
      },
      Counter(1) => {  // unique
         SA[tp[i]] = Index(i);
      },
      Counter(c) if c>1 => {
        SA[tp[i] + c-1] = Index(i);
        SA[i] = Counter(c-1);
      },
      _ => {}, // ignore empty entries
    }//match
  } // for first n entries in SA
 

  tp
}//rename
// uses more memory than paper specifies.  - reuse t?


// brute force, use separate type array


fn main() {
  let t0 = "2113311331210".as_bytes();
  let n= t0.len();
  let mut t = [0u8;256];
  t[0..n].copy_from_slice(&t0);
  let mut sa = [0usize;256];
  let tp = rename(&mut t, &mut sa, 13);
  println!("Tprime: {:?}",&tp[..n]);
}//main















///////////////////
// bkt (buckets) entry are (start of LMS in T, end of LMS in T)), index is
// the first byte of the LMS char at the end of LMS.


#[derive(PartialEq,Eq,Copy,Clone,Debug)]
enum SAEntry
{
   Index(usize),
   Counter(usize), // Counter(0) is empty, Counter(1) is unique
}//SAEntry
impl Default for SAEntry {
  fn default() -> Self { SAEntry::Counter(0) }
}//default for SAEntry
impl SAEntry {
  fn is_index(&self) -> bool {
    if let &SAEntry::Index(_) = self {true} else {false}
  }
  fn is_empty(&self) -> bool {
    if let &SAEntry::Counter(0) = self {true} else {false}
  }
}




        /*
        Counter(1) => { SA[tp[i]] = Index(i); },
        Counter(count) if count>1 && tp[i]>0 && SA[tp[i]-1]==Counter(0) => {
          if tp[i]>1 && SA[tp[i]-2].empty() {
            SA[tp[i]-2] = Index(i);
          }
          else {
            SA[tp[i]] = Index(i);
            SA[tp[i]-1] = Counter(0);
          }
        },
        Counter(count) if count>1 && tp[i]>0 => {  //(3)  SA[tp[i]-1] is counter
          match SA[tp[i]-1] {
            Counter(c) if c>0 && tp[i]>=c+2 => {
              if let Counter(0) = SA[tp[i]-c-2] {
                SA[tp[i]-c-2] = Index(i);
                SA[tp[i]-1] = Counter(c+1);
              }
              else {  // reaching another bucket
               // shift?? 
              }
            },
            
          }//match
          
        }
        */
        // if multi, just insert one by one

