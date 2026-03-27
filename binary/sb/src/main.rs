use ::clap::Parser;

mod clap;
mod workspace;

fn main() {
    let args = clap::ClapArgs::parse();
    println!("{args:?}")
}
// just sbk
