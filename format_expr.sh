#!/usr/bin/env bash
gsed -Ei 's/a/0/g; s/b/40/g; s/c/65/g; s/d/80/g; s/e/80/g; s/f/80/g; s/g/70/g; s/h/0/g;' regex.txt
gsed -Ei 's/p/0/g;' regex.txt
# gsed -Ei 's/([0-9]+)/\1./g; s/t\^([0-9])\./f64::powi(t,\1)/g' regex.txt
# gsed -Ei ':a;N;$!ba;s/\n\s*//g' regex.txt