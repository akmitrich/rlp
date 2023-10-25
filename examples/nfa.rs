fn main() {
    let re = rlp::nfa::parse(r"(%w-)(&*)");
    let subj = "bab&&&";
    let m = rlp::nfa::regex_match(&re, subj);
    println!("NFA: {:?} ({})", m, m.len());
    let pcre = pcre2::bytes::RegexBuilder::new()
        .ucp(true)
        .utf(true)
        .build(r"\w+?&*")
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
