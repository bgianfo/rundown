// Copyright 2019 Brian Gianforcaro

#[cfg(test)]
#[macro_use]
extern crate doc_comment;

// Test examples in the README file.
#[cfg(test)]
doctest!("../README.md");
