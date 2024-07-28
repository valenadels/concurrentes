///Prints a RED msg
pub fn error(msg: &str) {
    println!("[ROBOTS] \x1b[31m{}\x1b[0m", msg);
}

///Prints a YELLOW msg
pub fn warn(msg: &str) {
    println!("[ROBOTS] \x1b[33m{}\x1b[0m", msg);
}

///Prints a GREEN msg
pub fn info(msg: &str) {
    println!("[ROBOTS] \x1b[32m{}\x1b[0m", msg);
}
