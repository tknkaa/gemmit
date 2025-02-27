# Gemmit

set your api key
```
export GEMINI_API_KEY=your_api_key
```

build and publish
```
cargo build --release
cp target/release/gemmit pkg/
npm login
cd pkg
npm publish --access public
```