## 2026-11-07 - [Optimize clone() on XName inside hot loops]
**Learning:** Calling .clone() on XName objects when passed as references inside hot loops like dom.name_is causes unnecessary atomic increment operations, hurting performance.
**Action:** Removed .clone() when passing XName instances to dom.name_is and similar methods.
