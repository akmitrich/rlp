use crate::bytecode::code::Code;
use crate::bytecode::context::Context;

pub fn exec(ctx: &mut Context) -> bool {
    loop {
        match &ctx.program[ctx.program_counter] {
            Code::Char(c) => {
                let other = ctx.subj.get(ctx.subj_pointer).map(|(_, other)| other);
                if other.is_some() && c.is_matched(other.unwrap()) {
                    ctx.program_counter += 1;
                    ctx.subj_pointer += 1;
                } else {
                    return false;
                }
            }
            Code::Captured(n) => {
                let old = ctx.subj_pointer;
                for i in ctx.saved_range(*n) {
                    if ctx.subj.get(ctx.subj_pointer).map(|(_, c)| c) == Some(&ctx.subj[i].1) {
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
                let start = ctx.subj[ctx.subj_pointer].1;
                if x != &start {
                    return false;
                }
                let mut counter = 1;
                while counter > 0 {
                    ctx.subj_pointer += 1;
                    if let Some((_, c)) = ctx.subj.get(ctx.subj_pointer) {
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
