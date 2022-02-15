use std::io;

macro_rules! parse_input {
    ($x:expr, $t:ident) => ($x.trim().parse::<$t>().unwrap())
}

fn get_cgxcode() -> String {
    let mut input_line = String::new();
    let mut cgxcode = String::new();
    io::stdin().read_line(&mut input_line).unwrap();
    let n = parse_input!(input_line, i32);
    for i in 0..n as usize {
        let mut input_line = String::new();
        io::stdin().read_line(&mut input_line).unwrap();
        let cgxline = input_line.trim_matches('\n').to_string();
        cgxcode.push_str(&cgxline);
    }
    cgxcode
}



fn main() {

    println!("answer");
}
