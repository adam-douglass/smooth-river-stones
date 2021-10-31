
extern crate pest;
#[macro_use]
extern crate pest_derive;

mod root;
mod zone;
mod nom_zone;

// use lalrpop_util::lalrpop_mod;
// lalrpop_mod!(pub parser);

use root::Root;



fn main() {
    yew::start_app::<Root>();
}