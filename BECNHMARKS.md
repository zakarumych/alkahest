# Benchmarks

This is results of the <https://github.com/djkoloski/rust_serialization_benchmark>
updated to use `alkahest 0.2.0-rc.8`.
After `0.2.0` is released, PR will be made to the benchmark repo.

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [log](#log)
    - [mesh](#mesh)
    - [minecraft_savedata](#minecraft_savedata)

## Benchmark Results

### log

|                                 | `alkahest`                | `bincode`                        | `rkyv`                                | `speedy`                         | `dlhn`                            |
|:--------------------------------|:--------------------------|:---------------------------------|:--------------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `341.08 us` (✅ **1.00x**) | `461.92 us` (❌ *1.35x slower*)   | `303.27 us` (✅ **1.12x faster**)      | `295.02 us` (✅ **1.16x faster**) | `577.52 us` (❌ *1.69x slower*)    |
| **`access`**                    | `1.28 ns` (✅ **1.00x**)   | `N/A`                            | `455.35 us` (❌ *356791.25x slower*)   | `N/A`                            | `N/A`                             |
| **`read`**                      | `330.13 us` (✅ **1.00x**) | `N/A`                            | `464.63 us` (❌ *1.41x slower*)        | `N/A`                            | `N/A`                             |
| **`deserialize`**               | `1.83 ms` (✅ **1.00x**)   | `1.67 ms` (✅ **1.09x faster**)   | `1.81 ms` (✅ **1.01x faster**)        | `1.51 ms` (✅ **1.21x faster**)   | `2.03 ms` (❌ *1.11x slower*)      |
| **`access (unvalidated)`**      | `N/A`                     | `N/A`                            | `0.83 ns` (✅ **1.00x**)               | `N/A`                            | `N/A`                             |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `8.26 us` (✅ **1.00x**)               | `N/A`                            | `N/A`                             |
| **`update`**                    | `N/A`                     | `N/A`                            | `8.15 us` (✅ **1.00x**)               | `N/A`                            | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `1.33 ms` (✅ **1.00x**)               | `N/A`                            | `N/A`                             |

### mesh

|                                 | `alkahest`                | `bincode`                       | `rkyv`                           | `speedy`                         | `dlhn`                           |
|:--------------------------------|:--------------------------|:--------------------------------|:---------------------------------|:---------------------------------|:-------------------------------- |
| **`serialize`**                 | `401.39 us` (✅ **1.00x**) | `5.31 ms` (❌ *13.22x slower*)   | `323.68 us` (✅ **1.24x faster**) | `179.92 us` (🚀 **2.23x faster**) | `6.10 ms` (❌ *15.20x slower*)    |
| **`access`**                    | `1.29 ns` (✅ **1.00x**)   | `N/A`                           | `10.71 ns` (❌ *8.30x slower*)    | `N/A`                            | `N/A`                            |
| **`read`**                      | `59.80 us` (✅ **1.00x**)  | `N/A`                           | `39.89 us` (✅ **1.50x faster**)  | `N/A`                            | `N/A`                            |
| **`deserialize`**               | `552.88 us` (✅ **1.00x**) | `1.70 ms` (❌ *3.08x slower*)    | `303.72 us` (🚀 **1.82x faster**) | `297.27 us` (🚀 **1.86x faster**) | `3.80 ms` (❌ *6.87x slower*)     |
| **`access (unvalidated)`**      | `N/A`                     | `N/A`                           | `0.83 ns` (✅ **1.00x**)          | `N/A`                            | `N/A`                            |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                           | `39.89 us` (✅ **1.00x**)         | `N/A`                            | `N/A`                            |
| **`update`**                    | `N/A`                     | `N/A`                           | `103.19 us` (✅ **1.00x**)        | `N/A`                            | `N/A`                            |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                           | `286.05 us` (✅ **1.00x**)        | `N/A`                            | `N/A`                            |

### minecraft_savedata

|                                 | `alkahest`                | `bincode`                        | `rkyv`                                | `speedy`                         | `dlhn`                            |
|:--------------------------------|:--------------------------|:---------------------------------|:--------------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `395.92 us` (✅ **1.00x**) | `503.42 us` (❌ *1.27x slower*)   | `341.98 us` (✅ **1.16x faster**)      | `307.95 us` (✅ **1.29x faster**) | `597.17 us` (❌ *1.51x slower*)    |
| **`access`**                    | `0.85 ns` (✅ **1.00x**)   | `N/A`                            | `349.57 us` (❌ *413163.06x slower*)   | `N/A`                            | `N/A`                             |
| **`read`**                      | `38.28 us` (✅ **1.00x**)  | `N/A`                            | `350.71 us` (❌ *9.16x slower*)        | `N/A`                            | `N/A`                             |
| **`deserialize`**               | `1.69 ms` (✅ **1.00x**)   | `1.42 ms` (✅ **1.19x faster**)   | `1.48 ms` (✅ **1.14x faster**)        | `1.25 ms` (✅ **1.35x faster**)   | `1.88 ms` (❌ *1.11x slower*)      |
| **`access (unvalidated)`**      | `N/A`                     | `N/A`                            | `0.83 ns` (✅ **1.00x**)               | `N/A`                            | `N/A`                             |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `173.07 ns` (✅ **1.00x**)             | `N/A`                            | `N/A`                             |
| **`update`**                    | `N/A`                     | `N/A`                            | `336.91 ns` (✅ **1.00x**)             | `N/A`                            | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `1.11 ms` (✅ **1.00x**)               | `N/A`                            | `N/A`                             |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

