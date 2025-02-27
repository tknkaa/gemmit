#Gemmit

set your api key
```
export GEMINI_API_KEY
```

build and publish
```
cargo build --release
cp target/release/gemmit pkg/
npm login
cd pkg
npm publish --access public
```