# near-guess-number

## Project setup
```
yarn install
```

### Compiles and hot-reloads for development
```
yarn serve
```

### Compiles and minifies for production
```
yarn build
```

### contract
```
cd contract

# contract build
cargo build --target wasm32-unknown-unknown --release

# contract test
cargo test -- --nocapture

# contract dev-deploy
near dev-deploy target/wasm32-unknown-unknown/release/near_guess_number.wasm
```

