fn main() {
    let s = "2024-01-29";
    let mut vars = std::collections::HashMap::<String, Vec<String>>::new();

    let day_re = rlp::regex::compile(r"%d%d$");
    let month_match = day_re.match_all(s);
    vars.insert(
        "day".to_owned(),
        month_match
            .iter()
            .map(|m| m.captured_str().iter().map(|day| day.to_string()).collect())
            .collect(),
    );
    let month_re = rlp::regex::compile(r"%d%d");
    let month_match = month_re.match_all(s);
    vars.insert(
        "month".to_owned(),
        month_match
            .iter()
            .map(|m| {
                m.captured_str()
                    .iter()
                    .map(|month| month.to_string())
                    .collect()
            })
            .collect(),
    );
    let year_re = rlp::regex::compile(r"%d%d%d%d");
    let year_match = year_re.match_all(s);
    vars.insert(
        "year".to_owned(),
        year_match
            .iter()
            .map(|m| {
                m.captured_str()
                    .iter()
                    .map(|year| year.to_string())
                    .collect()
            })
            .collect(),
    );

    println!("Vars: {:#?}", vars);
}
