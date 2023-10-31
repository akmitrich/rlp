fn main() {
    
    let re = rlp::nfa::compile(r"(мыла (%w+))");
    println!("{:?}", re);
    let s = "мама мыла раму.";
    dbg!(s.len());
    let m = re.match_all(s);
    println!("{:?}", m);
}

fn _main() {
    let re = rlp::nfa::compile(r"(%a-)(b*)");
    let subj = "&bab&&&";
    let m = re.match_all(subj);
    println!("Regex: {:?}", re);
    println!("NFA: {:?} ({})", m, m.len());
    let pcre = pcre2::bytes::RegexBuilder::new()
        .ucp(true)
        .utf(true)
        .build(r"\a+b*")
        .unwrap();
    let pcre_m = pcre
        .find_iter(subj.as_bytes())
        .map(|x| {
            let m = x.unwrap();
            &subj[m.start()..m.end()]
        })
        .collect::<Vec<_>>();
    println!("PCRE2: {:?} ({})", pcre_m, pcre_m.len());
}
