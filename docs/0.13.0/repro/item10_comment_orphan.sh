#!/bin/zsh
# Item 10: accepting a deletion that holds a comment's reference mark deletes the comment
# silently and orphans its replies. jubarte 0.12.0, task redline-s1-t3-g15a.
J=~/.local/bin/jubarte
IN=/Users/arthrod/temp/T/redline-bench/.tasks/redline-s1-t3-g15a/environment/app/contract.docx
mkdir -p out
$J changes $IN | awk -F'\t' '$1=="r13"'
$J edit $IN -a "Reviewing Counsel (AgentCo)" -p r13 --accept -o out/item10.docx --force
$J comments out/item10.docx | grep -E $'^c(4|5)\t'
$J validate out/item10.docx | grep paraIdParent
