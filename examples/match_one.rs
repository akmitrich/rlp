fn main() {
    let re = rlp::regex::compile(r"%s(%l+%s%u+)");
    let s = "мама мыла РАМУ.";
    let m = re.match_one(s);
    println!("{:?} -> {:?}", m, m.as_ref().map(|m| m.captured_str()));
}
