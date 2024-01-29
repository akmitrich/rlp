fn main() {
    let re = rlp::regex::compile(r"%%(%S[^\\%%]+)%%");
    let s = "Hello, %global_name%! %var_1% = %var% 127%";
    let m = re.match_all(s);
    println!(
        "{:?} -> {:?}",
        m,
        m.iter().map(|m| m.captured_str()).collect::<Vec<_>>()
    );
}
