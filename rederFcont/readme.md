pour compiler :
```
ocamlfind ocamlopt -O3 -o renderFcont -linkpkg -package oplot -thread renderFcont.ml && time ./renderFcont
```
et faut dowmload le module oplot !