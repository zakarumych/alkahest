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
| **`serialize`**                 | `292.20 us` (✅ **1.00x**) | `456.34 us` (❌ *1.56x slower*)   | `302.20 us` (✅ **1.03x slower**) | `284.21 us` (✅ **1.03x faster**)  |
| **`read`**                      | `324.09 us` (✅ **1.00x**) | `1.68 ms` (❌ *5.17x slower*)     | `462.96 us` (❌ *1.43x slower*)   | `1.55 ms` (❌ *4.79x slower*)      |
| **`deserialize`**               | `1.61 ms` (✅ **1.00x**)   | `1.65 ms` (✅ **1.02x slower**)   | `1.78 ms` (✅ **1.10x slower**)   | `1.54 ms` (✅ **1.05x faster**)    |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `8.26 us` (✅ **1.00x**)          | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `1.30 ms` (✅ **1.00x**)          | `N/A`                             |

### mesh

|                                 | `alkahest`                | `bincode`                       | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:--------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `385.04 us` (✅ **1.00x**) | `5.29 ms` (❌ *13.75x slower*)   | `323.78 us` (✅ **1.19x faster**) | `143.95 us` (🚀 **2.67x faster**)  |
| **`read`**                      | `39.58 us` (✅ **1.00x**)  | `2.11 ms` (❌ *53.24x slower*)   | `39.41 us` (✅ **1.00x faster**)  | `342.81 us` (❌ *8.66x slower*)    |
| **`deserialize`**               | `483.74 us` (✅ **1.00x**) | `1.79 ms` (❌ *3.70x slower*)    | `248.82 us` (🚀 **1.94x faster**) | `262.07 us` (🚀 **1.85x faster**)  |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                           | `39.44 us` (✅ **1.00x**)         | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                           | `254.86 us` (✅ **1.00x**)        | `N/A`                             |

### minecraft_savedata

|                                 | `alkahest`                | `bincode`                        | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `282.82 us` (✅ **1.00x**) | `501.87 us` (❌ *1.77x slower*)   | `342.94 us` (❌ *1.21x slower*)   | `290.76 us` (✅ **1.03x slower**)  |
| **`read`**                      | `37.71 us` (✅ **1.00x**)  | `1.42 ms` (❌ *37.66x slower*)    | `347.38 us` (❌ *9.21x slower*)   | `1.27 ms` (❌ *33.77x slower*)     |
| **`deserialize`**               | `1.44 ms` (✅ **1.00x**)   | `1.41 ms` (✅ **1.02x faster**)   | `1.48 ms` (✅ **1.03x slower**)   | `1.27 ms` (✅ **1.13x faster**)    |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `170.53 ns` (✅ **1.00x**)        | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `1.11 ms` (✅ **1.00x**)          | `N/A`                             |

## Smaller benchmark results

### log

|                                 | `alkahest`               | `bincode`                       | `rkyv`                          | `speedy`                         |
|:--------------------------------|:-------------------------|:--------------------------------|:--------------------------------|:-------------------------------- |
| **`serialize`**                 | `1.89 us` (✅ **1.00x**)  | `3.74 us` (❌ *1.98x slower*)    | `2.06 us` (✅ **1.09x slower**)  | `1.85 us` (✅ **1.02x faster**)   |
| **`read`**                      | `1.96 us` (✅ **1.00x**)  | `15.11 us` (❌ *7.70x slower*)   | `2.94 us` (❌ *1.50x slower*)    | `14.59 us` (❌ *7.44x slower*)    |
| **`deserialize`**               | `13.19 us` (✅ **1.00x**) | `15.20 us` (❌ *1.15x slower*)   | `14.73 us` (❌ *1.12x slower*)   | `14.47 us` (✅ **1.10x slower**)  |
| **`read (unvalidated)`**        | `N/A`                    | `N/A`                           | `72.12 ns` (✅ **1.00x**)        | `N/A`                            |
| **`deserialize (unvalidated)`** | `N/A`                    | `N/A`                           | `11.83 us` (✅ **1.00x**)        | `N/A`                            |

### mesh

|                                 | `alkahest`                | `bincode`                        | `rkyv`                           | `speedy`                          |
|:--------------------------------|:--------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`serialize`**                 | `3.44 us` (✅ **1.00x**)   | `52.44 us` (❌ *15.26x slower*)   | `2.82 us` (✅ **1.22x faster**)   | `920.21 ns` (🚀 **3.74x faster**)  |
| **`read`**                      | `403.72 ns` (✅ **1.00x**) | `15.00 us` (❌ *37.16x slower*)   | `413.98 ns` (✅ **1.03x slower**) | `1.74 us` (❌ *4.32x slower*)      |
| **`deserialize`**               | `2.07 us` (✅ **1.00x**)   | `14.20 us` (❌ *6.86x slower*)    | `903.62 ns` (🚀 **2.29x faster**) | `984.31 ns` (🚀 **2.10x faster**)  |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `406.36 ns` (✅ **1.00x**)        | `N/A`                             |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `912.53 ns` (✅ **1.00x**)        | `N/A`                             |

### minecraft_savedata

|                                 | `alkahest`                | `bincode`                        | `rkyv`                         | `speedy`                         |
|:--------------------------------|:--------------------------|:---------------------------------|:-------------------------------|:-------------------------------- |
| **`serialize`**                 | `1.41 us` (✅ **1.00x**)   | `3.00 us` (❌ *2.13x slower*)     | `1.63 us` (❌ *1.16x slower*)   | `1.39 us` (✅ **1.01x faster**)   |
| **`read`**                      | `293.30 ns` (✅ **1.00x**) | `10.43 us` (❌ *35.55x slower*)   | `1.63 us` (❌ *5.56x slower*)   | `9.51 us` (❌ *32.43x slower*)    |
| **`deserialize`**               | `9.18 us` (✅ **1.00x**)   | `10.45 us` (❌ *1.14x slower*)    | `9.30 us` (✅ **1.01x slower**) | `9.50 us` (✅ **1.04x slower**)   |
| **`read (unvalidated)`**        | `N/A`                     | `N/A`                            | `2.55 ns` (✅ **1.00x**)        | `N/A`                            |
| **`deserialize (unvalidated)`** | `N/A`                     | `N/A`                            | `7.68 us` (✅ **1.00x**)        | `N/A`                            |


---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

