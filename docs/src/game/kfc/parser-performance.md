# Resource reader performance and KFC4 writer checks

This analysis uses baseline commit `618bd0e` and the matching Enshrouded 1.0 client and dedicated server packages installed on 2026-10-16. The test machine has an AMD Ryzen 5 5600X, 12 logical processors, 32 GiB RAM, and Rust 1.92.0. SQL is not involved: the expensive operation is repeatedly decompressing resource chunks with Zstandard.

## Why the baseline regressed

The baseline retains at most 32 MiB of decompressed chunks **per cursor**. Most chunks contain 8 MiB, so this usually holds four chunks. The CLI collects resources in a `HashSet` and dispatches individual IDs through a shared `Mutex<Vec<_>>`. That order does not follow file offsets. Workers therefore revisit evicted chunks and decompress them again. Different workers can also decompress and retain independent copies of the same chunk.

Raising the old per-cursor limit alone increases retained memory with the worker count. Sorting resources alone improves locality, but adjacent workers can still load the same chunk. The implementation addresses both causes.

## Implemented approach

Each `KFCReader` owns one lazy cache entry per chunk. A `OnceLock<Box<[u8]>>` retains successfully decompressed data until the reader is dropped. A mutex guards only the first load of that individual chunk. Cache hits do not take a mutex, different chunks can load concurrently, and failed loads leave the entry available for retry. Decompressed sizes are checked against the index before caching.

This deliberately permits a larger cache than 32 MiB. Loading the whole package retains its entire decompressed resource stream once per reader, shared by all its cursors. It has no fixed memory cap independent of package size. Per-worker compressed input and resource output buffers still consume additional memory. A strict eviction limit would permit repeated decompression and is therefore a different performance tradeoff.

The CLI sorts selected resource IDs by offset, then workers claim ascending batches through an atomic counter. Each batch contains at most 256 IDs. Several batches per worker preserve load balancing without locking the queue for every resource. The shared cache handles chunks used by more than one batch.

For ordered, non-overlapping chunk tables, binary search replaces two linear scans per resource. A linear fallback preserves lookup support for other table layouts. The final chunk is located using the resource's exclusive end minus one, avoiding an unnecessary read of the next chunk when a resource ends exactly at a boundary. Empty resources and invalid decompressed ranges have explicit handling.

Looking up a key in an empty `StaticMap` now returns `None` instead of dividing by a zero bucket count. The CLI's empty and unknown-selection test exercises this case.

## Measurements

Five release builds isolate the alternatives: baseline `618bd0e`, the former unbounded per-cursor cache, the bounded cache with CLI batching, the shared cache with the original CLI scheduling, and the combined implementation. Reader-only timings are separate from complete CLI JSON export. Runs use the same installed files and matching extracted reflection data. No builds or package tests run concurrently with the benchmarks.

All measurements use eight workers and a warm operating-system file cache. Every observation starts a fresh process. Complete CLI exports run `kfc-parser --threads 8 unpack -g GAME_DIR --stdout`; their output is drained asynchronously to `Stream.Null`. They include descriptor conversion, JSON serialization and output buffering, but no directory/file creation. Peak memory is the process's kernel-reported peak working set, sampled every 20 ms.

| Complete client export | Runs | Elapsed seconds | Peak process MiB |
| --- | ---: | ---: | ---: |
| Baseline 32 MiB LRU per cursor | 1 | 236.135 | 2,006.8 |
| Former unbounded cache per cursor | 3 | 9.096 | 4,195.6 |
| Bounded cache with CLI batching | 3 | 6.943 | 1,971.1 |
| Shared cache with original CLI scheduling | 3 | 7.405 | 2,141.3 |
| Shared cache with CLI batching and redundant JSON copy removed | 3 | **6.319** | **1,993.8** |

Values for three runs are medians. The full baseline completed successfully but took 236 seconds, so it was not repeated. The combined implementation is 37.37 times faster than that observed baseline and takes 30.5% less time than the unbounded reference, with 52.5% less peak memory. Its three export times were 5.644, 7.855 and 6.319 seconds. The runtime ranges of the faster strategies overlap, so the smaller batching advantage is not a universal guarantee.

The controlled reader test uses the same 2,048 selected resources (7,829,621 bytes), hash-index order, eight workers, and three fresh processes per implementation:

| Reader only, same subset | Median reader seconds | Median peak process MiB |
| --- | ---: | ---: |
| Baseline 32 MiB LRU | 4.618 | 419.4 |
| Former unbounded per-cursor cache | 0.929 | 2,568.5 |
| Shared reader cache | **0.180** | **486.1** |

This isolates a 25.62-fold reader improvement against the bounded baseline and a 5.16-fold improvement against the unbounded reference. The bounded baseline uses less memory on this subset, but pays for repeated decompression.

Reading the full client in hash-index order processes all 91,812 resources and 481,007,781 resource bytes. The shared reader takes a median 0.212 seconds versus 1.286 seconds with unbounded per-cursor caching. Peak process memory falls from 3,424.3 to 530.2 MiB, an 84.5% reduction. The package has 58 chunks and 481,531,128 uncompressed chunk bytes (459.22 MiB). That is the cache payload ceiling for this reader. The roughly 2 GiB CLI peak also includes parsed descriptors, JSON and other buffers and must not be described as the cache size.

For strictly sequential workloads, a bounded cache remains a useful alternative: the baseline reader in offset order completes the full package in 0.514 reader seconds at 260.6 MiB peak. This uses a different access order and is not used in the random-access speed ratios. The shared cache was selected to cover both arbitrary mod-loader access and bulk export without repeated successful decompressions.

The new reader also reads all 42,599 KFC3 server resources (459,801,643 bytes). The earlier audit's KFC3 client backup location was unavailable during this run. This is a raw KFC3 server read check, not a new full KFC3 reflection audit.

The 32 individual benchmark process results, executable/source hashes and summary remain under `target/perf-review`. `measurements.jsonl` records individual runs and `summary.json` records the grouped results.

`bench_reader` reads and copies resource bytes without JSON conversion or output-file writes. Its `index` order models hash-index access with a shared work counter, but does not reproduce the CLI's randomized `HashSet` order. Its `offset` order gives each worker one contiguous range, whereas the CLI uses smaller dynamic batches. Every pass creates fresh cursors on the same reader. Both the operating system's file cache and the new reader cache can survive between passes. These timings must not be presented as cold-disk results.

## Verified writer defects

Previously, finalizing an incremental writer without writing any resources or content appended an empty compressed chunk. It also rounded the existing content-container count up to a power of two. Current KFC4 packages do not require such a count: the tested client has 55 containers and the server has 10.

The writer now skips empty chunks and preserves the existing container directory when no new content containers are written. It explicitly flushes the resource stream before writing the index and propagates flush failures. Two regression tests cover empty submission and unchanged incremental finalization.

The actual installed packages were copied into `target/writer-audit` for verification. No installed game files were written.

| Unchanged incremental finalization | Original | Previous writer | Fixed writer |
| --- | --- | --- | --- |
| Server containers / chunks | 10 / 42 | 16 / 43 | 10 / 42 |
| Client containers / chunks | 55 / 58 | Not measured | 55 / 58 |

All 43,939 server and 91,812 client resource entries retain their original offsets and sizes after the fixed no-op operation. The complete original and resulting `.kfc_resources` files have matching SHA256 values:

| Package | SHA256 before and after |
| --- | --- |
| Server | `0D448AD81E9C4D6925978362F59F04F5D2C610A5F6260BA727FCA53935D7F506` |
| Client | `CF47DB07AE2A959C22E88C661C8E7318A283E3EA063B7A74570ADA6A5AF9467B` |

These checks verify no-op writer behavior. They do not prove that the game accepts a complete deserialize/serialize/repack operation.

## Validation performed

All 22 local unit tests pass. They include concurrent first loads sharing one allocation, retry after a failed load, exact chunk boundaries, resources spanning chunks, cache reuse by fresh cursors, parallel CLI work distribution, and unchanged writer finalization.

All five existing package tests pass against both current matching installations. They validate every resource and content entry, normal and human-readable byte roundtrips, a JSON roundtrip, and executable reflection extraction. Client resource count is 91,812 and server resource count is 43,939. `cargo check --workspace` also passes, with existing warnings in unchanged modules.

The new ignored `cli-legacy/tests/unpack_repack.rs` test runs the actual `unpack --stdout | repack --stdin` process pipeline on temporary copies. It passes for both the full client and full dedicated server. Afterwards it checks every resource byte, ID set, package format/version, content entry, container metadata, bundle hash and bundle membership. It also requires every resource offset to point into the newly appended data, so a repack that silently skipped a resource cannot pass by retaining its old entry. No original package is written and no content container is copied or modified. Successful temporary fixtures are removed; failures retain the fixture and process logs.

These complete pipeline checks establish data preservation through the CLI and writer using this parser. They do not validate engine-specific package requirements or constitute a game launch.

## KFC4 compatibility remains a separate question

`keen::ResourceContentReference` is already supported by generic struct reflection. Both matching executables describe its 32-byte layout: `hash` at byte 0, `category` at byte 16, `bundleId` at byte 20, and `sortKey` at byte 24. The absence of a hardcoded Rust type name is not evidence of missing parser support. Lua helpers accepting a `ContentHash` currently require the reference's inner `.hash`.

All resource-bundle internal hashes match their corresponding executable. The reserved fields of every tested resource ID are zero. Neither observation explains the reported game failure.

Two format differences remain unconfirmed hypotheses for engine-specific investigation. Incremental writes append after the original final partial chunk, producing subsequent chunk offsets that are not multiples of 8 MiB. Also, the original large `VoxelTemperatureResource` stores size 11,105,420 while its actual payload is 44,659,852 bytes, whereas the writer stores the full size. No speculative alignment or 24-bit masking change has been made. No engine startup comparison was performed, and the reported game crash is not claimed fixed.

## Reproduce

Run the following PowerShell commands from the repository root. These tests read the selected installed package. Use a matching executable and index.

```powershell
cargo build --release -p kfc-base --example bench_reader
$env:GAME_DIR = 'C:\Program Files (x86)\Steam\steamapps\common\Enshrouded'
$env:GAME_FILE_NAME = 'enshrouded'
& .\target\release\examples\bench_reader.exe $env:GAME_DIR --file-name $env:GAME_FILE_NAME --threads 8 --order index --passes 1
& .\target\release\examples\bench_reader.exe $env:GAME_DIR --file-name $env:GAME_FILE_NAME --threads 8 --order offset --passes 1
& .\target\release\examples\bench_reader.exe $env:GAME_DIR --file-name $env:GAME_FILE_NAME --threads 8 --order index --limit 2048 --passes 1
cargo test --release -p kfc -- --ignored --test-threads=1
cargo test --release -p cli-legacy --test unpack_repack -- --ignored
```

For the dedicated server, repeat after changing the environment variables:

```powershell
$env:GAME_DIR = 'C:\Program Files (x86)\Steam\steamapps\common\EnshroudedServer'
$env:GAME_FILE_NAME = 'enshrouded_server'
```

Repeat the benchmark command in three fresh processes to reproduce the median method. Use `--passes 2` to inspect reuse across fresh cursors on an already populated reader, or `--threads 1` for a single-worker comparison. A baseline checkout needs an identical copy of `crates/kfc-base/examples/bench_reader.rs`, because the example was added after `618bd0e`. Build each implementation in a separate target directory to avoid mixing artifacts.
