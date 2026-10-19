#!/bin/zsh
# Item 15: rewriting a counterparty comment under their name (s2-t2-g03c).
# Item 16: a bare reject renumbers every comment id (s3-t4-g13a).
J=~/.local/bin/jubarte
T=/Users/arthrod/temp/T/redline-bench/.tasks
mkdir -p out
$J edit $T/redline-s2-t2-g03c/environment/app/contract.docx -a "Reviewing Counsel (AgentCo)" -p c106 --content "We will share PHI with AgentCo." -o out/item15.docx --force -q
unzip -p out/item15.docx word/comments.xml | grep -o '<w:comment w:id="106"[^>]*>'
$J comments out/item15.docx | grep -E $'^c106\t'
$J comments $T/redline-s3-t4-g13a/environment/app/contract.docx | grep -E $'^c(7|9|12)\t' | cut -c1-70
$J reject $T/redline-s3-t4-g13a/environment/app/contract.docx -o out/item16.docx --force -q
$J comments out/item16.docx | grep -E $'^c(6|7|9)\t' | cut -c1-70
