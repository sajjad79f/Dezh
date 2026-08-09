use crate::version::Version;

pub fn print_banner() {
    println!();

    println!("==================================================");
    println!("                 {}", Version::PRODUCT);
    println!("      Distributed Enterprise Zero Trust Hub");
    println!("                 {}", Version::VERSION);
    println!("==================================================");

    println!();
}