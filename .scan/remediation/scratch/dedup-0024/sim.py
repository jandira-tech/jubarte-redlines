import random, sys

# ---------- unescape_xml_text ----------
def decode_entity(body):
    if not body: return None
    if body[0] == '#':
        if len(body) > 1 and body[1] in 'xX':
            try: code = int(body[2:], 16)
            except ValueError: return None
        else:
            try: code = int(body[1:], 10)
            except ValueError: return None
            if any(c not in '0123456789' for c in body[1:]): return None
        # rust u32 from_str_radix rejects '-'; python int() accepts it -> guard
        if code < 0 or code > 0xFFFFFFFF: return None
        if code > 0x10FFFF or 0xD800 <= code <= 0xDFFF: return None
        return chr(code)
    if not all('a' <= c <= 'z' or 'A' <= c <= 'Z' for c in body): return None
    return {'lt':'<','gt':'>','amp':'&','quot':'"','apos':"'"}.get(body)

def old_xml(s):
    if '&' not in s: return s
    chars = list(s); out = []; i = 0
    while i < len(chars):
        if chars[i] == '&':
            tail = chars[i+1:]
            semi = tail.index(';') if ';' in tail else None
            if semi is not None:
                body = ''.join(tail[:semi])
                d = decode_entity(body)
                if d is not None:
                    out.append(d); i = i + 1 + semi + 1; continue
        out.append(chars[i]); i += 1
    return ''.join(out)

def new_xml(s):
    if '&' not in s: return s
    chars = list(s); out = []; i = 0
    while i < len(chars):
        if chars[i] == '&':
            j = i + 1
            if j < len(chars) and chars[j] == '#': j += 1
            while j < len(chars) and (chars[j].isascii() and (chars[j].isalnum() or chars[j] == '+')):
                j += 1
            if j < len(chars) and chars[j] == ';':
                d = decode_entity(''.join(chars[i+1:j]))
                if d is not None:
                    out.append(d); i = j + 1; continue
        out.append(chars[i]); i += 1
    return ''.join(out)

pinned = [("&;","&;"), ("&#;","&#;"), ("&#X41;","A"), ("&#x1F433;","🐳"),
          ("&#1114112;","&#1114112;"), ("&#55296;","&#55296;"), ("&bad1;","&bad1;"),
          ("&é;","&é;"), ("&unknown;","&unknown;"), ("before &amp","before &amp"),
          ("A &lt; B &amp; C", "A < B & C")]
for src, want in pinned:
    got_new, got_old = new_xml(src), old_xml(src)
    assert got_new == want, f"pinned new {src!r}: {got_new!r}"
    assert got_old == want
extra = ["&#000000000000000000065;", "&#x+41;", "&#+65;", "&#x-41;", "&" + "z"*50 + ";",
         "&é&nbsp;", "x&amp&amp;y", "&&&amp;;;", "&#xZZ;", "&aé;", "&am#p;", "&#65#;", "&+;", "&a+mp;"]
for src in extra:
    a, b = new_xml(src), old_xml(src)
    assert a == b, f"divergence {src!r}: new {a!r} old {b!r}"
alphabet = list("abcdef&amp;#;<>+'éx \t")
random.seed(7)
for _ in range(200000):
    s = ''.join(random.choice(alphabet) for _ in range(random.randrange(0, 14)))
    assert new_xml(s) == old_xml(s), f"fuzz divergence {s!r}: {new_xml(s)!r} vs {old_xml(s)!r}"
print("unescape_xml_text: pinned + extra + 200k fuzz inputs identical")

# ---------- decode_entities (HTML) ----------
def old_html(s):
    if '&' not in s: return s
    out = ""; rest = s
    while True:
        a = rest.find('&')
        if a < 0: return out + rest
        out += rest[:a]; tail = rest[a:]
        e = tail.find(';')
        semi = e if (e is not None and e != -1 and e <= 10) else None
        dec = None
        if semi is not None:
            ent = tail[1:e]
            ch = {'amp':'&','lt':'<','gt':'>','quot':'"','apos':"'",'nbsp':'\xa0'}.get(ent)
            if ch is None:
                if ent[:2] in ('#x','#X'):
                    try:
                        v = int(ent[2:], 16); ch = chr(v) if 0 <= v <= 0x10FFFF and not (0xD800 <= v <= 0xDFFF) else None
                    except ValueError: ch = None
                elif ent[:1] == '#':
                    try:
                        v = int(ent[1:], 10); ch = chr(v) if 0 <= v <= 0x10FFFF and not (0xD800 <= v <= 0xDFFF) else None
                    except ValueError: ch = None
            if ch is not None: dec = (ch, e)
        if dec is not None:
            out += dec[0]; rest = tail[dec[1]+1:]
        else:
            out += '&'; rest = tail[1:]

def new_html(s):
    if '&' not in s: return s
    out = ""; rest = s
    while True:
        a = rest.find('&')
        if a < 0: return out + rest
        out += rest[:a]; tail = rest[a:]
        window = min(len(tail), 11)
        while window < len(tail) and (window < 0 or not is_char_boundary(tail, window)):
            window += 1
        seg = tail[:window]
        e = seg.find(';')
        semi = e if (e != -1 and e <= 10) else None
        # identical decision path from here
        dec = None
        if semi is not None:
            ent = tail[1:e]
            ch = {'amp':'&','lt':'<','gt':'>','quot':'"','apos':"'",'nbsp':'\xa0'}.get(ent)
            if ch is None:
                if ent[:2] in ('#x','#X'):
                    try:
                        v = int(ent[2:], 16); ch = chr(v) if 0 <= v <= 0x10FFFF and not (0xD800 <= v <= 0xDFFF) else None
                    except ValueError: ch = None
                elif ent[:1] == '#':
                    try:
                        v = int(ent[1:], 10); ch = chr(v) if 0 <= v <= 0x10FFFF and not (0xD800 <= v <= 0xDFFF) else None
                    except ValueError: ch = None
            if ch is not None: dec = (ch, e)
        if dec is not None:
            out += dec[0]; rest = tail[dec[1]+1:]
        else:
            out += '&'; rest = tail[1:]

def is_char_boundary(s, i):
    # utf-8 boundary: not a continuation byte 10xxxxxx
    if i >= len(s): return True
    b = s[i].encode('utf-8')[0] if ord(s[i]) < 128 else None
    # python strings are code points; emulate byte view
    bs = s.encode('utf-8')
    return bs[i] & 0xC0 != 0x80

for src, want in [("&amp;&lt;&gt;&quot;&apos;&nbsp;&#65;&#x1F600;&#X41;", "&<>\"'\xa0A\U0001F600A")]:
    got = new_html(src)
    assert got == want, repr(got)
invalid = "&unknown; &#xD800; &#1114112; &#xZZ; &unfinished"
assert new_html(invalid) == invalid == old_html(invalid)
halpha = list("abcdef&amp;#;<>é0 ")
random.seed(11)
for _ in range(200000):
    s = ''.join(random.choice(halpha) for _ in range(random.randrange(0, 14)))
    assert new_html(s) == old_html(s), f"fuzz {s!r}: {new_html(s)!r} vs {old_html(s)!r}"
for s in ["&"*100000, "&" + "x"*40 + ";", "&é&nbsp;"]:
    assert new_html(s) == old_html(s)
print("decode_entities: pinned + 200k fuzz inputs identical")

# ---------- blocks() field state ----------
def vis_old(marks):  # returns list of (char, visible) for non-marker chars
    fields = []; res = []
    for ch in marks:
        in_code = any(fields)
        if ch == '\x13': fields.append(True); continue
        if ch == '\x14':
            if fields: fields[-1] = False
            continue
        if ch == '\x15':
            if fields: fields.pop()
            continue
        if in_code: res.append(ch); continue
        res.append(ch.upper())  # mark visible distinctly
    # visible text as joined uppercase-ness: return two lists
    return res

def vis_new(marks):
    fields = []; code_frames = 0; res = []
    for ch in marks:
        in_code = code_frames > 0
        if ch == '\x13': fields.append(True); code_frames += 1; continue
        if ch == '\x14':
            if fields:
                if fields[-1]: code_frames -= 1
                fields[-1] = False
            continue
        if ch == '\x15':
            if fields and fields.pop(): code_frames -= 1
            continue
        if in_code: res.append(ch); continue
        res.append(ch.upper())
    return res

random.seed(3)
marks = ['\x13', '\x14', '\x15']
for _ in range(300000):
    seq = [random.choice(marks + ['a','b','\r']) for _ in range(random.randrange(0, 16))]
    # compare full visible/hidden partition, not just visible text
    def partition(fn, seq):
        fields = []; cf = 0; vis = []
        for ch in seq:
            if ch == '\x13': fields.append(True); cf += 1; continue
            if ch == '\x14':
                if fields:
                    if fields[-1]: cf -= 1
                    fields[-1] = False
                continue
            if ch == '\x15':
                if fields and fields.pop(): cf -= 1
                continue
            vis.append((ch, not (cf > 0)))
        return vis
    # old predicate via any()
    def partition_old(seq):
        fields = []; vis = []
        for ch in seq:
            in_code = any(fields)
            if ch == '\x13': fields.append(True); continue
            if ch == '\x14':
                if fields: fields[-1] = False
                continue
            if ch == '\x15':
                if fields: fields.pop()
                continue
            vis.append((ch, not in_code))
        return vis
    assert partition(None, seq) == partition_old(seq), f"field-state divergence {seq!r}"
print("blocks() code_frames predicate: 300k fuzz sequences identical to any()")

# ---------- bang_construct_end ----------
def bce(s, at):
    ln = len(s); quote = '\0'; i = at + 2
    while i < ln:
        ch = s[i]
        if quote != '\0':
            if ch == quote: quote = '\0'
        elif ch in '"\'': quote = ch
        elif ch == '>': return i + 1
        elif ch == '<' and i + 1 < ln and s[i+1] == '!':
            close = None
            if i + 3 < ln and s[i+2] == '-' and s[i+3] == '-': close = '-->'
            elif i + 2 < ln and s[i+2] == '[': close = ']]>'
            if close is not None:
                e = s.find(close, i)
                if e < 0: return ln
                i = e + 3; continue
        i += 1
    return ln

cases = [
    ('<!DOCTYPE d [<!ENTITY e "x><a><a><a>">]><r>t</r>', 0, len('<!DOCTYPE d [<!ENTITY e "x><a><a><a>">]>')),
    ('<!DOCTYPE d [<!-- c > d -->]><r/>', 0, len('<!DOCTYPE d [<!-- c > d -->]>')),
    ('<!DOCTYPE x>', 0, len('<!DOCTYPE x>')),
    ('<!DOCTYPE d [<!ATTLIST e a CDATA "1"> <!ELEMENT e (#PCDATA)>]>', 0, len('<!DOCTYPE d [<!ATTLIST e a CDATA "1"> <!ELEMENT e (#PCDATA)>]>')),
    ('<!DOCTYPE d [<![INCLUDE[ <a> ]]>]>', 0, len('<!DOCTYPE d [<![INCLUDE[ <a> ]]>]>')),
    ('<!DOCTYPE d [<!ENTITY e "unterminated', 0, len('<!DOCTYPE d [<!ENTITY e "unterminated')),
    ('<!DOCTYPE d [<!-- never closes', 0, len('<!DOCTYPE d [<!-- never closes')),
    ('<!foo "a>b">', 0, len('<!foo "a>b">')),
]
for s, at, want in cases:
    got = bce(s, at)
    assert got == want, f"bce {s!r}: {got} != {want}"
print("bang_construct_end: all boundary cases return the expected end")
