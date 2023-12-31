use crate::bytecode::code::Code;
use crate::bytecode::context::Context;

pub(crate) fn exec(ctx: &mut Context) -> bool {
    loop {
        match &ctx.program[ctx.program_counter] {
            Code::Char(c) => {
                let other = ctx.input.get_char(ctx.subj_pointer).copied();
                if other.is_some() && c.is_matched(other.unwrap()) {
                    ctx.program_counter += 1;
                    ctx.subj_pointer += 1;
                } else {
                    return false;
                }
            }
            Code::Captured(n) => {
                let old = ctx.subj_pointer;
                for captured_index in ctx.saved_range(*n) {
                    if ctx.input.get_char(ctx.subj_pointer) == ctx.input.get_char(captured_index) {
                        ctx.subj_pointer += 1;
                    } else {
                        ctx.subj_pointer = old;
                        return false;
                    }
                }
                ctx.program_counter += 1;
            }
            Code::Border(x, y) => {
                let old = ctx.subj_pointer;
                let start = ctx.input.get_char(ctx.subj_pointer);
                if start.is_none() || x != start.unwrap() {
                    return false;
                }
                let mut counter = 1;
                while counter > 0 {
                    ctx.subj_pointer += 1;
                    if let Some(c) = ctx.input.get_char(ctx.subj_pointer) {
                        if x == c {
                            counter += 1;
                        }
                        if y == c {
                            counter -= 1;
                        }
                    } else {
                        ctx.subj_pointer = old;
                        return false;
                    }
                }
                ctx.subj_pointer += 1;
                ctx.program_counter += 1;
            }
            Code::Frontier(s) => {
                let prev = if ctx.subj_pointer == 0 {
                    '\0'
                } else {
                    ctx.input
                        .get_char(ctx.subj_pointer - 1)
                        .copied()
                        .unwrap_or('\0')
                };
                let current = ctx
                    .input
                    .get_char(ctx.subj_pointer)
                    .copied()
                    .unwrap_or('\0');
                if s.is_matched(current) && !s.is_matched(prev) {
                    ctx.program_counter += 1;
                } else {
                    return false;
                }
            }
            Code::Jmp(x) => ctx.program_counter = *x,
            Code::Split { x, y } => {
                ctx.program_counter = *x;
                if exec(ctx) {
                    return true;
                } else {
                    ctx.program_counter = *y;
                }
            }
            Code::Save(x) => {
                let slot = *x;
                let old = ctx.saved[slot];
                ctx.saved[slot] = ctx.subj_pointer;
                ctx.program_counter += 1;
                if exec(ctx) {
                    return true;
                } else {
                    ctx.saved[slot] = old;
                    return false;
                }
            }
            Code::Match => return true,
        }
    }
}

#[cfg(test)]
mod test {
    use pcre2::bytes::RegexBuilder;

    use crate::{regex::compile, Capture};

    #[test]
    fn it_works() {
        let regex = compile("%w+&?");
        let subj = "bab__&&&ghi";
        let m = regex.match_all(subj);
        let pcre = RegexBuilder::new()
            .ucp(true)
            .utf(true)
            .build(r"\w+&?")
            .unwrap();
        let pcre_m = pcre.find_iter(b"bab__&&&ghi").collect::<Vec<_>>();
        assert_eq!(pcre_m.len(), m.len());
        for (m, pcre_m) in m.iter().zip(pcre_m.iter().map(|m| m.as_ref().unwrap())) {
            let Capture::Value(s1) = m.capture(0).unwrap() else {todo!()};
            let s2 = &subj[pcre_m.start()..pcre_m.end()];
            assert_eq!(s1, s2);
        }
    }
}
