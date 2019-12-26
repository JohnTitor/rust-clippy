// run-rustfix

#![allow(clippy::All)]
#![warn(clippy::pedantic)]

// check if suggesting similar lint
#[warn(clippy::match_sa_ruf)]
#[warn(clippy::if_not_els)]
#[warn(clippy::inherent_to_strin_shadew_display)]
// check if suggesting similar lowercase lint
#[warn(clippy::Unit_Cm)]
fn main() {}
