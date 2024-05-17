## rust-bitcoin-m31-with-muldiv

This repository implements M31 field arithmetic assuming `OP_MUL` and `OP_DIV` in Bitcoin Script.

### Performance

For M31, we have:

- addition: 18 weight units
- subtraction: 12 weight units
- multiplication: 215 weight units

For the degree-4 extension of M31 using y^2 - 2 - i over the complex field x^2 + 1, we have:

- addition: 84 weight units
- subtraction: 63 weight units
- multiplication: 2521 weight units
- multiplication by M31: 877 weight units

### Credits

The implementation is based on [BitVM/rust-bitcoin-m31-or-babybear](https://www.github.com/BitVM/rust-bitcoin-m31-or-babybear), 
with the main changes of assuming the existence of `OP_MUL` and `OP_DIV`.