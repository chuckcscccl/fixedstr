#![allow(unused_variables)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_parens)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate std;
use crate::tstr;
use std::string::String;
use Strunion::*;

#[derive(Eq, PartialEq, Hash)]
pub enum Strunion<const N: usize> {
    fixed(tstr<N>),
    owned(String),
} //Strunion

impl<const N: usize> Clone for Strunion<N> {
    fn clone(&self) -> Self {
        match &self {
            fixed(s) => fixed(*s),
            owned(s) => owned(s.clone()),
        } //match
    }
} //impl Clone
