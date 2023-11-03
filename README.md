# Rusty Lua Patterns
## Specification
I found only this [Lua Patterns](https://www.gsp.com/cgi-bin/man.cgi?section=7&topic=PATTERNS). Please let me know if there is a better specification of Lua Patterns

## Solution
[Regular Expression Matching: the Virtual Machine Approach](https://swtch.com/~rsc/regexp/regexp2.html) is an absolute stunner.

## API
```rust
    let re = rlp::regex::compile(r"%f[%a]%u+%f[%A]");
    println!("{:?}", re);
    let s = "маМА мЫЛа МЫла РАМУ";
    let m = re.match_all(s);
    println!("{:?}", m);
```
As usual
1. Compile Lua pattern into ```Regex``` struct.
2. Then match any subject via ```match_all``` method of the ```Regex```.
```rust
pub fn match_all<'a>(&self, subj: &'a str) -> Box<[Match<'a>]>
```
```rust
pub struct Match<'a> {
    pub subj: &'a str,
    pub captures: Box<[&'a str]>,
}
```
```captures[0]``` is the matched part of the subject. Other captures is captured strings if they present in the pattern.
## Problem
My code is a recursive execution of the virtual machine bytecode described in [Regular Expression Matching: the Virtual Machine Approach](https://swtch.com/~rsc/regexp/regexp2.html) with some special codes for Lua Patterns:
```rust
enum Code {
    Char(CharacterClass), //%w, %d, %X, etc.
    Captured(usize), // %1, %2, etc.
    Border(char, char), // %b(), %b{}, %bая etc.
    Frontier(CharacterClass), // %f[%a], %f[%A], %f[а-я], etc.
    Jmp(usize),
    Split { x: usize, y: usize },
    Save(usize),
    Match,
}
```
That is why this crate is far far from production quality. You are welcomed to make some improvements to it. Or be inspired to make your own implementation of the Lua patterns in Rust.

Keep in touch!