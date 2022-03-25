Library for strings of fixed maximum lengths that can be copied and
stack-allocated using Rust's const generics feature.  Certain provided
types such as `zstr<8>` are smaller in size than a &str.

Version 0.2.3, 0.2.4: minor internal changes

Version 0.2.2: The type aliases str8 through str256 are now bound ot
an internal type.  See docs.

Version 0.2.1: bug fixes and minor adjustments

Version 0.2 adds unicode support and a zero-terminated variant, which is
more memory efficient at the cost of slightly longer runtimes.


Version 0.1.2:

as_str() added.  The underlying representation uses [u8; N] arrays, but this
minimally affects the interface.


Version 0.1.1:

Ord trait, some minor other conveniences implemented
fstr::new() function now creates empty string. use fstr::from or fstr::make
to create fstr from owned string or str slice.
