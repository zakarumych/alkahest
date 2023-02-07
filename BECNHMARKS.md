# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [ser](#ser)
    - [de](#de)

## Benchmark Results

### ser

|        | `alkahest`               | `json`                          | `bincode`                       | `rkyv`                           |
|:-------|:-------------------------|:--------------------------------|:--------------------------------|:-------------------------------- |
|        | `25.14 ns` (✅ **1.00x**) | `95.11 ns` (❌ *3.78x slower*)   | `23.27 ns` (✅ **1.08x faster**) | `68.63 ns` (❌ *2.73x slower*)    |

### de

|        | `alkahest`               | `json`                            | `bincode`                         | `rkyv`                           |
|:-------|:-------------------------|:----------------------------------|:----------------------------------|:-------------------------------- |
|        | `18.19 ns` (✅ **1.00x**) | `300.28 ns` (❌ *16.51x slower*)   | `661.77 ns` (❌ *36.38x slower*)   | `19.04 ns` (✅ **1.05x slower**)  |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)