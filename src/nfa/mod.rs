mod program;
pub mod regex;

pub use regex::{compile, regex_match};

#[derive(Debug)]
pub struct Match<'a> {
    pub subj: &'a str,
    pub captures: Vec<&'a str>,
}

#[cfg(test)]
mod test {
    use pcre2::bytes::RegexBuilder;

    use super::*;

    #[test]
    fn it_works() {
        let regex = compile("%w+&?");
        let subj = "bab__&&&ghi";
        let m = regex_match(&regex, subj);
        let pcre = RegexBuilder::new()
            .ucp(true)
            .utf(true)
            .build(r"\w+&?")
            .unwrap();
        let pcre_m = pcre.find_iter(b"bab__&&&ghi").collect::<Vec<_>>();
        assert_eq!(pcre_m.len(), m.len());
        for (m, pcre_m) in m.iter().zip(pcre_m.iter().map(|m| m.as_ref().unwrap())) {
            let s1 = *m.captures.first().unwrap();
            let s2 = &subj[pcre_m.start()..pcre_m.end()];
            assert_eq!(s1, s2);
        }
    }
}
