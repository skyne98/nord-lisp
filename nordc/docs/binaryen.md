1. Binaryen's text format allows only s-expressions. WebAssembly's official text format is primarily a linear instruction list (with s-expression extensions).
2. https://github.com/WebAssembly/binaryen/issues/1705
   - wat2wasm.exe .\nordc\test_scripts\var_set.wat -o .\nordc\test_scripts\var_set.wasm
   - wasm-opt.exe -O4 .\nordc\test_scripts\var_set.wasm -S --output .\nordc\test_scripts\var_set.opt.wat --debug -all