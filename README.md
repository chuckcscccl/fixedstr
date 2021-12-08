Library for strings of fixed maximum lengths that can be copied and
stack-allocated using Rust's new const generics feature..  Rust will
probably provide something equivalent in the future, with even more features,
but *just can't wait.*  Been wanting something like this for a long time...

Version 0.1.1:

Ord trait, some minor other conveniences implemented
fstr::new() function now creates empty string. use fstr::from or fstr::make
to create fstr from owned string or str slice.

Version 0.1.2:

as_str() added.  The underlying representation uses [u8; N] arrays, but this
minimally affects the interface.

