#!/bin/zsh
# Reproduces the jubarte 0.12.0 findings. Run from /tmp/jb-bugs/probe/repro. Outputs go to ./out
set -u
J=~/.local/bin/jubarte
T=/Users/arthrod/temp/T/redline-bench/.tasks
S1T1=$T/redline-s1-t1-g01a/environment/app/contract.docx
S2T1=$T/redline-s2-t1-g01a/environment/app/contract.docx
S2T2=$T/redline-s2-t2-g03c/environment/app/contract.docx
S1T3=$T/redline-s1-t3-g03a/environment/app/contract.docx
S2T3=$T/redline-s2-t3-g06a/environment/app/contract.docx
S3T4=$T/redline-s3-t4-g13a/environment/app/contract.docx
D=2026-01-02T03:04:05Z
mkdir -p out
echo "### B1 delete paragraph (clean doc) touches next paragraph"
$J edit $S1T1 -a "Ana Lee" -d $D -p p14 --delete -o out/B1.docx --force -q; $J changes out/B1.docx
$J accept out/B1.docx --id r1 --id r2 --id r3 --id r4 -o out/B1_acc.docx --force -q; $J reject out/B1_acc.docx -o out/B1_final.docx --force -q
$J read out/B1_final.docx --no-page-markers -p p14 | tail -1 | cut -c1-60
echo "### B2 replace: whole replacement takes first run's formatting (clean + keep paths)"
$J edit $S1T1 -a "Ana Lee" -d $D -p p6 --anchor '“Affiliates” means any' --content '“Affiliates” means each' -o out/B2a.docx --force -q; $J changes out/B2a.docx
$J edit $S1T1 -a "Ana Lee" -d $D -p p6 --anchor 'Affiliates” means' --content 'Affiliated Entities” means' -o out/B2b.docx --force -q; $J changes out/B2b.docx
$J edit $S2T2 -a "Ana Lee" -d $D -p p73 --anchor "Interpretation. If any" --content "Construction. If any" -o out/B2c.docx --force -q; $J read out/B2c.docx --no-page-markers -p p73 | tail -1 | cut -c1-140
echo "### B3 lenient anchor match rewrites curly quotes / nbsp"
$J edit $S1T1 -a "Ana Lee" -d $D -p p6 --anchor '"Affiliates" means any' --content '"Affiliates" means each' -o out/B3a.docx --force -q; $J changes out/B3a.docx
$J edit $S1T1 -a "Ana Lee" -d $D -p p16 --anchor "(1) any officer" --content "(1) every officer" -o out/B3b.docx --force -q; $J changes out/B3b.docx
echo "### B4 insert_paragraph + edit in next paragraph steals the opening quote"
$J edit $S1T1 --plan B4_plan.json -o out/B4.docx --force -q; $J changes out/B4.docx
$J reject out/B4.docx --id r1 --id r2 -o out/B4_rej.docx --force -q; $J read out/B4_rej.docx --no-page-markers -p p23 | tail -1 | cut -c1-60
echo "### B5 insert_paragraph with style after numbered heading stays numbered"
$J edit $S1T1 --plan B5_plan.json -o out/B5.docx --force -q; $J read out/B5.docx --no-page-markers -p p25-p27 | grep -A1 '<!--' | cut -c1-90
echo "### B6 new paragraph after a paragraph whose mark is a pending deletion gets merged on Accept All"
$J add $S2T2 -a "Ana Lee" -d $D -p p3 --content "NEW PARAGRAPH AFTER P3." -o out/B6.docx --force -q; $J accept out/B6.docx -o out/B6_acc.docx --force -q
$J read out/B6_acc.docx --json -p p3 | python3 -c 'import json,sys; print(repr(json.load(sys.stdin)["paragraphs"][0]["text"][-60:]))'
echo "### B7 accept/reject (no filter) renumber comment ids"
$J comments $S3T4 | grep -E $'^c(7|9)\t' | cut -c1-80; $J reject $S3T4 -o out/B7.docx --force -q; $J comments out/B7.docx | grep -E $'^c(6|7)\t' | cut -c1-80
echo "### B8 edit/delete another author's comment silently"
$J edit $S2T2 -a "Ana Lee" -d $D -p c106 --content "We will share PHI with AgentCo." -o out/B8.docx --force -q; $J comments out/B8.docx
echo "### B9 read view pairs cross-author ins/del into a fake substitution"
$J edit $S2T2 -a "Ana Lee" -d $D -p p47 --anchor "non-exclusive" --content "nonexclusive" -o out/B9.docx --force -q; $J read out/B9.docx --no-page-markers -p p47 | tail -1 | grep -o 'Affiliates a .\{170\}'
echo "### B10 read view drops 'on rN' for a deletion inside a multi-run insertion"
$J read $S1T3 --no-page-markers -p p8 | tail -1 | cut -c1-200; $J changes $S1T3 | grep -E '^r1[3-8]\b'
echo "### B11 comparer edit remaps numbering of untouched paragraphs + duplicate num durableIds"
$J accept $S2T3 -o out/B11_clean.docx --force -q
$J edit out/B11_clean.docx -a "Ana Lee" -d $D -p p212 --anchor "Help Center and email:" --content "Help Center or by email:" -o out/B11.docx --force -q
$J changes out/B11.docx | awk -F'\t' '{print $2"\t"$3}' | sort | uniq -c
$J edit $S2T1 -a "Ana Lee" -d $D -p p3 --anchor "sets out" --content "states" -o out/B11b.docx --force -q
unzip -p out/B11b.docx word/numbering.xml | grep -o '<w:num w:numId="[0-9]*" w16cid:durableId="[0-9]*"' | tail -4
echo "### B12 changes text drops tabs"
$J edit $S1T1 -a "Ana Lee" -d $D -p p86 --anchor $'Title: \t\tTitle:' --content $'Position: \t\tPosition:' -o out/B12.docx --force -q; $J changes out/B12.docx --json
