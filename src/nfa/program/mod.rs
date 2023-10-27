mod character_class;
mod code;
mod context;

pub use character_class::CharacterClass;
pub use code::Code;
pub use context::Context;

pub(super) fn exec(ctx: &mut Context) -> bool {
    loop {
        match &ctx.program[ctx.program_counter] {
            Code::Char(c) => {
                let other = ctx.subj.get(ctx.subj_pointer);
                if other.is_some() && c.is_matched(other.unwrap()) {
                    ctx.program_counter += 1;
                    ctx.subj_pointer += 1;
                } else {
                    return false;
                }
            }
            Code::Captured(n) => {
                let old = ctx.subj_pointer;
                for i in ctx.captured_range(*n) {
                    if ctx.subj[ctx.subj_pointer] == ctx.subj[i] {
                        ctx.subj_pointer += 1;
                    } else {
                        ctx.subj_pointer = old;
                        return false;
                    }
                }
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
