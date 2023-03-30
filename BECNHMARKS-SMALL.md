# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [log](#log)
    - [mesh](#mesh)
    - [minecraft_savedata](#minecraft_savedata)

## Benchmark Results

### log

|                                 | `alkahest`               | `bincode`                       | `rkyv`                          | `speedy`                         |
|:--------------------------------|:-------------------------|:--------------------------------|:--------------------------------|:-------------------------------- |
| **`serialize`**                 | `1.83 us` (✅ **1.00x**)  | `3.74 us` (❌ *2.04x slower*)    | `2.08 us` (❌ *1.13x slower*)    | `1.87 us` (✅ **1.02x slower**)   |
| **`read`**                      | `1.95 us` (✅ **1.00x**)  | `15.14 us` (❌ *7.75x slower*)   | `3.67 us` (❌ *1.88x slower*)    | `14.62 us` (❌ *7.48x slower*)    |
| **`deserialize`**               | `14.46 us` (✅ **1.00x**) | `15.02 us` (✅ **1.04x slower**) | `15.50 us` (✅ **1.07x slower**) | `14.50 us` (✅ **1.00x slower**)  |
| **`read (unvalidated)`**        | `N/A`                    | `N/A`                           | `71.95 ns` (✅ **1.00x**)        | `N/A`                            |
| **`deserialize (unvalidated)`** | `N/A`                    | `N/A`                           | `11.91 us` (✅ **1.00x**)        | `N/A`                            |

### mesh

|                                 | `alkahest`                | `bincode`                        | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `3.44 us` (✅ **1.00x**)   | `53.00 us` (❌ *15.39x slower*)   | `2.79 us` (✅ **1.23x faster**)   | `923.77 ns` (🚀 **3.73x faster**)  |
| **`read`**                      | `593.18 ns` (✅ **1.00x**) | `14.98 us` (❌ *25.25x slower*)   | `417.38 ns` (✅ **1.42x faster**) | `1.73 us` (❌ *2.92x slower*)      |
| **`deserialize`**               | `4.38 us` (✅ **1.00x**)   | `14.22 us` (❌ *3.25x slower*)    | `901.21 ns` (🚀 **4.86x faster**) | `947.68 ns` (🚀 **4.62x faster**)  |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `405.07 ns` (✅ **1.00x**)        | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `894.43 ns` (✅ **1.00x**)        | `N/A`                             |

### minecraft_savedata

|                                 | `alkahest`                | `bincode`                        | `rkyv`                         | `speedy`                         |
|:--------------------------------|:--------------------------|:---------------------------------|:-------------------------------|:-------------------------------- |
| **`serialize`**                 | `1.43 us` (✅ **1.00x**)   | `3.00 us` (❌ *2.11x slower*)     | `1.59 us` (❌ *1.11x slower*)   | `1.39 us` (✅ **1.03x faster**)   |
| **`read`**                      | `295.94 ns` (✅ **1.00x**) | `10.45 us` (❌ *35.31x slower*)   | `1.94 us` (❌ *6.54x slower*)   | `9.48 us` (❌ *32.04x slower*)    |
| **`deserialize`**               | `11.01 us` (✅ **1.00x**)  | `10.40 us` (✅ **1.06x faster**)  | `9.60 us` (✅ **1.15x faster**) | `9.50 us` (✅ **1.16x faster**)   |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `2.55 ns` (✅ **1.00x**)        | `N/A`                            |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `7.67 us` (✅ **1.00x**)        | `N/A`                            |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

