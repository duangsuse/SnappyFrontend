// Copyright (c) 2018 duangsuse

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

//! Snappy is a compression/decompression library.
//!
//! It does not aim for maximum compression, or compatibility with any other compression library
//!
//! instead, it aims for very high speeds and reasonable compression.
//!
//! GitHub repository see [google/snappy](https://github.com/google/snappy)
//!
//! Simple Rust binding by duangsuse

#![crate_type = "rlib"]
#![crate_name = "snappy"]

#![doc(html_logo_url = "https://www.rust-lang.org/logos/rust-logo-128x128-blk-v2.png",
  html_favicon_url = "https://doc.rust-lang.org/favicon.ico")]

#![allow(unused_attributes)]
#![feature(libc)]

extern crate libc;
extern crate core;

use core::fmt;

use libc::size_t;
use libc::malloc;

/// Return values for snappy operations
///
/// See the documentation for each function to know what each can return.
#[repr(C)]
pub enum SnappyResult {
  /// Operation succeed, no exception
  Ok = 0,
  /// Bad input buffer given
  InvalidInput = 1,
  /// Allocated buffer too small
  InsufficientBuffer = 2
}

/// `Display` implementation for `SnappyResult`
///
/// + SnappyResult::Ok => "Ok"
/// + SnappyResult::InvalidInput => "Invalid Input"
/// + SnappyResult::InsufficientBuffer => "Insufficient Buffer"
impl fmt::Display for SnappyResult {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      SnappyResult::Ok => f.write_str("Ok"),
      SnappyResult::InvalidInput => f.write_str("Invalid Input"),
      SnappyResult::InsufficientBuffer => f.write_str("Insufficient Buffer"),
    }
  }
}

/// Result type checker methods
///
/// Check if result is ok, or error kind of this result enumeration
///
impl SnappyResult {
  pub fn is_ok(&self) -> bool { match self { SnappyResult::Ok => true, _ => false } }
  pub fn not_ok(&self) -> bool { !self.is_ok() }
  pub fn bad_input(&self) -> bool { match self { SnappyResult::InvalidInput => true, _ => false } }
  pub fn insuff_buf(&self) -> bool { match self { SnappyResult::InsufficientBuffer => true, _ => false } }
}

/// Deflates(compress) a byte slice
pub unsafe extern "C" fn deflate(input: *const u8, length: size_t, buffer_ptr: *mut u8) -> SnappyResult {
  let output_len = snappy_max_compressed_length(length);
  let buffer_ptr = malloc(output_len) as *mut u8;

  snappy_compress(input, length, buffer_ptr, output_len)
}

/// Inflates(uncompress) a byte slice
pub unsafe extern "C" fn inflate(input: *const u8, length: size_t, output: *mut u8) -> SnappyResult {
  let output_len: *mut usize = &mut 0usize;
  let check = snappy_uncompressed_length(input, length, output_len);

  if check.not_ok() { return SnappyResult::InvalidInput }

  snappy_uncompress(input, length, output, *output_len)
}


/// Validate a byte slice
pub unsafe extern "C" fn validate(input: *const u8, length: size_t) -> bool {
  snappy_validate_compressed_buffer(input, length).is_ok()
}

#[link(name = "snappy")]
extern {
  /// Takes the data stored in "input[0..input_length-1]" and stores
  /// it in the array pointed to by "compressed".
  ///
  /// <compressed_length> signals the space available in "compressed".
  /// If it is not at least equal to "snappy_max_compressed_length(input_length)",
  /// SNAPPY_BUFFER_TOO_SMALL is returned. After successful compression,
  /// <compressed_length> contains the true length of the compressed output,
  /// and SNAPPY_OK is returned.
  ///
  /// Example:
  ///   ```c
  ///   size_t output_length = snappy_max_compressed_length(input_length);
  ///   char* output = (char*)malloc(output_length);
  ///   if (snappy_compress(input, input_length, output, &output_length)
  ///       == SNAPPY_OK) {
  ///     ... Process(output, output_length) ...
  ///   }
  ///   free(output);
  ///   ```
  pub fn snappy_compress(input: *const u8, length: size_t, compressed: *mut u8, compressed_length: size_t) -> SnappyResult;

  /// Given data in "compressed[0..compressed_length-1]" generated by
  /// calling the snappy_compress routine, this routine stores
  /// the uncompressed data to
  ///   uncompressed[0..uncompressed_length-1].
  /// Returns failure (a value not equal to SNAPPY_OK) if the message
  /// is corrupted and could not be decrypted.
  ///
  /// <uncompressed_length> signals the space available in "uncompressed".
  /// If it is not at least equal to the value returned by
  /// snappy_uncompressed_length for this stream, SNAPPY_BUFFER_TOO_SMALL
  /// is returned. After successful decompression, <uncompressed_length>
  /// contains the true length of the decompressed output.
  ///
  /// Example:
  ///   ```c
  ///   size_t output_length;
  ///   if (snappy_uncompressed_length(input, input_length, &output_length)
  ///       != SNAPPY_OK) {
  ///     ... fail ...
  ///   }
  ///   char* output = (char*)malloc(output_length);
  ///   if (snappy_uncompress(input, input_length, output, &output_length)
  ///       == SNAPPY_OK) {
  ///     ... Process(output, output_length) ...
  ///   }
  ///   free(output);
  ///   ```
  ///
  pub fn snappy_uncompress(input: *const u8, compressed_length: size_t, uncompressed: *mut u8, uncompressed_length: size_t) -> SnappyResult;


  /// Returns the maximal size of the compressed representation of
  /// input data that is "source_length" bytes in length.
  ///
  pub fn snappy_max_compressed_length(source_length: size_t) -> size_t;

  /// REQUIRES: "compressed\[\]" was produced by snappy_compress()
  /// Returns SNAPPY_OK and stores the length of the uncompressed data in
  /// *result normally. Returns SNAPPY_INVALID_INPUT on parsing error.
  /// This operation takes O(1) time.
  ///
  pub fn snappy_uncompressed_length(compressed: *const u8, compressed_length: size_t, result: *mut size_t) -> SnappyResult;

  /// Check if the contents of "compressed\[\]" can be uncompressed successfully.
  /// Does not return the uncompressed data; if so, returns SNAPPY_OK,
  /// or if not, returns SNAPPY_INVALID_INPUT.
  /// Takes time proportional to compressed_length, but is usually at least a
  /// factor of four faster than actual decompression.
  ///
  pub fn snappy_validate_compressed_buffer(compressed: *const u8, compressed_length: size_t) -> SnappyResult;
}

