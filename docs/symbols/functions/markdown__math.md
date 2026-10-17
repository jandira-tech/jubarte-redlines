# Functions and methods in `src/markdown/math.rs`

50 non-test, 19 test. Part of the [symbol manifest](../README.md).

#### `nary_command`  (L527-529, pub(super))
`pub(super) fn nary_command(chr: char) -> Option<&'static str>`

- **Does:** The command the reader prints for an n-ary operator.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:3946); `tests::more_nary_operators_and_their_limits` (markdown/math.rs:2014)

#### `accent_command`  (L532-537, pub(super))
`pub(super) fn accent_command(chr: char) -> Option<&'static str>`

- **Does:** The command the reader prints for an accent character.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map`
- **Called by:** none found in src

#### `tokens`  (L640-679)
`fn tokens(latex: &str) -> Vec<Token>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.get`, `.is_ascii_alphabetic`, `.is_some_and`, `.iter`, `.len`, `.push`, `.to_string`, `Some`, `Token::Change`, `Token::Char`, `Token::Command`, `Vec::new`
- **Called by:** `reflow_body` (markdown/diff.rs:547) (?); `words` (markdown/diff.rs:870) (?); `words_diff` (markdown/diff.rs:911) (?); `Parser::new` (markdown/math.rs:792)

#### `closer`  (L682-689)
`fn closer(token: &Token) -> Option<Token>`

- **Does:** The opener's closer, for the brackets that build a delimiter bare.
- **Calls (std/external/unresolved):** `.into`, `Some`, `Token::Char`, `Token::Command`
- **Called by:** `pairs` (markdown/math.rs:698)

#### `is_command`  (L691-693)
`fn is_command(token: &Token, name: &str) -> bool`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `Token::Command`
- **Macros:** `matches!`
- **Called by:** `pairs` (markdown/math.rs:698); `Parser::stopped` (markdown/math.rs:813); `Parser::left` (markdown/math.rs:1294); `Parser::nary` (markdown/math.rs:1375); `Parser::environment` (markdown/math.rs:1403); `equation` (markdown/math.rs:1630)

#### `pairs`  (L698-745)
`fn pairs(tokens: &[Token]) -> Vec<Option<usize>>`

- **Does:** Each bare bracket's matching closer, by position: brackets pair inside one `{…}` group, one `\left…\right` and one matrix cell, so `(a]` and `{(}` leave their brackets as text.
- **Calls (crate):** `closer` (markdown/math.rs:682) L724,736; `is_command` (markdown/math.rs:691) L727,728,729
- **Calls (std/external/unresolved):** `.as_ref`, `.enumerate`, `.is_some`, `.iter`, `.last`, `.len`, `.pop`, `.push`, `Frame::Bracket`, `Some`, `Vec::new`, `unwind`
- **Macros:** `vec!`
- **Called by:** `Parser::new` (markdown/math.rs:792)

#### `token_latex`  (L778-789)
`fn token_latex(token: &Token) -> String`

- **Does:** A token as the LaTeX it was read from.
- **Calls (std/external/unresolved):** `.into`, `.to_string`, `String::new`
- **Macros:** `format!`
- **Called by:** `Parser::flat` (markdown/math.rs:845); `Parser::atom` (markdown/math.rs:1042)

#### `Parser::new`  (L792-802)
`fn new(latex: &str) -> Self`

- **Does:** _no doc comment_
- **Calls (crate):** `tokens` (markdown/math.rs:640) L793; `pairs` (markdown/math.rs:698) L794
- **Called by:** `equation` (markdown/math.rs:1630)

#### `Parser::peek`  (L804-806)
`fn peek(&self) -> Option<&Token>`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.get`
- **Called by:** `body_looks_like_section_number` (comparer/finalize.rs:1971) (?); `drawings_follow_page_break` (convert/mod.rs:9505) (?); `code_words` (fields.rs:452) (?); `unescape_markdown` (markdown/anchor.rs:50) (?); `plain_anchor` (markdown/anchor.rs:251) (?); `Writer::list_marker` (markdown/from_docx/mod.rs:2898) (?); `field_words` (markdown/from_docx/mod.rs:4491) (?); `Parser::stopped` (markdown/math.rs:813); `Parser::skip_spaces` (markdown/math.rs:824) (?); `Parser::flat` (markdown/math.rs:845); `Parser::nested_list` (markdown/math.rs:867); `Parser::group` (markdown/math.rs:917); +16 more

#### `Parser::text`  (L809-811)
`fn text(&self, text: &str, style: Style) -> Node`

- **Does:** A run of `text` in `style`, in the change open now.
- **Calls (std/external/unresolved):** `.into`, `Node::Text`
- **Called by:** `Parser::atom` (markdown/math.rs:1042); `Parser::nested_atom` (markdown/math.rs:1054); `Parser::command` (markdown/math.rs:1096)

#### `Parser::stopped`  (L813-822)
`fn stopped(&self, stops: Stops) -> bool`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) L814; `is_command` (markdown/math.rs:691) L819,820
- **Calls (std/external/unresolved):** `Some`
- **Called by:** `Parser::flat` (markdown/math.rs:845); `Parser::nested_list` (markdown/math.rs:867); `Parser::operand` (markdown/math.rs:1016)

#### `Parser::skip_spaces`  (L824-828)
`fn skip_spaces(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L825
- **Calls (std/external/unresolved):** `.is_whitespace`, `Some`, `Token::Char`
- **Macros:** `matches!`
- **Called by:** `Parser::argument` (markdown/math.rs:930); `Parser::raw_argument` (markdown/math.rs:944); `Parser::optional` (markdown/math.rs:981); `Parser::operand` (markdown/math.rs:1016); `Parser::command` (markdown/math.rs:1096); `Parser::fence` (markdown/math.rs:1331); `Parser::environment` (markdown/math.rs:1403)

#### `Parser::list`  (L833-841)
`fn list(&mut self, stops: Stops) -> Vec<Node>`

- **Does:** Nodes up to a stop, which is left unread.
- **Calls (crate):** `Parser::flat` (markdown/math.rs:845) L835; `Parser::nested_list` (markdown/math.rs:867) L838
- **Called by:** `Parser::group` (markdown/math.rs:917); `Parser::optional` (markdown/math.rs:981); `Parser::bracketed` (markdown/math.rs:1082); `Parser::left` (markdown/math.rs:1294); `Parser::environment` (markdown/math.rs:1403); `equation` (markdown/math.rs:1630)

#### `Parser::flat`  (L845-865)
`fn flat(&mut self, stops: Stops) -> Vec<Node>`

- **Does:** The tokens up to a stop as one text, past [`MOST_NESTING`]: a `{…}` read here keeps its `}`, so it does not end the group around it.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) L848; `Parser::stopped` (markdown/math.rs:813) L849; `token_latex` (markdown/math.rs:778) L857; `.text` (ambiguous, 10 candidates) L863
- **Calls (std/external/unresolved):** `.is_empty`, `.push_str`, `.saturating_sub`, `String::new`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Parser::list` (markdown/math.rs:833)

#### `Parser::nested_list`  (L867-914)
`fn nested_list(&mut self, stops: Stops) -> Vec<Node>`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::stopped` (markdown/math.rs:813) L869; `Parser::peek` (markdown/math.rs:804) L870,885,890,894; `take_base` (markdown/math.rs:1571) L876,883; `Parser::scripts` (markdown/math.rs:997) L877; `.text` (ambiguous, 10 candidates) L889; `primes` (markdown/math.rs:1465) L889; `Parser::argument` (markdown/math.rs:930) L892,896; `merge` (markdown/math.rs:1603) L903,913; `Parser::atom` (markdown/math.rs:1042) L907
- **Calls (std/external/unresolved):** `.extend`, `.is_empty`, `.push`, `Some`, `Token::Char`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Parser::list` (markdown/math.rs:833)

#### `Parser::group`  (L917-927)
`fn group(&mut self) -> Vec<Node>`

- **Does:** The nodes of a `{…}` group whose `{` is next.
- **Calls (crate):** `Parser::list` (markdown/math.rs:833) L919; `Parser::peek` (markdown/math.rs:804) L923
- **Calls (std/external/unresolved):** `Some`, `Stops::default`
- **Called by:** `Parser::argument` (markdown/math.rs:930); `Parser::operand` (markdown/math.rs:1016); `Parser::nested_atom` (markdown/math.rs:1054); `Parser::command` (markdown/math.rs:1096)

#### `Parser::argument`  (L930-941)
`fn argument(&mut self) -> Vec<Node>`

- **Does:** One argument: a `{…}` group, a command's node or one character.
- **Calls (crate):** `Parser::skip_spaces` (markdown/math.rs:824) L931; `Parser::peek` (markdown/math.rs:804) L932,936; `Parser::group` (markdown/math.rs:917) L937; `Parser::atom` (markdown/math.rs:1042) L939
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `Stops::default`, `Vec::new`
- **Called by:** `Parser::nested_list` (markdown/math.rs:867); `Parser::raw_argument` (markdown/math.rs:944); `Parser::scripts` (markdown/math.rs:997); `Parser::command` (markdown/math.rs:1096); `Parser::function` (markdown/math.rs:1354) (?); `Parser::nary` (markdown/math.rs:1375)

#### `Parser::raw_argument`  (L944-978)
`fn raw_argument(&mut self) -> String`

- **Does:** An argument's raw text: `\text{a b}` keeps its spaces.
- **Calls (crate):** `Parser::skip_spaces` (markdown/math.rs:824) L945,971; `Parser::peek` (markdown/math.rs:804) L946,952; `Parser::argument` (markdown/math.rs:930) L947; `escaped` (markdown/math.rs:1535) L969
- **Calls (std/external/unresolved):** `.cloned`, `.collect`, `.is_ascii_alphabetic`, `.iter`, `.map`, `.push_str`, `.push`, `.starts_with`, `Some`, `String::new`
- **Called by:** `Parser::command` (markdown/math.rs:1096); `Parser::environment` (markdown/math.rs:1403)

#### `Parser::optional`  (L981-994)
`fn optional(&mut self) -> Option<Vec<Node>>`

- **Does:** An optional `[…]` argument whose `[` is next and paired.
- **Calls (crate):** `Parser::skip_spaces` (markdown/math.rs:824) L982; `Parser::peek` (markdown/math.rs:804) L983; `Parser::list` (markdown/math.rs:833) L988
- **Calls (std/external/unresolved):** `Some`, `Stops::default`, `Token::Char`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `Parser::scripts`  (L997-1013)
`fn scripts(&mut self, base: Vec<Node>) -> Node`

- **Does:** The scripts after `base`.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) L1000; `Parser::argument` (markdown/math.rs:930) L1003,1007
- **Calls (std/external/unresolved):** `.is_none`, `Some`
- **Called by:** `Parser::nested_list` (markdown/math.rs:867); `Parser::operand` (markdown/math.rs:1016) (?); `Parser::function` (markdown/math.rs:1354) (?)

#### `Parser::operand`  (L1016-1037)
`fn operand(&mut self, stops: Stops) -> Vec<Node>`

- **Does:** The node an operator applies to: the next atom and its scripts.
- **Calls (crate):** `Parser::skip_spaces` (markdown/math.rs:824) L1017; `Parser::stopped` (markdown/math.rs:813) L1018; `Parser::peek` (markdown/math.rs:804) (by name) L1018,1023,1032; `Parser::peek` (markdown/math.rs:804) L1021; `Parser::group` (markdown/math.rs:917) L1022; `Parser::scripts` (markdown/math.rs:997) (by name) L1024,1033; `Parser::atom` (markdown/math.rs:1042) L1029
- **Calls (std/external/unresolved):** `Node::Group`, `Some`, `Vec::new`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Parser::function` (markdown/math.rs:1354); `Parser::nary` (markdown/math.rs:1375)

#### `Parser::atom`  (L1042-1052)
`fn atom(&mut self, stops: Stops) -> Option<Node>`

- **Does:** The next node, from the token at the cursor.
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) L1044; `Parser::text` (markdown/math.rs:809) L1046; `token_latex` (markdown/math.rs:778) L1046; `Parser::nested_atom` (markdown/math.rs:1054) L1049
- **Calls (std/external/unresolved):** `.clone`, `Some`
- **Called by:** `Parser::nested_list` (markdown/math.rs:867); `Parser::argument` (markdown/math.rs:930); `Parser::operand` (markdown/math.rs:1016)

#### `Parser::nested_atom`  (L1054-1079)
`fn nested_atom(&mut self, stops: Stops) -> Option<Node>`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) L1055; `Parser::group` (markdown/math.rs:917) L1061; `Parser::text` (markdown/math.rs:809) L1063,1064,1075; `Parser::bracketed` (markdown/math.rs:1082) L1073; `Parser::command` (markdown/math.rs:1096) L1077
- **Calls (std/external/unresolved):** `.clone`, `.to_string`, `Node::Group`, `Some`
- **Called by:** `Parser::atom` (markdown/math.rs:1042)

#### `Parser::bracketed`  (L1082-1094)
`fn bracketed(&mut self, open: &str, close: &str, end: usize, stops: Stops) -> Node`

- **Does:** A delimiter whose opener was read and whose closer is at `end`.
- **Calls (crate):** `Parser::list` (markdown/math.rs:833) L1083
- **Calls (std/external/unresolved):** `.into`, `Some`
- **Macros:** `vec!`
- **Called by:** `Parser::nested_atom` (markdown/math.rs:1054); `Parser::command` (markdown/math.rs:1096)

#### `Parser::command`  (L1096-1291)
`fn command(&mut self, name: &str, start: usize, stops: Stops) -> Option<Node>`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) (by name) L1101; `Parser::skip_spaces` (markdown/math.rs:824) L1103,1231; `Parser::bracketed` (markdown/math.rs:1082) L1108; `text` (ambiguous, 13 candidates) L1110,1225,1233,1235,1237,1253...; `Parser::argument` (markdown/math.rs:930) L1113,1114,1122,1123,1139,1140...; `delimiter` (markdown/math.rs:1525) L1124,1149,1249; `Parser::raw_argument` (markdown/math.rs:944) L1135,1136,1137,1138,1196,1201...; `Parser::optional` (markdown/math.rs:981) L1153; `Parser::left` (markdown/math.rs:1294) L1157; `Parser::peek` (markdown/math.rs:804) L1175,1222,1277; `Parser::function` (markdown/math.rs:1354) L1197,1261; `Parser::environment` (markdown/math.rs:1403) L1199; `Parser::text` (markdown/math.rs:809) L1202; `styled` (markdown/math.rs:1556) L1208,1209,1210,1211,1212,1213...; `negated` (markdown/math.rs:1492) L1225,1233; `escaped` (markdown/math.rs:1535) L1235,1237,1279,1288; `merge` (markdown/math.rs:1603) L1249; `Parser::nary` (markdown/math.rs:1375) L1264; `Parser::group` (markdown/math.rs:917) L1283
- **Calls (std/external/unresolved):** `.cloned`, `.contains`, `.extend`, `.find`, `.into`, `.is_ascii_alphabetic`, `.is_empty`, `.is_whitespace`, `.iter`, `.push`, `.starts_with`, `.to_string`, `.trim`, `.unwrap_or_default`, `Node::Group`, `Node::Text`, `Some`, `Style::Script`, `Token::Char`
- **Macros:** `format!`, `matches!`, `vec!`
- **Called by:** `Parser::nested_atom` (markdown/math.rs:1054)

#### `Parser::left`  (L1294-1328)
`fn left(&mut self) -> Node`

- **Does:** `\left` read: the delimiter up to its `\right`, split at `\middle`.
- **Calls (crate):** `Parser::fence` (markdown/math.rs:1331) L1295,1306,1310; `trimmed` (markdown/math.rs:1590) L1299; `Parser::list` (markdown/math.rs:833) L1299; `Parser::peek` (markdown/math.rs:804) L1303; `is_command` (markdown/math.rs:691) L1304,1308
- **Calls (std/external/unresolved):** `.push`, `Stops::default`, `String::from`, `String::new`, `Vec::new`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `Parser::fence`  (L1331-1351)
`fn fence(&mut self) -> String`

- **Does:** The character after `\left`, `\middle` or `\right`; `.` is none.
- **Calls (crate):** `Parser::skip_spaces` (markdown/math.rs:824) L1332; `Parser::peek` (markdown/math.rs:804) L1333; `escaped` (markdown/math.rs:1535) L1347
- **Calls (std/external/unresolved):** `.as_str`, `.cloned`, `.find`, `.into`, `.iter`, `.map_or_else`, `.to_string`, `String::new`
- **Called by:** `Parser::left` (markdown/math.rs:1294)

#### `Parser::function`  (L1354-1373)
`fn function(&mut self, word: &str, operator: bool, stops: Stops) -> Node`

- **Does:** A function by its name, read: its scripts, then its argument.
- **Calls (crate):** `.text` (ambiguous, 10 candidates) L1355; `Parser::peek` (markdown/math.rs:804) (by name) L1356,1359; `.argument` (ambiguous: `Parser::argument` (markdown/math.rs:930); `Writer::argument` (markdown/math.rs:1722)) L1361; `Parser::scripts` (markdown/math.rs:997) (by name) L1368; `Parser::operand` (markdown/math.rs:1016) L1371
- **Calls (std/external/unresolved):** `.contains`, `Some`
- **Macros:** `matches!`, `vec!`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `Parser::nary`  (L1375-1400)
`fn nary(&mut self, chr: char, stops: Stops) -> Node`

- **Does:** _no doc comment_
- **Calls (crate):** `Parser::peek` (markdown/math.rs:804) L1378; `Parser::argument` (markdown/math.rs:930) L1381,1385; `is_command` (markdown/math.rs:691) L1387; `Parser::operand` (markdown/math.rs:1016) L1393
- **Calls (std/external/unresolved):** `Vec::new`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `Parser::environment`  (L1403-1461)
`fn environment(&mut self) -> Node`

- **Does:** `\begin` read: a matrix or an equation array up to its `\end`.
- **Calls (crate):** `Parser::raw_argument` (markdown/math.rs:944) L1404,1409,1430; `Parser::skip_spaces` (markdown/math.rs:824) L1407; `Parser::peek` (markdown/math.rs:804) L1408,1422; `trimmed` (markdown/math.rs:1590) L1414,1453; `Parser::list` (markdown/math.rs:833) L1414; `is_command` (markdown/math.rs:691) L1424,1428; `delimiter` (markdown/math.rs:1525) L1447,1457
- **Calls (std/external/unresolved):** `.as_str`, `.collect`, `.ends_with`, `.flatten`, `.into_iter`, `.last_mut`, `.map`, `.push`, `Node::Array`, `Node::Matrix`, `Some`, `Stops::default`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `primes`  (L1465-1473, pub(super))
`pub(super) fn primes(count: usize) -> String`

- **Does:** `count` primes as one character: ′, ″, ‴, ⁗, then ′ repeated.
- **Calls (std/external/unresolved):** `.into`, `.repeat`
- **Called by:** `Parser::nested_list` (markdown/math.rs:867)

#### `prime_count`  (L1476-1488, pub(super))
`pub(super) fn prime_count(text: &str) -> Option<usize>`

- **Does:** How many primes `text` is, if it is only primes.
- **Calls (std/external/unresolved):** `.chars`, `.then_some`
- **Called by:** `math_inner` (markdown/from_docx/mod.rs:3946)

#### `negated`  (L1492-1523)
`fn negated(relation: &str) -> String`

- **Does:** `relation` negated: its own negated character, or the combining long solidus (U+0338) Word overlays.
- **Calls (std/external/unresolved):** `.find`, `.iter`, `.map_or_else`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `delimiter`  (L1525-1532)
`fn delimiter(open: &str, close: &str, body: Vec<Node>) -> Node`

- **Does:** _no doc comment_
- **Calls (std/external/unresolved):** `.into`
- **Macros:** `vec!`
- **Called by:** `Parser::command` (markdown/math.rs:1096); `Parser::environment` (markdown/math.rs:1403)

#### `escaped`  (L1535-1544)
`fn escaped(name: &str) -> String`

- **Does:** The text a command names when it is not one: `\{` is `{`, `\foo` stays.
- **Calls (std/external/unresolved):** `.find`, `.into`, `.iter`, `.map_or_else`, `.to_string`
- **Macros:** `format!`
- **Called by:** `Parser::raw_argument` (markdown/math.rs:944); `Parser::command` (markdown/math.rs:1096); `Parser::fence` (markdown/math.rs:1331)

#### `plain`  (L1547-1553)
`fn plain(node: &Node) -> String`

- **Does:** The text of a node, for an argument read as text.
- **Calls (std/external/unresolved):** `.clone`, `.collect`, `.iter`, `.map`, `String::new`
- **Called by:** none found in src

#### `styled`  (L1556-1567)
`fn styled(nodes: Vec<Node>, style: Style) -> Node`

- **Does:** `nodes` with every run set in `style`.
- **Calls (crate):** `styled::{fn restyle}` (markdown/math.rs:1557) L1566
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Node::Group`
- **Called by:** `Parser::command` (markdown/math.rs:1096)

#### `styled::{fn restyle}`  (L1557-1565)
`fn restyle(node: Node, style: Style) -> Node`

- **Does:** _no doc comment_
- **Calls (crate):** `styled::{fn restyle}` (markdown/math.rs:1557) L1561
- **Calls (std/external/unresolved):** `.collect`, `.into_iter`, `.map`, `Node::Group`, `Node::Text`
- **Called by:** `styled` (markdown/math.rs:1556); `styled::{fn restyle}` (markdown/math.rs:1557)

#### `take_base`  (L1571-1585)
`fn take_base(out: &mut Vec<Node>) -> Vec<Node>`

- **Does:** The base a script takes from the nodes before it: the last node, or the last character of a run of text.
- **Calls (std/external/unresolved):** `.chars`, `.collect`, `.into_iter`, `.is_empty`, `.map`, `.pop`, `.push`, `.unwrap_or_default`, `Node::Text`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Parser::nested_list` (markdown/math.rs:867)

#### `trimmed`  (L1590-1599)
`fn trimmed(mut nodes: Vec<Node>) -> Vec<Node>`

- **Does:** `nodes` without the spaces at their edges: the reader pads matrix cells, array lines and equations (`a & b \\ c`), and a person writes `\left( a \right)`; spaces inside stay, as Word keeps them.
- **Calls (std/external/unresolved):** `.first_mut`, `.is_empty`, `.last_mut`, `.retain`, `.to_string`, `.trim_end`, `.trim_start`, `Node::Text`
- **Macros:** `matches!`
- **Called by:** `Parser::left` (markdown/math.rs:1294); `Parser::environment` (markdown/math.rs:1403); `equation` (markdown/math.rs:1630)

#### `merge`  (L1603-1625)
`fn merge(nodes: Vec<Node>) -> Vec<Node>`

- **Does:** Neighbouring runs of one style as one run, and groups that are no script's base written inline.
- **Calls (std/external/unresolved):** `.last_mut`, `.push_str`, `.push`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `Parser::nested_list` (markdown/math.rs:867); `Parser::command` (markdown/math.rs:1096); `equation` (markdown/math.rs:1630)

#### `equation`  (L1630-1661)
`fn equation(latex: &str) -> Vec<Node>`

- **Does:** The equation `latex` is.
- **Calls (crate):** `Parser::new` (markdown/math.rs:792) L1631; `Parser::list` (markdown/math.rs:833) L1634; `Parser::peek` (markdown/math.rs:804) L1641; `is_command` (markdown/math.rs:691) L1642; `trimmed` (markdown/math.rs:1590) L1656; `merge` (markdown/math.rs:1603) L1656
- **Calls (std/external/unresolved):** `.collect`, `.extend`, `.into_iter`, `.into`, `.last_mut`, `.len`, `.map`, `.pop`, `.push`, `.unwrap_or_default`, `Node::Array`, `Node::Text`, `Stops::default`, `Vec::new`
- **Macros:** `vec!`
- **Called by:** `omml` (markdown/math.rs:1923)

#### `Writer::marked`  (L1674-1683)
`fn marked(&mut self, inner: &str, change: Option<Kind>)`

- **Does:** `inner` inside a fresh `w:ins` or `w:del` when `change` is one.
- **Calls (std/external/unresolved):** `.next`, `.push_str`, `.split_whitespace`, `.to_string`, `.unwrap_or`
- **Macros:** `write!`
- **Called by:** `Writer::control` (markdown/math.rs:1685); `Writer::run` (markdown/math.rs:1698); `build_with` (markdown/unified.rs:522) (?)

#### `Writer::control`  (L1685-1689)
`fn control(&mut self)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::marked` (markdown/math.rs:1674) L1687
- **Calls (std/external/unresolved):** `.push_str`
- **Called by:** `Transaction::select_control` (edit/controls.rs:205) (?); `Writer::properties` (markdown/math.rs:1692)

#### `Writer::properties`  (L1692-1696)
`fn properties(&mut self, name: &str, properties: &str)`

- **Does:** A structure's properties: `properties`, then the control.
- **Calls (crate):** `Writer::control` (markdown/math.rs:1685) L1694
- **Macros:** `write!`
- **Called by:** `Writer::node` (markdown/math.rs:1734)

#### `Writer::run`  (L1698-1719)
`fn run(&mut self, text: &str, style: Style, change: Option<Kind>)`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::marked` (markdown/math.rs:1674) L1714; `escape` (markdown/xml.rs:426) L1715
- **Calls (std/external/unresolved):** `.ends_with`, `.into`, `.or`, `.push_str`, `.starts_with`, `String::new`
- **Macros:** `format!`
- **Called by:** `boundary_coverage_tests::run_projection_preserves_field_delimiters_breaks_and_old_formatting` (debug/diff.rs:2372) (?); `raw_source_projection_contract_tests::recorded_run_properties_and_saved_page_breaks_keep_exact_raw_source_projection` (debug/diff.rs:2538) (?); `Writer::inline` (markdown/from_docx/mod.rs:3021) (?); `Writer::node` (markdown/math.rs:1734) (?)

#### `Writer::argument`  (L1722-1726)
`fn argument(&mut self, name: &str, nodes: &[Node])`

- **Does:** `<m:name>nodes</m:name>`.
- **Calls (crate):** `Writer::nodes` (markdown/math.rs:1728) L1724
- **Macros:** `write!`
- **Called by:** `Parser::function` (markdown/math.rs:1354) (?); `Writer::node` (markdown/math.rs:1734)

#### `Writer::nodes`  (L1728-1732)
`fn nodes(&mut self, nodes: &[Node])`

- **Does:** _no doc comment_
- **Calls (crate):** `Writer::node` (markdown/math.rs:1734) L1730
- **Called by:** `public_memory_append_owner_boundary_tests::semantic` (append.rs:1448) (?); `CommentFamily::set_text` (comments.rs:737) (?); `coverage_real_revision_merge_matrix_tests::multiple_original_paragraphs_before_a_live_blank_control_tail_keep_exact_phase_payloads` (comparer/finalize.rs:25406) (?); `finalizer_distinct_boundary_contract_tests::semantic` (comparer/finalize.rs:29448) (?); `finalizer_remaining_source_boundary_tests::semantic` (comparer/finalize.rs:30689) (?); `finalizer_merge_source_guard_tests::semantic` (comparer/finalize.rs:31431) (?); `finalizer_reached_predicate_boundary_tests::semantic` (comparer/finalize.rs:31868) (?); `public_body_heuristic_story_scope_tests::canonical` (comparer/finalize.rs:32220) (?); `default_source_property_carrier_tests::semantic` (comparer/finalize.rs:32559) (?); `existing_mark_layout_ownership_tests::semantic` (comparer/finalize.rs:32742) (?); `interleave_terminal_source_ownership_tests::semantic` (comparer/finalize.rs:33070) (?); `whitespace_source_mark_carrier_tests::semantic` (comparer/finalize.rs:33289) (?); +14 more

#### `Writer::node`  (L1734-1914)
`fn node(&mut self, node: &Node)`

- **Does:** _no doc comment_
- **Calls (crate):** `.run` (ambiguous: `Writer::run` (markdown/from_docx/mod.rs:3119); `Writer::run` (markdown/math.rs:1698)) L1736; `Writer::nodes` (markdown/math.rs:1728) L1737; `Writer::properties` (markdown/math.rs:1692) L1745,1757,1769,1783,1806,1814...; `Writer::argument` (markdown/math.rs:1722) L1746,1747,1758,1759,1770,1772...; `escape` (markdown/xml.rs:426) L1798,1801,1804
- **Calls (std/external/unresolved):** `.get`, `.is_empty`, `.is_none`, `.iter`, `.map_or`, `.map`, `.max`, `.push_str`, `.unwrap_or`, `String::new`
- **Macros:** `format!`, `matches!`, `write!`
- **Called by:** `Writer::nodes` (markdown/math.rs:1728)

#### `omml`  (L1923-1949, pub(super))
`pub(super) fn omml( latex: &str, display: bool, change: Option<Kind>, mark: &mut dyn FnMut(Kind) -> String, ) -> String`

- **Does:** `latex` as Office Math: display math as an `m:oMath` in an `m:oMathPara`, inline math as an `m:oMath`.
- **Calls (crate):** `Writer::nodes` (markdown/math.rs:1728) L1943; `equation` (markdown/math.rs:1630) L1943
- **Calls (std/external/unresolved):** `.as_str`, `.push_str`, `String::new`
- **Macros:** `format!`, `write!`
- **Called by:** `tests::shape` (markdown/math.rs:1956); `tests::more_nary_operators_and_their_limits` (markdown/math.rs:2014); `tests::a_change_marks_every_run_and_control_inside` (markdown/math.rs:2177); `tests::a_change_inside_marks_only_its_runs` (markdown/math.rs:2190); `tests::display_lines_are_an_equation_array` (markdown/math.rs:2204); `story` (markdown/xml.rs:587)

## Test functions (19)

- `tests::shape` (L1956): The math's element names and text, without properties or fonts.
- `tests::ams_and_unicode_math_names_are_symbols` (L2005): _no doc comment_
- `tests::more_nary_operators_and_their_limits` (L2014): _no doc comment_
- `tests::no_command_is_defined_twice` (L2037): _no doc comment_
- `tests::scripts_take_the_last_character_as_base` (L2048): _no doc comment_
- `tests::balanced_brackets_build_a_delimiter_and_strays_stay_text` (L2055): _no doc comment_
- `tests::structures_come_back_as_word_writes_them` (L2063): _no doc comment_
- `tests::an_nary_takes_one_operand_with_its_scripts` (L2096): _no doc comment_
- `tests::an_unknown_command_stays_as_text` (L2104): _no doc comment_
- `tests::primes_are_a_superscript_as_word_builds_them` (L2112): _no doc comment_
- `tests::not_negates_the_next_relation` (L2121): _no doc comment_
- `tests::sized_delimiters_are_delimiters` (L2129): _no doc comment_
- `tests::nesting_past_the_limit_reads_as_text_instead_of_overflowing` (L2136): _no doc comment_
- `tests::scripts_within_the_limit_still_nest` (L2151): _no doc comment_
- `tests::an_array_is_a_matrix_without_its_column_spec` (L2157): _no doc comment_
- `tests::modulo_and_spacing_commands` (L2170): _no doc comment_
- `tests::a_change_marks_every_run_and_control_inside` (L2177): _no doc comment_
- `tests::a_change_inside_marks_only_its_runs` (L2190): _no doc comment_
- `tests::display_lines_are_an_equation_array` (L2204): _no doc comment_
