## rust-bitcoin-m31-or-babybear

This repository implements the Bitcoin script for processing the M31 or BabyBear field.

### Performance

In the current implementation, M31 and BabyBear has equivalent performance for the standalone field. 
The overhead for field extension is, however, very different, since M31's field extension is much more complicated.

- addition: 18 weight units
- subtraction: 12 weight units
- multiplication: 1767 weight units

For the degree-4 extension of BabyBear over x^4 - 11, we have:

- addition: 84 weight units
- subtraction: 63 weight units
- multiplication: 21992 weight units