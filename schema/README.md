# Alkahest schema version checks

`alkahest-schema` compares a trusted schema snapshot with the formulas compiled
from the current source. It fails when a published version changes silently,
including changes to any nested element's selected version. The library and CLI
run locally or in any CI; GitHub annotations and a composite action are optional.

## Export application formulas

Rust applications explicitly register their public formula families under stable
IDs. The exporter uses actual `Formula<VERSION>` metadata, including nested
formulas, `VERSIONS` tables and layout at every supported address width (1–16).
It records the exporting host's pointer width; compare snapshots produced for
the same pointer width.

Add `alkahest-schema` as a development dependency from this repository's `schema`
directory, and enable `alkahest`'s `proc` feature when using derives. Place an
exporter in your application's `examples/export_schema.rs`:

```rust
use alkahest::Formula;
use alkahest_schema::Snapshot;

#[derive(Formula)]
#[alkahest(version = 1)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Formula)]
#[alkahest(version = 2)]
struct Entity {
    id: u64,
    #[alkahest(version = 1)]
    position: Position,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os().nth(1).ok_or("expected output path")?;
    let mut snapshot = Snapshot::new();
    snapshot.add::<Position, 1>("game.position")?;
    snapshot.add::<Entity, 2>("game.entity")?;
    std::fs::write(output, snapshot.to_json()?)?;
    Ok(())
}
```

Register **each independently versioned reusable formula**, including children.
The parent snapshot includes the child's selected version and complete schema,
so a child-version bump requires a parent-version bump. Registering the child
separately also rejects changing its schema without its own version bump when
the parent version advances. Repeat this registration for independently
versioned descendants. The tool cannot infer a formula family's durable identity
from a Rust type name or from matching tree positions across different versions.

IDs such as `game.entity` belong to your protocol and must remain stable across
Rust module moves or type renames. Removing or renaming an existing registered
family fails the check. Add new families freely. Choose the local published
version as the `add` const parameter; it must not decrease relative to baseline.

The repository includes a runnable exporter:

```sh
cargo run -p alkahest-schema --example export -- target/current-schema.json
cargo install --path schema
alkahest-schema check ci/schema-baseline.json target/current-schema.json
```

For your application, invoke its exporter with the same features and target
configuration used by the protocol. Write JSON directly to a file, as above, so
shell redirection does not change its encoding. Check every supported target or
feature configuration that changes formula declarations, using a separate
baseline for each configuration.

## Version invariants

For every registered family, comparison enforces:

- The current local version is at least the baseline's local version.
- Every revision present in both snapshots has exactly the same schema. A local
  bump cannot rewrite a shared historical entry in `Formula::VERSIONS`.
- If the local version is unchanged, the entire nested schema is unchanged.
  This includes child version numbers, field and variant names and order,
  discriminant tags, lengths, primitive encodings, indirection, inhabitance and
  layout bounds at all 16 address widths.

Version equality is stricter than wire compatibility. Even a compatible schema
change needs a version bump. Use Alkahest's separate compatibility functions to
require backward, forward or both directions for a newly introduced revision.
An old revision may disappear from the available revision table when the local
version advances; any historical revision still available must remain immutable.
Keep an archive of release snapshots if you need to compare against revisions
absent from the immediately previous release.

Export and parsing fail closed for opaque schemas, unsupported or malformed
metadata, unsupported snapshot formats and schemas beyond the supported nesting
limit (including recursive graphs). Limits are 64 nested nodes, 256 registered
families, 256 revisions per family, 4,096 total schema nodes and 32 MiB of JSON.
Serialized snapshots are deterministic; they
do not persist pointers or callback addresses. The checker verifies declared
metadata and cannot prove that handwritten codecs obey it. Semantic changes with
identical metadata also require your own review and tests.

## CLI and library

```text
alkahest-schema check BASELINE CURRENT [--format text|json|github]
```

The CLI only reads its input files. It never creates or refreshes a baseline.
For paths that start with a dash, use `check --format json -- BASELINE CURRENT`.
Exit codes are stable:

| Code | Meaning |
| --- | --- |
| 0 | Version invariants passed |
| 1 | Version violations found |
| 2 | Invalid arguments, unreadable inputs or invalid snapshots |

`text` is the default. `json` emits a single object on stdout with `status`
(`passed`, `failed` or `error`), `violations` and `errors`. Each violation includes
the registered `id`, schema `path` and `message`. `github` emits safely escaped
workflow error annotations and preserves the same exit status. Both failed
comparisons and invalid input therefore fail a CI step.

You can embed the same check without spawning a process:

```rust,ignore
let baseline = alkahest_schema::Snapshot::from_json(&baseline_json)?;
let current = alkahest_schema::Snapshot::from_json(&current_json)?;
let violations = alkahest_schema::check(&baseline, &current)?;
assert!(violations.is_empty(), "{violations:#?}");
```

## Baseline lifecycle

Initialize the first baseline deliberately from the released or agreed protocol
and commit it. There is no automatic bootstrap mode: a missing baseline is an
error. During a change, export the proposed formulas and compare them against the
last trusted baseline. After the check passes, review and commit the exported
snapshot as the next baseline, or publish it as an immutable release artifact.

**CI must obtain its baseline from a trusted base commit or release artifact.**
Comparing against a baseline that the same pull request can replace allows a
silent schema change to replace its own expected result. Keep the baseline path,
exporter's registration set and checker invocation under review; no checker can
protect a gate that a change disables.

## GitHub Actions

The composite action at `.github/actions/schema-check` runs a preinstalled
executable. It does not download dependencies, build application code or update
snapshots. Inputs are passed through environment variables and quoted arguments:

```yaml
- uses: ./.github/actions/schema-check
  with:
    baseline: baseline.json
    current: current.json
    command: ./target/debug/alkahest-schema
```

`command` is one executable name or path, not a shell command with arguments.
The action requires Bash; GitHub-hosted Linux, macOS and Windows runners provide
it. On Windows, use the executable's `.exe` suffix when giving an explicit path.

This application example assumes a pinned Alkahest checkout at
`vendor/alkahest`, an `export_schema` application example, and a committed
`ci/alkahest-schema.json` baseline. Adjust paths for your repository:

```yaml
name: Formula versions
on: pull_request
permissions:
  contents: read
jobs:
  schema:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v6
        with:
          submodules: recursive
      - name: Install Rust
        run: rustup toolchain install nightly --profile minimal
      - name: Build checker
        run: cargo +nightly install --path vendor/alkahest/schema --root "$RUNNER_TEMP/schema-tools"
      - name: Read trusted baseline
        env:
          BASE_SHA: ${{ github.event.pull_request.base.sha }}
        run: |
          git fetch --no-tags --depth=1 origin "$BASE_SHA"
          git show "${BASE_SHA}:ci/alkahest-schema.json" > "$RUNNER_TEMP/schema-baseline.json"
      - name: Export proposed formulas
        run: cargo +nightly run --example export_schema -- "$RUNNER_TEMP/schema-current.json"
      - name: Check formula versions
        uses: ./vendor/alkahest/.github/actions/schema-check
        with:
          baseline: ${{ runner.temp }}/schema-baseline.json
          current: ${{ runner.temp }}/schema-current.json
          command: ${{ runner.temp }}/schema-tools/bin/alkahest-schema
```

Use `pull_request` with read-only permissions. Do not run proposed exporter code
with secrets or elevated permissions under `pull_request_target`. A missing
baseline in the base commit fails the fetch step rather than initializing an
empty comparison. For a release baseline, download the artifact identified by
your trusted release configuration instead of reading a PR-controlled path.

The repository's `schema-tool.yml` workflow tests this tool on Linux and Windows
and smoke-tests its example exporter and action. Applications must register and
gate their own protocol formulas.

GitHub references: [workflow commands and annotation escaping](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands),
[workflow contexts](https://docs.github.com/en/actions/reference/workflows-and-actions/contexts).
