//! John Walker's moontool.c calculation routines, ported to Rust.
//!
//! This version is released under the 0BSD license, for lack of a
//! proper "public domain license", but:
//!
//! > _Do what thou wilt shall be the whole of the law._
//!
//! Several versions of the tool can be found, including the original
//! Sun Workstation version (moontool), the X Window System version
//! (xmoontool), and two Windows versions (moontoolw), for 16 and
//! 32-bit architectures.
//!
//! The major part of the code comes from the MOONCALC.C file, from
//! moontoolw's 32-bit version (the most recent).
//!
//! Originally, the calculation routines had been extracted and made
//! available only as a C library[^1], with the least possible amount of
//! deviation from the original.
//!
//! This version takes some slight liberties to make the API more usable
//! in comparison to the historical C one. But the calculation routines
//! do not change, and yield the exact same results (see unit tests of
//! both versions).
//!
//! [^1]: <https://github.com/qrichert/moontool/tree/main/moon>
//!
//! ## John Walker
//!
//! - <http://www.fourmilab.ch/>
//! - <https://fourmilab.ch/moontool/>
//! - <https://fourmilab.ch/moontoolw/>

pub mod datetime;
pub mod moon;
