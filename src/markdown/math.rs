// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! LaTeX to Office Math (OMML), the inverse of the reader's `math_inner`
//! (`from_docx/mod.rs`). Every structure the reader prints comes back as
//! the element Word wrote, and the commands a person types land on the
//! element Word's equation editor builds for them: balanced brackets become
//! a delimiter (`m:d`), as when Word builds up `f(x)`. An unknown command
//! stays as its text, never dropped.
//!
//! The tables here are shared with the reader, so a command reads back as
//! the character it wrote.

use std::fmt::Write as _;

use super::xml::{Kind, escape};

/// Where an insertion inside the math starts (`{++`) and ends (`++}`), and
/// a deletion (`{--`, `--}`): the characters CriticMarkup's stand-ins use.
pub(super) const INSERT_START: char = '\u{2E0E}';
pub(super) const INSERT_END: char = '\u{2E0F}';
pub(super) const DELETE_START: char = '\u{2E10}';
pub(super) const DELETE_END: char = '\u{2E11}';

/// The Office Math namespace.
pub(super) const M_NS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";

/// The font Word puts on every math run and control.
const FONT: &str = "<w:rPr><w:rFonts w:ascii=\"Cambria Math\" w:hAnsi=\"Cambria Math\"/></w:rPr>";

/// Function names LaTeX sets upright with a command of their own.
pub(super) const FUNCTIONS: [&str; 33] = [
    "sin", "cos", "tan", "cot", "sec", "csc", "sinh", "cosh", "tanh", "coth", "arcsin", "arccos",
    "arctan", "log", "ln", "lg", "exp", "lim", "liminf", "limsup", "max", "min", "sup", "inf",
    "det", "gcd", "Pr", "arg", "deg", "dim", "hom", "ker", "mod",
];

/// Function names whose scripts Word sets under and over them (`m:limLow`,
/// `m:limUpp`) rather than beside them.
const LIMIT_FUNCTIONS: [&str; 9] = [
    "lim", "liminf", "limsup", "max", "min", "sup", "inf", "det", "gcd",
];

/// N-ary operators by command (`m:nary`'s `m:chr`).
pub(super) const NARY: [(&str, char); 30] = [
    ("int", '∫'),
    ("sum", '∑'),
    ("prod", '∏'),
    ("coprod", '∐'),
    ("iint", '∬'),
    ("iiint", '∭'),
    ("oint", '∮'),
    ("oiint", '∯'),
    ("oiiint", '∰'),
    ("bigcup", '⋃'),
    ("bigcap", '⋂'),
    ("bigwedge", '⋀'),
    ("bigvee", '⋁'),
    ("bigodot", '⨀'),
    ("bigoplus", '⨁'),
    ("bigotimes", '⨂'),
    ("biguplus", '⨄'),
    ("bigsqcup", '⨆'),
    ("intop", '∫'),
    ("smallint", '∫'),
    ("bigsqcap", '⨅'),
    ("bigtimes", '⨉'),
    ("fint", '⨏'),
    ("iiiint", '⨌'),
    ("intBar", '⨎'),
    ("intbar", '⨍'),
    ("ointctrclockwise", '∳'),
    ("sqint", '⨖'),
    ("sumint", '⨋'),
    ("varointclockwise", '∲'),
];

/// Accents by command (`m:acc`'s `m:chr`). The first command of a
/// character is the one the reader prints.
pub(super) const ACCENTS: [(&str, char); 15] = [
    ("hat", '\u{302}'),
    ("widehat", '\u{302}'),
    ("tilde", '\u{303}'),
    ("widetilde", '\u{303}'),
    ("bar", '\u{305}'),
    ("dot", '\u{307}'),
    ("ddot", '\u{308}'),
    ("dddot", '\u{20db}'),
    ("vec", '\u{20d7}'),
    ("overrightarrow", '\u{20d7}'),
    ("overleftarrow", '\u{20d6}'),
    ("check", '\u{30c}'),
    ("breve", '\u{306}'),
    ("acute", '\u{301}'),
    ("grave", '\u{300}'),
];

/// Characters LaTeX writes as a command: Greek, operators, relations,
/// arrows and the literal brackets and specials the reader escapes.
const SYMBOLS: &[(&str, &str)] = &[
    ("alpha", "α"),
    ("beta", "β"),
    ("gamma", "γ"),
    ("delta", "δ"),
    ("epsilon", "ϵ"),
    ("varepsilon", "ε"),
    ("zeta", "ζ"),
    ("eta", "η"),
    ("theta", "θ"),
    ("vartheta", "ϑ"),
    ("iota", "ι"),
    ("kappa", "κ"),
    ("lambda", "λ"),
    ("mu", "μ"),
    ("nu", "ν"),
    ("xi", "ξ"),
    ("omicron", "ο"),
    ("pi", "π"),
    ("varpi", "ϖ"),
    ("rho", "ρ"),
    ("varrho", "ϱ"),
    ("sigma", "σ"),
    ("varsigma", "ς"),
    ("tau", "τ"),
    ("upsilon", "υ"),
    ("phi", "ϕ"),
    ("varphi", "φ"),
    ("chi", "χ"),
    ("psi", "ψ"),
    ("omega", "ω"),
    ("Gamma", "Γ"),
    ("Delta", "Δ"),
    ("Theta", "Θ"),
    ("Lambda", "Λ"),
    ("Xi", "Ξ"),
    ("Pi", "Π"),
    ("Sigma", "Σ"),
    ("Upsilon", "Υ"),
    ("Phi", "Φ"),
    ("Psi", "Ψ"),
    ("Omega", "Ω"),
    ("pm", "±"),
    ("mp", "∓"),
    ("times", "×"),
    ("div", "÷"),
    ("cdot", "⋅"),
    ("ast", "∗"),
    ("star", "⋆"),
    ("circ", "∘"),
    ("bullet", "∙"),
    ("oplus", "⊕"),
    ("ominus", "⊖"),
    ("otimes", "⊗"),
    ("odot", "⊙"),
    ("setminus", "∖"),
    ("cup", "∪"),
    ("cap", "∩"),
    ("wedge", "∧"),
    ("land", "∧"),
    ("vee", "∨"),
    ("lor", "∨"),
    ("neg", "¬"),
    ("lnot", "¬"),
    ("le", "≤"),
    ("leq", "≤"),
    ("ge", "≥"),
    ("geq", "≥"),
    ("ne", "≠"),
    ("neq", "≠"),
    ("ll", "≪"),
    ("gg", "≫"),
    ("approx", "≈"),
    ("equiv", "≡"),
    ("sim", "∼"),
    ("simeq", "≃"),
    ("cong", "≅"),
    ("propto", "∝"),
    ("prec", "≺"),
    ("succ", "≻"),
    ("preceq", "⪯"),
    ("succeq", "⪰"),
    ("in", "∈"),
    ("notin", "∉"),
    ("ni", "∋"),
    ("subset", "⊂"),
    ("supset", "⊃"),
    ("subseteq", "⊆"),
    ("supseteq", "⊇"),
    ("perp", "⊥"),
    ("parallel", "∥"),
    ("mid", "∣"),
    ("nmid", "∤"),
    ("vdash", "⊢"),
    ("models", "⊨"),
    ("to", "→"),
    ("rightarrow", "→"),
    ("leftarrow", "←"),
    ("gets", "←"),
    ("leftrightarrow", "↔"),
    ("Rightarrow", "⇒"),
    ("Leftarrow", "⇐"),
    ("Leftrightarrow", "⇔"),
    ("implies", "⟹"),
    ("iff", "⟺"),
    ("mapsto", "↦"),
    ("uparrow", "↑"),
    ("downarrow", "↓"),
    ("longrightarrow", "⟶"),
    ("longleftarrow", "⟵"),
    ("infty", "∞"),
    ("partial", "∂"),
    ("nabla", "∇"),
    ("forall", "∀"),
    ("exists", "∃"),
    ("nexists", "∄"),
    ("emptyset", "∅"),
    ("varnothing", "∅"),
    ("angle", "∠"),
    ("triangle", "△"),
    ("hbar", "ℏ"),
    ("ell", "ℓ"),
    ("Re", "ℜ"),
    ("Im", "ℑ"),
    ("aleph", "ℵ"),
    ("wp", "℘"),
    ("prime", "′"),
    ("degree", "°"),
    ("ldots", "…"),
    ("dots", "…"),
    ("cdots", "⋯"),
    ("vdots", "⋮"),
    ("ddots", "⋱"),
    ("therefore", "∴"),
    ("because", "∵"),
    ("langle", "⟨"),
    ("rangle", "⟩"),
    ("lfloor", "⌊"),
    ("rfloor", "⌋"),
    ("lceil", "⌈"),
    ("rceil", "⌉"),
    ("lvert", "|"),
    ("rvert", "|"),
    ("vert", "|"),
    ("Vert", "‖"),
    ("lparen", "("),
    ("rparen", ")"),
    ("lbrack", "["),
    ("rbrack", "]"),
    ("lbrace", "{"),
    ("rbrace", "}"),
    ("backslash", "\\"),
    ("quad", "\u{2003}"),
    ("qquad", "\u{2003}\u{2003}"),
    // AMS and unicode-math names (unicode-math-table.tex, or the Unicode
    // character name where unicode-math has none).
    ("Box", "□"),
    ("Bumpeq", "≎"),
    ("Cap", "⋒"),
    ("Cup", "⋓"),
    ("Diamond", "◇"),
    ("Digamma", "Ϝ"),
    ("Downarrow", "⇓"),
    ("Finv", "Ⅎ"),
    ("Game", "⅁"),
    ("Longleftarrow", "⟸"),
    ("Longleftrightarrow", "⟺"),
    ("Longrightarrow", "⟹"),
    ("Lsh", "↰"),
    ("Rsh", "↱"),
    ("Subset", "⋐"),
    ("Supset", "⋑"),
    ("Uparrow", "⇑"),
    ("VDash", "⊫"),
    ("Vdash", "⊩"),
    ("Vvdash", "⊪"),
    ("adots", "⋰"),
    ("amalg", "⨿"),
    ("approxeq", "≊"),
    ("asymp", "≍"),
    ("backepsilon", "϶"),
    ("backprime", "‵"),
    ("backsim", "∽"),
    ("backsimeq", "⋍"),
    ("barwedge", "⊼"),
    ("beth", "ℶ"),
    ("between", "≬"),
    ("bigcirc", "◯"),
    ("bigstar", "★"),
    ("bigtriangledown", "▽"),
    ("bigtriangleup", "△"),
    ("blacklozenge", "⧫"),
    ("blacksquare", "■"),
    ("blacktriangle", "▴"),
    ("blacktriangledown", "▾"),
    ("bot", "⊥"),
    ("boxdot", "⊡"),
    ("boxminus", "⊟"),
    ("boxplus", "⊞"),
    ("boxtimes", "⊠"),
    ("bumpeq", "≏"),
    ("cdotp", "·"),
    ("checkmark", "✓"),
    ("circeq", "≗"),
    ("circlearrowleft", "↺"),
    ("circlearrowright", "↻"),
    ("circledR", "®"),
    ("circledS", "Ⓢ"),
    ("circledast", "⊛"),
    ("circledcirc", "⊚"),
    ("circleddash", "⊝"),
    ("clubsuit", "♣"),
    ("coloneq", "≔"),
    ("complement", "∁"),
    ("curlyvee", "⋎"),
    ("curlywedge", "⋏"),
    ("curvearrowleft", "↶"),
    ("curvearrowright", "↷"),
    ("daleth", "ℸ"),
    ("dashleftarrow", "⇠"),
    ("dashrightarrow", "⇢"),
    ("dashv", "⊣"),
    ("diamond", "⋄"),
    ("diamondsuit", "♢"),
    ("digamma", "ϝ"),
    ("divideontimes", "⋇"),
    ("doteq", "≐"),
    ("dotminus", "∸"),
    ("dotplus", "∔"),
    ("dotsb", "⋯"),
    ("dotsc", "…"),
    ("dotsi", "⋯"),
    ("dotsm", "⋯"),
    ("dotso", "…"),
    ("doublebarwedge", "⩞"),
    ("doublecap", "⋒"),
    ("doublecup", "⋓"),
    ("downdownarrows", "⇊"),
    ("downharpoonleft", "⇃"),
    ("downharpoonright", "⇂"),
    ("empty", "∅"),
    ("emsp", "\u{2003}"),
    ("enskip", "\u{2002}"),
    ("enspace", "\u{2002}"),
    ("eqcirc", "≖"),
    ("eqcolon", "≕"),
    ("eqqsim", "⩳"),
    ("eqsim", "≂"),
    ("eqslantgtr", "⪖"),
    ("eqslantless", "⪕"),
    ("eth", "ð"),
    ("fallingdotseq", "≒"),
    ("flat", "♭"),
    ("frown", "⌢"),
    ("geqq", "≧"),
    ("geqslant", "⩾"),
    ("ggg", "⋙"),
    ("gggtr", "⋙"),
    ("gimel", "ℷ"),
    ("gneq", "⪈"),
    ("gneqq", "≩"),
    ("gtrapprox", "⪆"),
    ("gtrdot", "⋗"),
    ("gtreqless", "⋛"),
    ("gtreqqless", "⪌"),
    ("gtrless", "≷"),
    ("gtrsim", "≳"),
    ("heartsuit", "♡"),
    ("hookleftarrow", "↩"),
    ("hookrightarrow", "↪"),
    ("imath", "𝚤"),
    ("intercal", "⊺"),
    ("jmath", "𝚥"),
    ("leadsto", "⇝"),
    ("leftarrowtail", "↢"),
    ("leftharpoondown", "↽"),
    ("leftharpoonup", "↼"),
    ("leftrightharpoons", "⇋"),
    ("leftrightsquigarrow", "↭"),
    ("leftsquigarrow", "⇜"),
    ("leftthreetimes", "⋋"),
    ("leqq", "≦"),
    ("leqslant", "⩽"),
    ("lessapprox", "⪅"),
    ("lessdot", "⋖"),
    ("lesseqgtr", "⋚"),
    ("lesseqqgtr", "⪋"),
    ("lessgtr", "≶"),
    ("lesssim", "≲"),
    ("lgroup", "⟮"),
    ("lhd", "⊲"),
    ("lll", "⋘"),
    ("llless", "⋘"),
    ("lmoustache", "⎰"),
    ("lneq", "⪇"),
    ("lneqq", "≨"),
    ("longleftrightarrow", "⟷"),
    ("longmapsfrom", "⟻"),
    ("longmapsto", "⟼"),
    ("looparrowleft", "↫"),
    ("looparrowright", "↬"),
    ("lozenge", "◊"),
    ("ltimes", "⋉"),
    ("maltese", "✠"),
    ("mapsfrom", "↤"),
    ("measuredangle", "∡"),
    ("medspace", "\u{205F}"),
    ("mho", "℧"),
    ("multimap", "⊸"),
    ("nLeftarrow", "⇍"),
    ("nLeftrightarrow", "⇎"),
    ("nRightarrow", "⇏"),
    ("nVDash", "⊯"),
    ("nVdash", "⊮"),
    ("napprox", "≉"),
    ("nasymp", "≭"),
    ("natural", "♮"),
    ("ncong", "≇"),
    ("nearrow", "↗"),
    ("ngeq", "≱"),
    ("ngtr", "≯"),
    ("nleftarrow", "↚"),
    ("nleftrightarrow", "↮"),
    ("nleq", "≰"),
    ("nless", "≮"),
    ("nparallel", "∦"),
    ("nprec", "⊀"),
    ("npreceq", "⋠"),
    ("nrightarrow", "↛"),
    ("nshortmid", "∤"),
    ("nshortparallel", "∦"),
    ("nsim", "≁"),
    ("nsqsubseteq", "⋢"),
    ("nsqsupseteq", "⋣"),
    ("nsubset", "⊄"),
    ("nsubseteq", "⊈"),
    ("nsucc", "⊁"),
    ("nsucceq", "⋡"),
    ("nsupset", "⊅"),
    ("nsupseteq", "⊉"),
    ("ntriangleleft", "⋪"),
    ("ntrianglelefteq", "⋬"),
    ("ntriangleright", "⋫"),
    ("ntrianglerighteq", "⋭"),
    ("nvDash", "⊭"),
    ("nvdash", "⊬"),
    ("nwarrow", "↖"),
    ("pitchfork", "⋔"),
    ("preccurlyeq", "≼"),
    ("precnapprox", "⪹"),
    ("precnsim", "⋨"),
    ("precsim", "≾"),
    ("rgroup", "⟯"),
    ("rhd", "⊳"),
    ("rightarrowtail", "↣"),
    ("rightharpoondown", "⇁"),
    ("rightharpoonup", "⇀"),
    ("rightleftharpoons", "⇌"),
    ("rightsquigarrow", "⇝"),
    ("rightthreetimes", "⋌"),
    ("risingdotseq", "≓"),
    ("rmoustache", "⎱"),
    ("rtimes", "⋊"),
    ("searrow", "↘"),
    ("sharp", "♯"),
    ("shortmid", "∣"),
    ("shortparallel", "∥"),
    ("smallfrown", "⌢"),
    ("smallsetminus", "∖"),
    ("smallsmile", "⌣"),
    ("smile", "⌣"),
    ("spadesuit", "♠"),
    ("sphericalangle", "∢"),
    ("sqcap", "⊓"),
    ("sqcup", "⊔"),
    ("sqsubset", "⊏"),
    ("sqsubseteq", "⊑"),
    ("sqsupset", "⊐"),
    ("sqsupseteq", "⊒"),
    ("square", "□"),
    ("subseteqq", "⫅"),
    ("subsetneq", "⊊"),
    ("subsetneqq", "⫋"),
    ("succcurlyeq", "≽"),
    ("succnapprox", "⪺"),
    ("succnsim", "⋩"),
    ("succsim", "≿"),
    ("supseteqq", "⫆"),
    ("supsetneq", "⊋"),
    ("supsetneqq", "⫌"),
    ("surd", "√"),
    ("swarrow", "↙"),
    ("thickspace", "\u{2005}"),
    ("thinspace", "\u{2009}"),
    ("top", "⊤"),
    ("triangledown", "▿"),
    ("triangleleft", "◁"),
    ("trianglelefteq", "⊴"),
    ("triangleq", "≜"),
    ("triangleright", "▷"),
    ("trianglerighteq", "⊵"),
    ("twoheadleftarrow", "↞"),
    ("twoheadmapsto", "⤅"),
    ("twoheadrightarrow", "↠"),
    ("unlhd", "⊴"),
    ("unrhd", "⊵"),
    ("updownarrow", "↕"),
    ("upharpoonleft", "↿"),
    ("upharpoonright", "↾"),
    ("uplus", "⊎"),
    ("upuparrows", "⇈"),
    ("vDash", "⊨"),
    ("varpropto", "∝"),
    ("varsubsetneq", "⊊"),
    ("varsubsetneqq", "⫋"),
    ("varsupsetneq", "⊋"),
    ("varsupsetneqq", "⫌"),
    ("vartriangle", "▵"),
    ("vartriangleleft", "⊲"),
    ("vartriangleright", "⊳"),
    ("veebar", "⊻"),
    ("wedgebar", "⩟"),
    ("wr", "≀"),
];

/// The command the reader prints for an n-ary operator.
pub(super) fn nary_command(chr: char) -> Option<&'static str> {
    NARY.iter().find(|(_, c)| *c == chr).map(|(name, _)| *name)
}

/// The command the reader prints for an accent character.
pub(super) fn accent_command(chr: char) -> Option<&'static str> {
    ACCENTS
        .iter()
        .find(|(_, c)| *c == chr)
        .map(|(name, _)| *name)
}

/// How a run of math text is set (`m:rPr`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Style {
    /// Word's default: letters italic, the rest upright.
    Math,
    /// `\mathrm`, and function names (`m:sty p`).
    Upright,
    Bold,
    BoldItalic,
    /// `\text`: ordinary text (`m:nor`).
    Text,
    /// A script (`m:scr`): double-struck, script, fraktur, sans-serif or
    /// monospace.
    Script(&'static str),
}

/// One piece of an equation.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    /// A run: its text, style and the change inside the math it is in.
    Text(String, Style, Option<Kind>),
    /// A `{…}` group: its nodes, written inline; a script's base when one
    /// follows.
    Group(Vec<Node>),
    Fraction {
        num: Vec<Node>,
        den: Vec<Node>,
        bar: bool,
    },
    Radical {
        degree: Vec<Node>,
        body: Vec<Node>,
    },
    Scripts {
        base: Vec<Node>,
        sub: Option<Vec<Node>>,
        sup: Option<Vec<Node>>,
    },
    Pre {
        sub: Vec<Node>,
        sup: Vec<Node>,
        body: Vec<Node>,
    },
    Delimiter {
        open: String,
        close: String,
        separator: String,
        items: Vec<Vec<Node>>,
    },
    Function {
        name: Vec<Node>,
        body: Vec<Node>,
    },
    Nary {
        chr: char,
        sub: Vec<Node>,
        sup: Vec<Node>,
        body: Vec<Node>,
    },
    Accent {
        chr: char,
        body: Vec<Node>,
    },
    Bar {
        top: bool,
        body: Vec<Node>,
    },
    Limit {
        upper: bool,
        base: Vec<Node>,
        limit: Vec<Node>,
    },
    Brace {
        top: bool,
        body: Vec<Node>,
    },
    Matrix(Vec<Vec<Vec<Node>>>),
    Array(Vec<Vec<Node>>),
    /// `\mathord{…}`: Word's box (`m:box`), an operator's when `operator`
    /// (`\mathop`).
    Boxed {
        operator: bool,
        body: Vec<Node>,
    },
}

/// A LaTeX token.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Token {
    Char(char),
    /// `\name`, or `\` and one other character (`\{`, `\\`, `\,`).
    Command(String),
    Open,
    Close,
    Sup,
    Sub,
    Amp,
    /// A change inside the math starts (its kind) or ends.
    Change(Option<Kind>),
}

fn tokens(latex: &str) -> Vec<Token> {
    let chars: Vec<char> = latex.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        i += 1;
        out.push(match c {
            '\\' => match chars.get(i) {
                Some(n) if n.is_ascii_alphabetic() => {
                    let start = i;
                    while chars.get(i).is_some_and(char::is_ascii_alphabetic) {
                        i += 1;
                    }
                    let name: String = chars[start..i].iter().collect();
                    // A starred name (`\operatorname*`) is the same command.
                    if chars.get(i) == Some(&'*') {
                        i += 1;
                    }
                    Token::Command(name)
                }
                Some(&n) => {
                    i += 1;
                    Token::Command(n.to_string())
                }
                None => Token::Char('\\'),
            },
            '{' => Token::Open,
            '}' => Token::Close,
            '^' => Token::Sup,
            '_' => Token::Sub,
            '&' => Token::Amp,
            INSERT_START => Token::Change(Some(Kind::Insert)),
            DELETE_START => Token::Change(Some(Kind::Delete)),
            INSERT_END | DELETE_END => Token::Change(None),
            c => Token::Char(c),
        });
    }
    out
}

/// The opener's closer, for the brackets that build a delimiter bare.
fn closer(token: &Token) -> Option<Token> {
    match token {
        Token::Char('(') => Some(Token::Char(')')),
        Token::Char('[') => Some(Token::Char(']')),
        Token::Command(c) if c == "{" => Some(Token::Command("}".into())),
        _ => None,
    }
}

fn is_command(token: &Token, name: &str) -> bool {
    matches!(token, Token::Command(c) if c == name)
}

/// Each bare bracket's matching closer, by position: brackets pair inside
/// one `{…}` group, one `\left…\right` and one matrix cell, so `(a]` and
/// `{(}` leave their brackets as text.
fn pairs(tokens: &[Token]) -> Vec<Option<usize>> {
    enum Frame {
        Bracket(usize),
        Wall,
    }
    let mut out = vec![None; tokens.len()];
    let mut stack: Vec<Frame> = Vec::new();
    let unwind = |stack: &mut Vec<Frame>| {
        while let Some(Frame::Bracket(_)) = stack.last() {
            stack.pop();
        }
    };
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Open => stack.push(Frame::Wall),
            Token::Close => {
                unwind(&mut stack);
                stack.pop();
            }
            Token::Command(c) if c == "left" || c == "begin" => stack.push(Frame::Wall),
            Token::Command(c) if c == "right" || c == "end" => {
                unwind(&mut stack);
                stack.pop();
            }
            Token::Amp => unwind(&mut stack),
            Token::Command(c) if c == "\\" || c == "middle" => unwind(&mut stack),
            _ if closer(token).is_some() => {
                // `\left(` and `\sqrt[` take their bracket themselves.
                let taken = i > 0
                    && (is_command(&tokens[i - 1], "left")
                        || is_command(&tokens[i - 1], "right")
                        || is_command(&tokens[i - 1], "middle"));
                if !taken {
                    stack.push(Frame::Bracket(i));
                }
            }
            _ => {
                if let Some(Frame::Bracket(open)) = stack.last()
                    && closer(&tokens[*open]).as_ref() == Some(token)
                {
                    out[*open] = Some(i);
                    stack.pop();
                }
            }
        }
    }
    out
}

/// Where a list of nodes ends.
#[derive(Clone, Copy, Default)]
struct Stops {
    /// At a `}`.
    group: bool,
    /// At this position: a bare bracket's closer.
    at: Option<usize>,
    /// At `\right` or `\middle`.
    right: bool,
    /// At `\\` (rows) and `\end`.
    rows: bool,
    /// At `&` (matrix cells).
    cells: bool,
}

struct Parser {
    tokens: Vec<Token>,
    pairs: Vec<Option<usize>>,
    at: usize,
    /// The change the text read now is in.
    change: Option<Kind>,
    /// How deep the atom read now nests.
    depth: usize,
}

/// Deeper than any equation Word builds. Past it a token reads as text, so
/// untrusted Markdown cannot nest the parser (and the tree the writer walks)
/// off the end of the stack.
const MOST_NESTING: usize = 64;

/// A token as the LaTeX it was read from.
fn token_latex(token: &Token) -> String {
    match token {
        Token::Char(c) => c.to_string(),
        Token::Command(name) => format!("\\{name}"),
        Token::Open => "{".into(),
        Token::Close => "}".into(),
        Token::Sup => "^".into(),
        Token::Sub => "_".into(),
        Token::Amp => "&".into(),
        Token::Change(_) => String::new(),
    }
}

impl Parser {
    fn new(latex: &str) -> Self {
        let tokens = tokens(latex);
        let pairs = pairs(&tokens);
        Self {
            tokens,
            pairs,
            at: 0,
            change: None,
            depth: 0,
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    /// A run of `text` in `style`, in the change open now.
    fn text(&self, text: &str, style: Style) -> Node {
        Node::Text(text.into(), style, self.change)
    }

    fn stopped(&self, stops: Stops) -> bool {
        let Some(token) = self.peek() else {
            return true;
        };
        stops.at == Some(self.at)
            || (stops.group && *token == Token::Close)
            || (stops.right && (is_command(token, "right") || is_command(token, "middle")))
            || (stops.rows && (is_command(token, "\\") || is_command(token, "end")))
            || (stops.cells && *token == Token::Amp)
    }

    fn skip_spaces(&mut self) {
        while matches!(self.peek(), Some(Token::Char(c)) if c.is_whitespace()) {
            self.at += 1;
        }
    }

    /// Nodes up to a stop, which is left unread.
    fn list(&mut self, stops: Stops) -> Vec<Node> {
        let mut out: Vec<Node> = Vec::new();
        while !self.stopped(stops) {
            match self.peek() {
                Some(Token::Change(change)) => {
                    self.change = *change;
                    self.at += 1;
                }
                Some(Token::Sub | Token::Sup) => {
                    let base = take_base(&mut out);
                    let node = self.scripts(base);
                    out.push(node);
                }
                // `f'` is `f` with a prime superscript, as Word builds it;
                // `f'^2` puts the 2 after the prime.
                Some(Token::Char('\'')) if !out.is_empty() => {
                    let base = take_base(&mut out);
                    let mut count = 0;
                    while self.peek() == Some(&Token::Char('\'')) {
                        self.at += 1;
                        count += 1;
                    }
                    let mut sup = vec![self.text(&primes(count), Style::Math)];
                    if self.peek() == Some(&Token::Sup) {
                        self.at += 1;
                        sup.extend(self.argument());
                    }
                    let sub = if self.peek() == Some(&Token::Sub) {
                        self.at += 1;
                        Some(self.argument())
                    } else {
                        None
                    };
                    out.push(Node::Scripts {
                        base,
                        sub,
                        sup: Some(merge(sup)),
                    });
                }
                _ => {
                    if let Some(node) = self.atom(stops) {
                        out.push(node);
                    }
                }
            }
        }
        merge(out)
    }

    /// The nodes of a `{…}` group whose `{` is next.
    fn group(&mut self) -> Vec<Node> {
        self.at += 1;
        let nodes = self.list(Stops {
            group: true,
            ..Stops::default()
        });
        if self.peek() == Some(&Token::Close) {
            self.at += 1;
        }
        nodes
    }

    /// One argument: a `{…}` group, a command's node or one character.
    fn argument(&mut self) -> Vec<Node> {
        self.skip_spaces();
        while let Some(Token::Change(change)) = self.peek() {
            self.change = *change;
            self.at += 1;
        }
        match self.peek() {
            Some(Token::Open) => self.group(),
            None | Some(Token::Close | Token::Amp | Token::Sub | Token::Sup) => Vec::new(),
            Some(_) => self.atom(Stops::default()).into_iter().collect(),
        }
    }

    /// An argument's raw text: `\text{a b}` keeps its spaces.
    fn raw_argument(&mut self) -> String {
        self.skip_spaces();
        if self.peek() != Some(&Token::Open) {
            return self.argument().iter().map(plain).collect();
        }
        self.at += 1;
        let mut depth = 0usize;
        let mut out = String::new();
        while let Some(token) = self.peek().cloned() {
            self.at += 1;
            match token {
                Token::Open => {
                    depth += 1;
                    out.push('{');
                }
                Token::Close if depth == 0 => break,
                Token::Close => {
                    depth -= 1;
                    out.push('}');
                }
                Token::Char(c) => out.push(c),
                Token::Sup => out.push('^'),
                Token::Sub => out.push('_'),
                Token::Amp => out.push('&'),
                Token::Command(name) => {
                    out.push_str(&escaped(&name));
                    if name.starts_with(|c: char| c.is_ascii_alphabetic()) {
                        self.skip_spaces();
                    }
                }
                Token::Change(change) => self.change = change,
            }
        }
        out
    }

    /// An optional `[…]` argument whose `[` is next and paired.
    fn optional(&mut self) -> Option<Vec<Node>> {
        self.skip_spaces();
        if self.peek() != Some(&Token::Char('[')) {
            return None;
        }
        let end = self.pairs[self.at]?;
        self.at += 1;
        let nodes = self.list(Stops {
            at: Some(end),
            ..Stops::default()
        });
        self.at = end + 1;
        Some(nodes)
    }

    /// The scripts after `base`.
    fn scripts(&mut self, base: Vec<Node>) -> Node {
        let (mut sub, mut sup) = (None, None);
        loop {
            match self.peek() {
                Some(Token::Sub) if sub.is_none() => {
                    self.at += 1;
                    sub = Some(self.argument());
                }
                Some(Token::Sup) if sup.is_none() => {
                    self.at += 1;
                    sup = Some(self.argument());
                }
                _ => break,
            }
        }
        Node::Scripts { base, sub, sup }
    }

    /// The node an operator applies to: the next atom and its scripts.
    fn operand(&mut self, stops: Stops) -> Vec<Node> {
        self.skip_spaces();
        if self.stopped(stops) || matches!(self.peek(), Some(Token::Sub | Token::Sup)) {
            return Vec::new();
        }
        if self.peek() == Some(&Token::Open) {
            let nodes = self.group();
            return if matches!(self.peek(), Some(Token::Sub | Token::Sup)) {
                vec![self.scripts(vec![Node::Group(nodes)])]
            } else {
                nodes
            };
        }
        let Some(node) = self.atom(stops) else {
            return Vec::new();
        };
        if matches!(self.peek(), Some(Token::Sub | Token::Sup)) {
            vec![self.scripts(vec![node])]
        } else {
            vec![node]
        }
    }

    /// The next node, from the token at the cursor.
    /// One atom: every nesting construct is read through here, so this
    /// bounds the depth ([`MOST_NESTING`]).
    fn atom(&mut self, stops: Stops) -> Option<Node> {
        if self.depth >= MOST_NESTING {
            let token = self.peek()?.clone();
            self.at += 1;
            return Some(self.text(&token_latex(&token), Style::Math));
        }
        self.depth += 1;
        let node = self.nested_atom(stops);
        self.depth -= 1;
        node
    }

    fn nested_atom(&mut self, stops: Stops) -> Option<Node> {
        let token = self.peek()?.clone();
        let start = self.at;
        self.at += 1;
        match token {
            Token::Open => {
                self.at = start;
                Some(Node::Group(self.group()))
            }
            Token::Close => Some(self.text("}", Style::Math)),
            Token::Amp => Some(self.text("&", Style::Math)),
            Token::Sub | Token::Sup => None,
            Token::Change(change) => {
                self.change = change;
                None
            }
            Token::Char(c) => {
                if let Some(end) = self.pairs[start] {
                    let close = if c == '(' { ")" } else { "]" };
                    return Some(self.bracketed(&c.to_string(), close, end, stops));
                }
                Some(self.text(&c.to_string(), Style::Math))
            }
            Token::Command(name) => self.command(&name, start, stops),
        }
    }

    /// A delimiter whose opener was read and whose closer is at `end`.
    fn bracketed(&mut self, open: &str, close: &str, end: usize, stops: Stops) -> Node {
        let body = self.list(Stops {
            at: Some(end),
            ..stops
        });
        self.at = end + 1;
        Node::Delimiter {
            open: open.into(),
            close: close.into(),
            separator: "|".into(),
            items: vec![body],
        }
    }

    fn command(&mut self, name: &str, start: usize, stops: Stops) -> Option<Node> {
        let change = self.change;
        let text = |s: &str| Some(Node::Text(s.into(), Style::Math, change));
        // Spaces after a control word end it.
        let word = name.starts_with(|c: char| c.is_ascii_alphabetic());
        let spaced = word && matches!(self.peek(), Some(Token::Char(c)) if c.is_whitespace());
        if word {
            self.skip_spaces();
        }
        match name {
            "{" => {
                if let Some(end) = self.pairs[start] {
                    return Some(self.bracketed("{", "}", end, stops));
                }
                text("{")
            }
            "frac" | "dfrac" | "tfrac" | "cfrac" => {
                let num = self.argument();
                let den = self.argument();
                Some(Node::Fraction {
                    num,
                    den,
                    bar: true,
                })
            }
            "binom" | "dbinom" | "tbinom" => {
                let num = self.argument();
                let den = self.argument();
                Some(delimiter(
                    "(",
                    ")",
                    vec![Node::Fraction {
                        num,
                        den,
                        bar: false,
                    }],
                ))
            }
            "genfrac" => {
                let open = self.raw_argument();
                let close = self.raw_argument();
                let thickness = self.raw_argument();
                let _style = self.raw_argument();
                let num = self.argument();
                let den = self.argument();
                let fraction = Node::Fraction {
                    num,
                    den,
                    bar: !thickness.trim().starts_with('0'),
                };
                if open.is_empty() && close.is_empty() {
                    Some(fraction)
                } else {
                    Some(delimiter(&open, &close, vec![fraction]))
                }
            }
            "sqrt" => {
                let degree = self.optional().unwrap_or_default();
                let body = self.argument();
                Some(Node::Radical { degree, body })
            }
            "left" => Some(self.left()),
            "prescript" => {
                let sub = self.argument();
                let sup = self.argument();
                let body = self.argument();
                Some(Node::Pre { sub, sup, body })
            }
            "overline" | "underline" => Some(Node::Bar {
                top: name == "overline",
                body: self.argument(),
            }),
            "overbrace" | "underbrace" => {
                let top = name == "overbrace";
                let brace = Node::Brace {
                    top,
                    body: self.argument(),
                };
                let script = if top { Token::Sup } else { Token::Sub };
                if self.peek() == Some(&script) {
                    self.at += 1;
                    let limit = self.argument();
                    return Some(Node::Limit {
                        upper: top,
                        base: vec![brace],
                        limit,
                    });
                }
                Some(brace)
            }
            "underset" | "overset" | "stackrel" => {
                let limit = self.argument();
                let base = self.argument();
                Some(Node::Limit {
                    upper: name != "underset",
                    base,
                    limit,
                })
            }
            "operatorname" => {
                let word = self.raw_argument();
                Some(self.function(&word, true, stops))
            }
            "begin" => Some(self.environment()),
            "text" | "textrm" | "textup" | "mbox" | "textnormal" => {
                let raw = self.raw_argument();
                Some(self.text(&raw, Style::Text))
            }
            "mathord" | "mathop" => Some(Node::Boxed {
                operator: name == "mathop",
                body: self.argument(),
            }),
            "mathrm" | "mathup" | "rm" => Some(styled(self.argument(), Style::Upright)),
            "mathbf" | "bf" => Some(styled(self.argument(), Style::Bold)),
            "boldsymbol" | "bm" | "mathbfit" => Some(styled(self.argument(), Style::BoldItalic)),
            "mathit" => Some(styled(self.argument(), Style::Math)),
            "mathbb" => Some(styled(self.argument(), Style::Script("double-struck"))),
            "mathcal" | "mathscr" => Some(styled(self.argument(), Style::Script("script"))),
            "mathfrak" => Some(styled(self.argument(), Style::Script("fraktur"))),
            "mathsf" => Some(styled(self.argument(), Style::Script("sans-serif"))),
            "mathtt" => Some(styled(self.argument(), Style::Script("monospace"))),
            "displaystyle" | "textstyle" | "scriptstyle" | "limits" | "nolimits" | "!" => None,
            // A size for the bracket after it: Word's delimiter grows to its
            // content, so the bracket alone builds it.
            "big" | "bigl" | "bigr" | "bigm" | "Big" | "Bigl" | "Bigr" | "Bigm" | "bigg"
            | "biggl" | "biggr" | "biggm" | "Bigg" | "Biggl" | "Biggr" | "Biggm" => None,
            "not" => match self.peek().cloned() {
                Some(Token::Char(c)) if !c.is_whitespace() => {
                    self.at += 1;
                    text(&negated(&c.to_string()))
                }
                Some(Token::Command(next)) => match SYMBOLS.iter().find(|(n, _)| *n == next) {
                    Some((_, symbol)) => {
                        self.at += 1;
                        if next.starts_with(|c: char| c.is_ascii_alphabetic()) {
                            self.skip_spaces();
                        }
                        text(&negated(symbol))
                    }
                    None => text(&escaped(name)),
                },
                _ => text(&escaped(name)),
            },
            "bmod" => Some(Node::Group(vec![
                Node::Text("mod".into(), Style::Upright, change),
                Node::Text(" ".into(), Style::Math, change),
            ])),
            "pmod" => {
                let mut body = vec![
                    Node::Text("mod".into(), Style::Upright, change),
                    Node::Text(" ".into(), Style::Math, change),
                ];
                body.extend(self.argument());
                Some(delimiter("(", ")", merge(body)))
            }
            "hspace" | "hskip" | "kern" | "mspace" => {
                let _ = self.raw_argument();
                text(" ")
            }
            "," | ":" | ">" => text("\u{2009}"),
            ";" => text("\u{2005}"),
            " " => text(" "),
            "\\" | "end" | "right" | "middle" => text(&format!("\\{name}")),
            _ => {
                if FUNCTIONS.contains(&name) {
                    return Some(self.function(name, false, stops));
                }
                if let Some(&(_, chr)) = NARY.iter().find(|(n, _)| *n == name) {
                    return Some(self.nary(chr, stops));
                }
                if let Some(&(_, chr)) = ACCENTS.iter().find(|(n, _)| *n == name) {
                    return Some(Node::Accent {
                        chr,
                        body: self.argument(),
                    });
                }
                if let Some((_, symbol)) = SYMBOLS.iter().find(|(n, _)| *n == name) {
                    return text(symbol);
                }
                // An unknown command keeps its text and the space after it,
                // and an argument right after it its braces.
                if !spaced && self.peek() == Some(&Token::Open) {
                    let mut nodes = vec![Node::Text(
                        format!("{}{{", escaped(name)),
                        Style::Math,
                        change,
                    )];
                    nodes.extend(self.group());
                    nodes.push(Node::Text("}".into(), Style::Math, self.change));
                    return Some(Node::Group(nodes));
                }
                let space = if spaced { " " } else { "" };
                text(&format!("{}{space}", escaped(name)))
            }
        }
    }

    /// `\left` read: the delimiter up to its `\right`, split at `\middle`.
    fn left(&mut self) -> Node {
        let open = self.fence();
        let mut items = Vec::new();
        let mut separator = String::from("|");
        loop {
            items.push(trimmed(self.list(Stops {
                right: true,
                ..Stops::default()
            })));
            match self.peek() {
                Some(token) if is_command(token, "middle") => {
                    self.at += 1;
                    separator = self.fence();
                }
                Some(token) if is_command(token, "right") => {
                    self.at += 1;
                    let close = self.fence();
                    return Node::Delimiter {
                        open,
                        close,
                        separator,
                        items,
                    };
                }
                _ => {
                    return Node::Delimiter {
                        open,
                        close: String::new(),
                        separator,
                        items,
                    };
                }
            }
        }
    }

    /// The character after `\left`, `\middle` or `\right`; `.` is none.
    fn fence(&mut self) -> String {
        self.skip_spaces();
        let Some(token) = self.peek().cloned() else {
            return String::new();
        };
        self.at += 1;
        match token {
            Token::Char('.') => String::new(),
            Token::Char(c) => c.to_string(),
            Token::Command(name) => match name.as_str() {
                "{" | "lbrace" => "{".into(),
                "}" | "rbrace" => "}".into(),
                "|" => "‖".into(),
                name => SYMBOLS
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map_or_else(|| escaped(name), |(_, s)| (*s).to_string()),
            },
            _ => String::new(),
        }
    }

    /// A function by its name, read: its scripts, then its argument.
    fn function(&mut self, word: &str, operator: bool, stops: Stops) -> Node {
        let mut name = vec![self.text(word, Style::Upright)];
        if matches!(self.peek(), Some(Token::Sub | Token::Sup)) {
            let limits = operator || LIMIT_FUNCTIONS.contains(&word);
            name = vec![if limits {
                let upper = self.peek() == Some(&Token::Sup);
                self.at += 1;
                let limit = self.argument();
                Node::Limit {
                    upper,
                    base: name,
                    limit,
                }
            } else {
                self.scripts(name)
            }];
        }
        let body = self.operand(stops);
        Node::Function { name, body }
    }

    fn nary(&mut self, chr: char, stops: Stops) -> Node {
        let (mut sub, mut sup) = (Vec::new(), Vec::new());
        loop {
            match self.peek() {
                Some(Token::Sub) => {
                    self.at += 1;
                    sub = self.argument();
                }
                Some(Token::Sup) => {
                    self.at += 1;
                    sup = self.argument();
                }
                Some(token) if is_command(token, "limits") || is_command(token, "nolimits") => {
                    self.at += 1;
                }
                _ => break,
            }
        }
        let body = self.operand(stops);
        Node::Nary {
            chr,
            sub,
            sup,
            body,
        }
    }

    /// `\begin` read: a matrix or an equation array up to its `\end`.
    fn environment(&mut self) -> Node {
        let name = self.raw_argument();
        // `array`'s column spec (`{cc}`) aligns nothing Word keeps. Without
        // one, the first cell starts here.
        self.skip_spaces();
        if name == "array" && self.peek() == Some(&Token::Open) {
            let _ = self.raw_argument();
        }
        let matrix = name.ends_with("matrix") || name == "array";
        let mut rows: Vec<Vec<Vec<Node>>> = vec![Vec::new()];
        loop {
            let cell = trimmed(self.list(Stops {
                rows: true,
                cells: matrix,
                ..Stops::default()
            }));
            if let Some(row) = rows.last_mut() {
                row.push(cell);
            }
            match self.peek() {
                Some(Token::Amp) => self.at += 1,
                Some(token) if is_command(token, "\\") => {
                    self.at += 1;
                    rows.push(Vec::new());
                }
                Some(token) if is_command(token, "end") => {
                    self.at += 1;
                    let _ = self.raw_argument();
                    break;
                }
                _ => break,
            }
        }
        if matrix {
            let node = Node::Matrix(rows);
            let fences = match name.as_str() {
                "pmatrix" => Some(("(", ")")),
                "bmatrix" => Some(("[", "]")),
                "Bmatrix" => Some(("{", "}")),
                "vmatrix" => Some(("|", "|")),
                "Vmatrix" => Some(("‖", "‖")),
                _ => None,
            };
            return match fences {
                Some((open, close)) => delimiter(open, close, vec![node]),
                None => node,
            };
        }
        let lines = rows
            .into_iter()
            .map(|row| trimmed(row.into_iter().flatten().collect()))
            .collect();
        let node = Node::Array(lines);
        if name == "cases" {
            delimiter("{", "", vec![node])
        } else {
            node
        }
    }
}

/// `count` primes as one character: ′, ″, ‴, ⁗, then ′ repeated.
pub(super) fn primes(count: usize) -> String {
    match count {
        1 => "′".into(),
        2 => "″".into(),
        3 => "‴".into(),
        4 => "⁗".into(),
        n => "′".repeat(n),
    }
}

/// How many primes `text` is, if it is only primes.
pub(super) fn prime_count(text: &str) -> Option<usize> {
    let mut count = 0;
    for c in text.chars() {
        count += match c {
            '′' => 1,
            '″' => 2,
            '‴' => 3,
            '⁗' => 4,
            _ => return None,
        };
    }
    (count > 0).then_some(count)
}

/// `relation` negated: its own negated character, or the combining long
/// solidus (U+0338) Word overlays.
fn negated(relation: &str) -> String {
    const NEGATED: [(&str, &str); 24] = [
        ("=", "≠"),
        ("<", "≮"),
        (">", "≯"),
        ("∈", "∉"),
        ("∋", "∌"),
        ("⊂", "⊄"),
        ("⊃", "⊅"),
        ("⊆", "⊈"),
        ("⊇", "⊉"),
        ("≡", "≢"),
        ("∼", "≁"),
        ("≤", "≰"),
        ("≥", "≱"),
        ("≈", "≉"),
        ("≅", "≇"),
        ("≃", "≄"),
        ("∣", "∤"),
        ("∥", "∦"),
        ("≺", "⊀"),
        ("≻", "⊁"),
        ("⊑", "⋢"),
        ("⊒", "⋣"),
        ("∃", "∄"),
        ("≍", "≭"),
    ];
    NEGATED
        .iter()
        .find(|(from, _)| *from == relation)
        .map_or_else(|| format!("{relation}\u{338}"), |(_, to)| (*to).to_string())
}

fn delimiter(open: &str, close: &str, body: Vec<Node>) -> Node {
    Node::Delimiter {
        open: open.into(),
        close: close.into(),
        separator: "|".into(),
        items: vec![body],
    }
}

/// The text a command names when it is not one: `\{` is `{`, `\foo` stays.
fn escaped(name: &str) -> String {
    match name {
        "{" | "}" | "_" | "&" | "$" | "%" | "#" | "^" => name.into(),
        "|" => "‖".into(),
        _ => SYMBOLS
            .iter()
            .find(|(n, _)| *n == name)
            .map_or_else(|| format!("\\{name}"), |(_, s)| (*s).to_string()),
    }
}

/// The text of a node, for an argument read as text.
fn plain(node: &Node) -> String {
    match node {
        Node::Text(text, ..) => text.clone(),
        Node::Group(nodes) => nodes.iter().map(plain).collect(),
        _ => String::new(),
    }
}

/// `nodes` with every run set in `style`.
fn styled(nodes: Vec<Node>, style: Style) -> Node {
    fn restyle(node: Node, style: Style) -> Node {
        match node {
            Node::Text(text, _, change) => Node::Text(text, style, change),
            Node::Group(nodes) => {
                Node::Group(nodes.into_iter().map(|n| restyle(n, style)).collect())
            }
            other => other,
        }
    }
    Node::Group(nodes.into_iter().map(|n| restyle(n, style)).collect())
}

/// The base a script takes from the nodes before it: the last node, or the
/// last character of a run of text.
fn take_base(out: &mut Vec<Node>) -> Vec<Node> {
    match out.pop() {
        None => Vec::new(),
        Some(Node::Group(nodes)) => nodes,
        Some(Node::Text(text, style, change)) => {
            let mut chars: Vec<char> = text.chars().collect();
            let last = chars.pop().map(String::from).unwrap_or_default();
            if !chars.is_empty() {
                out.push(Node::Text(chars.into_iter().collect(), style, change));
            }
            vec![Node::Text(last, style, change)]
        }
        Some(node) => vec![node],
    }
}

/// `nodes` without the spaces at their edges: the reader pads matrix
/// cells, array lines and equations (`a & b \\ c`), and a person writes
/// `\left( a \right)`; spaces inside stay, as Word keeps them.
fn trimmed(mut nodes: Vec<Node>) -> Vec<Node> {
    if let Some(Node::Text(text, Style::Math, None)) = nodes.first_mut() {
        *text = text.trim_start().to_string();
    }
    if let Some(Node::Text(text, Style::Math, None)) = nodes.last_mut() {
        *text = text.trim_end().to_string();
    }
    nodes.retain(|node| !matches!(node, Node::Text(text, ..) if text.is_empty()));
    nodes
}

/// Neighbouring runs of one style as one run, and groups that are no
/// script's base written inline.
fn merge(nodes: Vec<Node>) -> Vec<Node> {
    let mut out: Vec<Node> = Vec::new();
    for node in nodes {
        let nodes = match node {
            Node::Group(inner) => inner,
            other => vec![other],
        };
        for node in nodes {
            if let (
                Some(Node::Text(last, last_style, last_change)),
                Node::Text(text, style, change),
            ) = (out.last_mut(), &node)
                && *last_style == *style
                && *last_change == *change
            {
                last.push_str(text);
                continue;
            }
            out.push(node);
        }
    }
    out
}

/// The equation `latex` is. Lines a top-level `\\` breaks become one
/// equation array (`m:eqArr`): Word draws the equations of one paragraph
/// side by side, so lines are an array's.
fn equation(latex: &str) -> Vec<Node> {
    let mut parser = Parser::new(latex);
    let mut lines: Vec<Vec<Node>> = vec![Vec::new()];
    loop {
        let nodes = parser.list(Stops {
            rows: true,
            ..Stops::default()
        });
        if let Some(line) = lines.last_mut() {
            line.extend(nodes);
        }
        match parser.peek() {
            Some(token) if is_command(token, "\\") => {
                parser.at += 1;
                lines.push(Vec::new());
            }
            // A stray `\end` is text.
            Some(_) => {
                parser.at += 1;
                if let Some(line) = lines.last_mut() {
                    line.push(Node::Text("\\end".into(), Style::Math, None));
                }
            }
            None => break,
        }
    }
    let mut lines: Vec<Vec<Node>> = lines.into_iter().map(|line| trimmed(merge(line))).collect();
    match lines.len() {
        1 => lines.pop().unwrap_or_default(),
        _ => vec![Node::Array(lines)],
    }
}

/// Writes nodes as OMML, marking each run and control with a tracked
/// change when `mark` gives one.
struct Writer<'m> {
    xml: String,
    /// The change the whole math is in.
    change: Option<Kind>,
    mark: &'m mut dyn FnMut(Kind) -> String,
}

impl Writer<'_> {
    /// `inner` inside a fresh `w:ins` or `w:del` when `change` is one.
    fn marked(&mut self, inner: &str, change: Option<Kind>) {
        match change {
            Some(kind) => {
                let tag = (self.mark)(kind);
                let name = tag.split_whitespace().next().unwrap_or("w:ins").to_string();
                let _ = write!(self.xml, "<{tag}>{inner}</{name}>");
            }
            None => self.xml.push_str(inner),
        }
    }

    fn control(&mut self) {
        self.xml.push_str("<m:ctrlPr>");
        self.marked(FONT, self.change);
        self.xml.push_str("</m:ctrlPr>");
    }

    /// A structure's properties: `properties`, then the control.
    fn properties(&mut self, name: &str, properties: &str) {
        let _ = write!(self.xml, "<m:{name}>{properties}");
        self.control();
        let _ = write!(self.xml, "</m:{name}>");
    }

    fn run(&mut self, text: &str, style: Style, change: Option<Kind>) {
        let properties = match style {
            Style::Math => String::new(),
            Style::Upright => "<m:rPr><m:sty m:val=\"p\"/></m:rPr>".into(),
            Style::Bold => "<m:rPr><m:sty m:val=\"b\"/></m:rPr>".into(),
            Style::BoldItalic => "<m:rPr><m:sty m:val=\"bi\"/></m:rPr>".into(),
            Style::Text => "<m:rPr><m:nor/></m:rPr>".into(),
            Style::Script(script) => format!("<m:rPr><m:scr m:val=\"{script}\"/></m:rPr>"),
        };
        let space = if text.starts_with(char::is_whitespace) || text.ends_with(char::is_whitespace)
        {
            " xml:space=\"preserve\""
        } else {
            ""
        };
        self.xml.push_str("<m:r>");
        self.marked(
            &format!("{properties}{FONT}<m:t{space}>{}</m:t>", escape(text)),
            change.or(self.change),
        );
        self.xml.push_str("</m:r>");
    }

    /// `<m:name>nodes</m:name>`.
    fn argument(&mut self, name: &str, nodes: &[Node]) {
        let _ = write!(self.xml, "<m:{name}>");
        self.nodes(nodes);
        let _ = write!(self.xml, "</m:{name}>");
    }

    fn nodes(&mut self, nodes: &[Node]) {
        for node in nodes {
            self.node(node);
        }
    }

    fn node(&mut self, node: &Node) {
        match node {
            Node::Text(text, style, change) => self.run(text, *style, *change),
            Node::Group(nodes) => self.nodes(nodes),
            Node::Fraction { num, den, bar } => {
                self.xml.push_str("<m:f>");
                let kind = if *bar {
                    ""
                } else {
                    "<m:type m:val=\"noBar\"/>"
                };
                self.properties("fPr", kind);
                self.argument("num", num);
                self.argument("den", den);
                self.xml.push_str("</m:f>");
            }
            Node::Radical { degree, body } => {
                self.xml.push_str("<m:rad>");
                let hide = if degree.is_empty() {
                    "<m:degHide m:val=\"1\"/>"
                } else {
                    ""
                };
                self.properties("radPr", hide);
                self.argument("deg", degree);
                self.argument("e", body);
                self.xml.push_str("</m:rad>");
            }
            Node::Scripts { base, sub, sup } => {
                let name = match (sub, sup) {
                    (Some(_), Some(_)) => "sSubSup",
                    (Some(_), None) => "sSub",
                    _ => "sSup",
                };
                let _ = write!(self.xml, "<m:{name}>");
                self.properties(&format!("{name}Pr"), "");
                self.argument("e", base);
                if let Some(sub) = sub {
                    self.argument("sub", sub);
                }
                if let Some(sup) = sup {
                    self.argument("sup", sup);
                } else if sub.is_none() {
                    self.argument("sup", &[]);
                }
                let _ = write!(self.xml, "</m:{name}>");
            }
            Node::Pre { sub, sup, body } => {
                self.xml.push_str("<m:sPre>");
                self.properties("sPrePr", "");
                self.argument("sub", sub);
                self.argument("sup", sup);
                self.argument("e", body);
                self.xml.push_str("</m:sPre>");
            }
            Node::Delimiter {
                open,
                close,
                separator,
                items,
            } => {
                self.xml.push_str("<m:d>");
                let mut properties = String::new();
                if open != "(" {
                    let _ = write!(properties, "<m:begChr m:val=\"{}\"/>", escape(open));
                }
                if separator != "|" {
                    let _ = write!(properties, "<m:sepChr m:val=\"{}\"/>", escape(separator));
                }
                if close != ")" {
                    let _ = write!(properties, "<m:endChr m:val=\"{}\"/>", escape(close));
                }
                self.properties("dPr", &properties);
                for item in items {
                    self.argument("e", item);
                }
                self.xml.push_str("</m:d>");
            }
            Node::Function { name, body } => {
                self.xml.push_str("<m:func>");
                self.properties("funcPr", "");
                self.argument("fName", name);
                self.argument("e", body);
                self.xml.push_str("</m:func>");
            }
            Node::Nary {
                chr,
                sub,
                sup,
                body,
            } => {
                self.xml.push_str("<m:nary>");
                let integral = matches!(chr, '\u{222B}'..='\u{2233}' | '\u{2A0B}'..='\u{2A1C}');
                let mut properties = format!(
                    "<m:chr m:val=\"{chr}\"/><m:limLoc m:val=\"{}\"/>",
                    if integral { "subSup" } else { "undOvr" }
                );
                if sub.is_empty() {
                    properties.push_str("<m:subHide m:val=\"1\"/>");
                }
                if sup.is_empty() {
                    properties.push_str("<m:supHide m:val=\"1\"/>");
                }
                self.properties("naryPr", &properties);
                self.argument("sub", sub);
                self.argument("sup", sup);
                self.argument("e", body);
                self.xml.push_str("</m:nary>");
            }
            Node::Accent { chr, body } => {
                self.xml.push_str("<m:acc>");
                self.properties("accPr", &format!("<m:chr m:val=\"{chr}\"/>"));
                self.argument("e", body);
                self.xml.push_str("</m:acc>");
            }
            Node::Bar { top, body } => {
                self.xml.push_str("<m:bar>");
                let pos = if *top { "top" } else { "bot" };
                self.properties("barPr", &format!("<m:pos m:val=\"{pos}\"/>"));
                self.argument("e", body);
                self.xml.push_str("</m:bar>");
            }
            Node::Limit { upper, base, limit } => {
                let name = if *upper { "limUpp" } else { "limLow" };
                let _ = write!(self.xml, "<m:{name}>");
                self.properties(&format!("{name}Pr"), "");
                self.argument("e", base);
                self.argument("lim", limit);
                let _ = write!(self.xml, "</m:{name}>");
            }
            Node::Brace { top, body } => {
                self.xml.push_str("<m:groupChr>");
                let properties = if *top {
                    "<m:chr m:val=\"⏞\"/><m:pos m:val=\"top\"/><m:vertJc m:val=\"bot\"/>"
                } else {
                    "<m:chr m:val=\"⏟\"/><m:pos m:val=\"bot\"/><m:vertJc m:val=\"top\"/>"
                };
                self.properties("groupChrPr", properties);
                self.argument("e", body);
                self.xml.push_str("</m:groupChr>");
            }
            Node::Matrix(rows) => {
                let columns = rows.iter().map(Vec::len).max().unwrap_or(1).max(1);
                self.xml.push_str("<m:m>");
                self.properties(
                    "mPr",
                    &format!(
                        "<m:mcs><m:mc><m:mcPr><m:count m:val=\"{columns}\"/>\
                         <m:mcJc m:val=\"center\"/></m:mcPr></m:mc></m:mcs>"
                    ),
                );
                for row in rows {
                    self.xml.push_str("<m:mr>");
                    for at in 0..columns {
                        self.argument("e", row.get(at).map_or(&[][..], Vec::as_slice));
                    }
                    self.xml.push_str("</m:mr>");
                }
                self.xml.push_str("</m:m>");
            }
            Node::Boxed { operator, body } => {
                self.xml.push_str("<m:box>");
                let emulate = if *operator {
                    "<m:opEmu m:val=\"1\"/>"
                } else {
                    ""
                };
                self.properties("boxPr", emulate);
                self.argument("e", body);
                self.xml.push_str("</m:box>");
            }
            Node::Array(lines) => {
                self.xml.push_str("<m:eqArr>");
                self.properties("eqArrPr", "");
                for line in lines {
                    self.argument("e", line);
                }
                self.xml.push_str("</m:eqArr>");
            }
        }
    }
}

/// `latex` as Office Math: display math as an `m:oMath` in an
/// `m:oMathPara`, inline math as an `m:oMath`. `change` is the
/// change the whole math is in; [`INSERT_START`] and its kin mark changes
/// inside it. `mark` gives the start tag (`w:ins w:id=…`) of a fresh mark
/// of a change of its kind: Word marks each run and control inside the
/// math, never the math whole.
pub(super) fn omml(
    latex: &str,
    display: bool,
    change: Option<Kind>,
    mark: &mut dyn FnMut(Kind) -> String,
) -> String {
    let mut writer = Writer {
        xml: String::new(),
        change,
        mark,
    };
    let namespace = format!(" xmlns:m=\"{M_NS}\"");
    if display {
        let _ = write!(writer.xml, "<m:oMathPara{namespace}>");
    }
    let _ = write!(
        writer.xml,
        "<m:oMath{}>",
        if display { "" } else { namespace.as_str() }
    );
    writer.nodes(&equation(latex));
    writer.xml.push_str("</m:oMath>");
    if display {
        writer.xml.push_str("</m:oMathPara>");
    }
    writer.xml
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The math's element names and text, without properties or fonts.
    fn shape(latex: &str) -> String {
        let xml = omml(latex, false, None, &mut |_| String::new());
        let mut out = String::new();
        let mut rest = xml.as_str();
        let mut skip = 0usize;
        while let Some(open) = rest.find('<') {
            let text = &rest[..open];
            if skip == 0 && !text.is_empty() {
                out.push_str(text);
            }
            let close = rest[open..].find('>').map_or(rest.len(), |i| open + i);
            let tag = &rest[open + 1..close];
            rest = &rest[close + 1..];
            let name = tag
                .trim_start_matches('/')
                .split([' ', '/'])
                .next()
                .unwrap_or("");
            let hidden = name.ends_with("Pr") || name.starts_with("w:");
            let closing = tag.starts_with('/');
            let empty = tag.ends_with('/');
            if hidden {
                if !empty {
                    if closing {
                        skip = skip.saturating_sub(1);
                    } else {
                        skip += 1;
                    }
                }
                continue;
            }
            if skip > 0 || name == "m:t" || name == "m:oMath" {
                continue;
            }
            let short = name.trim_start_matches("m:");
            match (closing, empty) {
                (true, _) => out.push(')'),
                (false, true) => {
                    let _ = write!(out, "{short}()");
                }
                (false, false) => {
                    let _ = write!(out, "{short}(");
                }
            }
        }
        out
    }

    #[test]
    fn ams_and_unicode_math_names_are_symbols() {
        assert_eq!(shape(r"a\leqslant b"), "r(a⩽b)");
        assert_eq!(shape(r"A\lhd B\twoheadrightarrow C"), "r(A⊲B↠C)");
        assert_eq!(shape(r"\Box\blacksquare\lozenge"), "r(□■◊)");
        assert_eq!(shape(r"a\thinspace b"), "r(a\u{2009}b)");
        assert_eq!(shape(r"1+\dotsb+n"), "r(1+⋯+n)");
    }

    #[test]
    fn more_nary_operators_and_their_limits() {
        assert_eq!(shape(r"\iiiint_a^b f"), "nary(sub(r(a))sup(r(b))e(r(f)))");
        for (latex, chr, limits) in [
            (r"\iiiint_a^b f", '⨌', "subSup"),
            (r"\fint_a^b f", '⨏', "subSup"),
            (r"\ointctrclockwise_C f", '∳', "subSup"),
            (r"\bigsqcap_i A", '⨅', "undOvr"),
            (r"\bigtimes_i A", '⨉', "undOvr"),
        ] {
            let xml = omml(latex, false, None, &mut |_| String::new());
            assert!(
                xml.contains(&format!("<m:chr m:val=\"{chr}\"/>")),
                "{latex}: {xml}"
            );
            assert!(
                xml.contains(&format!("<m:limLoc m:val=\"{limits}\"/>")),
                "{latex}: {xml}"
            );
        }
        assert_eq!(nary_command('⨌'), Some("iiiint"));
    }

    #[test]
    fn no_command_is_defined_twice() {
        let mut names: Vec<&str> = SYMBOLS.iter().map(|(n, _)| *n).collect();
        names.extend(NARY.iter().map(|(n, _)| *n));
        names.extend(ACCENTS.iter().map(|(n, _)| *n));
        names.extend(FUNCTIONS);
        names.sort_unstable();
        let twice: Vec<_> = names.windows(2).filter(|w| w[0] == w[1]).collect();
        assert!(twice.is_empty(), "{twice:?}");
    }

    #[test]
    fn scripts_take_the_last_character_as_base() {
        assert_eq!(shape("A=πr^2"), "r(A=π)sSup(e(r(r))sup(r(2)))");
        assert_eq!(shape("x_i^2"), "sSubSup(e(r(x))sub(r(i))sup(r(2)))");
        assert_eq!(shape("{(x+a)}^n"), "sSup(e(d(e(r(x+a))))sup(r(n)))");
    }

    #[test]
    fn balanced_brackets_build_a_delimiter_and_strays_stay_text() {
        assert_eq!(shape("f(x)"), "r(f)d(e(r(x)))");
        assert_eq!(shape("(a]"), "r((a])");
        assert_eq!(shape(r"\{a\}"), "d(e(r(a)))");
        assert_eq!(shape(r"\lparen a\rparen"), "r((a))");
    }

    #[test]
    fn structures_come_back_as_word_writes_them() {
        assert_eq!(
            shape(r"\frac{-b±\sqrt{b^2-4ac}}{2a}"),
            "f(num(r(-b±)rad(deg()e(sSup(e(r(b))sup(r(2)))r(-4ac))))den(r(2a)))"
        );
        assert_eq!(
            shape(r"\sum_{k=0}^n \binom{n}{k}"),
            "nary(sub(r(k=0))sup(r(n))e(d(e(f(num(r(n))den(r(k)))))))"
        );
        assert_eq!(
            shape(r"\lim_{n→∞}{a}"),
            "func(fName(limLow(e(r(lim))lim(r(n→∞))))e(r(a)))"
        );
        assert_eq!(
            shape(r"\sin^2{x}"),
            "func(fName(sSup(e(r(sin))sup(r(2))))e(r(x)))"
        );
        assert_eq!(shape(r"\left[ a \middle| b \right)"), "d(e(r(a))e(r(b)))");
        assert_eq!(
            shape(r"\begin{matrix}1 & 0 \\ 0 & 1\end{matrix}"),
            "m(mr(e(r(1))e(r(0)))mr(e(r(0))e(r(1))))"
        );
        assert_eq!(
            shape(r"\underbrace{x+y}_{n}"),
            "limLow(e(groupChr(e(r(x+y))))lim(r(n)))"
        );
        assert_eq!(
            shape(r"\prescript{a}{b}{X}"),
            "sPre(sub(r(a))sup(r(b))e(r(X)))"
        );
    }

    #[test]
    fn an_nary_takes_one_operand_with_its_scripts() {
        assert_eq!(
            shape(r"\sum_{i=1}^n x_i+1"),
            "nary(sub(r(i=1))sup(r(n))e(sSub(e(r(x))sub(r(i)))))r(+1)"
        );
    }

    #[test]
    fn an_unknown_command_stays_as_text() {
        assert_eq!(shape(r"\foo x"), r"r(\foo x)");
        assert_eq!(shape(r"\alpha+\beta"), "r(α+β)");
        // Its argument keeps its braces: `\foo{x}` is not `\foox`.
        assert_eq!(shape(r"\foo{x}"), r"r(\foo{x})");
    }

    #[test]
    fn primes_are_a_superscript_as_word_builds_them() {
        assert_eq!(shape("f'(x)"), "sSup(e(r(f))sup(r(′)))d(e(r(x)))");
        assert_eq!(shape("f''"), "sSup(e(r(f))sup(r(″)))");
        assert_eq!(shape("f'''"), "sSup(e(r(f))sup(r(‴)))");
        assert_eq!(shape("f'^2"), "sSup(e(r(f))sup(r(′2)))");
        assert_eq!(shape("'a"), "r('a)");
    }

    #[test]
    fn not_negates_the_next_relation() {
        assert_eq!(shape(r"a \not\in B"), shape(r"a \notin B"));
        assert_eq!(shape(r"a \not= b"), shape("a ≠ b"));
        assert_eq!(shape(r"\not\subset"), "r(⊄)");
        assert_eq!(shape(r"\not\perp"), "r(⊥\u{338})");
    }

    #[test]
    fn sized_delimiters_are_delimiters() {
        assert_eq!(shape(r"\bigl( x \bigr)"), shape("( x )"));
        assert_eq!(shape(r"\Bigl[ y \Bigr]"), shape("[ y ]"));
        assert_eq!(shape(r"\biggl\{ z \biggr\}"), shape(r"\{ z \}"));
    }

    #[test]
    fn nesting_past_the_limit_reads_as_text_instead_of_overflowing() {
        // Markdown is untrusted: 50,000 nested groups must not take the
        // stack down with the process.
        let deep = format!("{}x{}", "{".repeat(50_000), "}".repeat(50_000));
        let _ = shape(&deep);
        let fracs = format!("{}x{}", r"\frac{".repeat(20_000), "}{y}".repeat(20_000));
        let _ = shape(&fracs);
    }

    #[test]
    fn an_array_is_a_matrix_without_its_column_spec() {
        assert_eq!(
            shape(r"\begin{array}{cc} a & b \\ c & d \end{array}"),
            shape(r"\begin{matrix} a & b \\ c & d \end{matrix}")
        );
        // Without a spec, the first cell is not taken for one.
        assert_eq!(
            shape(r"\begin{array} a & b \\ c & d\end{array}"),
            shape(r"\begin{matrix} a & b \\ c & d \end{matrix}")
        );
    }

    #[test]
    fn modulo_and_spacing_commands() {
        assert_eq!(shape(r"a \bmod n"), "r(a )r(mod)r( n)");
        assert_eq!(shape(r"a \pmod{n}"), "r(a )d(e(r(mod)r( n)))");
        assert_eq!(shape(r"x\hspace{1em}y"), "r(x y)");
    }

    #[test]
    fn a_change_marks_every_run_and_control_inside() {
        let mut next = 0;
        let xml = omml("x^2", true, Some(Kind::Insert), &mut |_| {
            next += 1;
            format!("w:ins w:id=\"{next}\" w:author=\"A\"")
        });
        assert!(xml.starts_with("<m:oMathPara xmlns:m="), "{xml}");
        assert_eq!(xml.matches("<w:ins ").count(), 3, "{xml}");
        assert!(xml.contains("<m:ctrlPr><w:ins w:id=\"1\""), "{xml}");
        assert!(!xml.contains("w:delText"), "{xml}");
    }

    #[test]
    fn a_change_inside_marks_only_its_runs() {
        let latex = format!("A={DELETE_START}π{DELETE_END}{INSERT_START}2{INSERT_END}r^2");
        let xml = omml(&latex, false, None, &mut |kind| match kind {
            Kind::Insert => "w:ins w:id=\"1\"".into(),
            Kind::Delete => "w:del w:id=\"2\"".into(),
        });
        assert!(xml.contains("<m:r><w:del w:id=\"2\">"), "{xml}");
        assert!(xml.contains("<m:t>π</m:t></w:del></m:r>"), "{xml}");
        assert!(xml.contains("<m:t>2</m:t></w:ins></m:r>"), "{xml}");
        assert_eq!(xml.matches("<w:ins ").count(), 1, "{xml}");
        assert_eq!(xml.matches("<w:del ").count(), 1, "{xml}");
    }

    #[test]
    fn display_lines_are_an_equation_array() {
        assert_eq!(shape(r"a \\ b"), "eqArr(e(r(a))e(r(b)))");
        let xml = omml(r"a \\ b", true, None, &mut |_| String::new());
        assert_eq!(xml.matches("<m:oMath>").count(), 1, "{xml}");
    }
}
