# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [log](#log)
    - [mesh](#mesh)
    - [minecraft_savedata](#minecraft_savedata)

## Benchmark Results

### log

|                                 | `alkahest`                | `bincode`                        | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `290.48 us` (✅ **1.00x**) | `456.00 us` (❌ *1.57x slower*)   | `302.38 us` (✅ **1.04x slower**) | `286.58 us` (✅ **1.01x faster**)  |
| **`read`**                      | `325.61 us` (✅ **1.00x**) | `1.66 ms` (❌ *5.10x slower*)     | `502.11 us` (❌ *1.54x slower*)   | `1.57 ms` (❌ *4.82x slower*)      |
| **`deserialize`**               | `1.74 ms` (✅ **1.00x**)   | `1.65 ms` (✅ **1.06x faster**)   | `1.83 ms` (✅ **1.05x slower**)   | `1.54 ms` (✅ **1.13x faster**)    |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `8.31 us` (✅ **1.00x**)          | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `1.32 ms` (✅ **1.00x**)          | `N/A`                             |

### mesh

|                                 | `alkahest`                | `bincode`                       | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:--------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `384.37 us` (✅ **1.00x**) | `5.34 ms` (❌ *13.90x slower*)   | `324.15 us` (✅ **1.19x faster**) | `128.40 us` (🚀 **2.99x faster**)  |
| **`read`**                      | `59.06 us` (✅ **1.00x**)  | `1.84 ms` (❌ *31.12x slower*)   | `39.74 us` (✅ **1.49x faster**)  | `373.85 us` (❌ *6.33x slower*)    |
| **`deserialize`**               | `590.87 us` (✅ **1.00x**) | `1.73 ms` (❌ *2.92x slower*)    | `237.76 us` (🚀 **2.49x faster**) | `265.78 us` (🚀 **2.22x faster**)  |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                           | `39.84 us` (✅ **1.00x**)         | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                           | `257.47 us` (✅ **1.00x**)        | `N/A`                             |

### minecraft_savedata

|                                 | `alkahest`                | `bincode`                        | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `282.45 us` (✅ **1.00x**) | `503.47 us` (❌ *1.78x slower*)   | `339.61 us` (❌ *1.20x slower*)   | `287.98 us` (✅ **1.02x slower**)  |
| **`read`**                      | `38.43 us` (✅ **1.00x**)  | `1.43 ms` (❌ *37.12x slower*)    | `384.05 us` (❌ *9.99x slower*)   | `1.28 ms` (❌ *33.23x slower*)     |
| **`deserialize`**               | `1.69 ms` (✅ **1.00x**)   | `1.43 ms` (✅ **1.18x faster**)   | `1.51 ms` (✅ **1.11x faster**)   | `1.28 ms` (✅ **1.32x faster**)    |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `173.15 ns` (✅ **1.00x**)        | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `1.11 ms` (✅ **1.00x**)          | `N/A`                             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

