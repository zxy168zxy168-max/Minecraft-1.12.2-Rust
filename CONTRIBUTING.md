# Contributing to Minecraft-1.12.2-Rust

Contributions are welcome. The project was originally initiated and created by
`zxy168zxy168-max`; the canonical upstream repository is:

https://github.com/zxy168zxy168-max/Minecraft-1.12.2-Rust

## License for contributions

The repository's project-owned code is licensed under Apache-2.0. You may fork,
clone, build, modify, and redistribute it subject to that license.

Unless you explicitly state otherwise, a pull request or other contribution
intentionally submitted for inclusion in this project is licensed under the
same Apache-2.0 terms (inbound = outbound), consistent with Section 5 of the
Apache License. No separate CLA is currently required.

The root `NOTICE` file records project-origin attribution. Distributions and
derivative works must handle that NOTICE as required by Apache-2.0 Section 4(d).
Git history remains the primary record of individual contributor authorship.

## Source-fidelity requirement

This repository is a semantic/source-level Rust port of Minecraft Java Edition
1.12.2, not a look-alike reimplementation. Changes to vanilla behavior should
be justified against the corresponding Minecraft 1.12.2/MCP behavior or another
explicitly identified compatibility source. Do not replace known source logic
with approximate algorithms merely to make a feature appear to work.

For behavior-changing pull requests, please include:

- the original class/method or other authoritative behavior being ported;
- the reason the Rust implementation is semantically equivalent;
- tests or verification steps covering state transitions and edge cases;
- any intentional deviation, with the compatibility reason stated explicitly.

## Third-party material

Do not submit material that you do not have permission to license or distribute.
The repository license does not grant rights to Minecraft/Mojang/Microsoft
assets or source, MCP, OptiFine, shader/resource packs, or other third-party
material. Keep such dependencies and references within their applicable terms.

## Pull-request hygiene

Before submitting, run at least:

```text
cargo fmt --all -- --check
cargo check --release --all-targets
```

Where the change has dedicated tests, run those as well. Keep unrelated
formatting or generated-file churn out of focused pull requests.
