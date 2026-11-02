# vera20k-ares — external gameplay reference component

This component implements the currently ported VERA20K shield chain outside
the game engine. One reference component uses the
`vera:mod-sdk@0.1.0` interface; INI selects compatible shields, authored overload
and Ares weapon ammo costs, reload amounts and low-ammo weapon selection under
the same engine binary.

The code was moved from VERA20K's Rust implementation, retaining its source
references. It does not include Ares/Phobos C++ code or Mental Omega resources.
The original VERA20K contribution history and GPL licensing continue to apply.

This plugin is maintained in the independent `plugin-dev` project at
`plugins/vera20k-ares/`. Its `sdk/` contains versioned API/WIT development
dependencies and the packager; the runtime stays in the engine. The Cargo package is
`vera20k-ares`, its raw core artifact is `vera20k_ares.wasm`, and its publication
filename is `vera20k-ares.wasm`. The internal `vera.reference-gameplay` identity,
version 0.7.0, config_version=5 and state_version=1. API v1 and WIT are unchanged. The filename
does not claim complete Ares compatibility; the retained Phobos-compatible
shield and authored overload remain examples in this first plugin.

Build from this project's root using standard Cargo, as described in the
[development guide](../../sdk/dev-kit/README.zh-CN.md). API/WIT dependencies
are local SDK snapshots whose authoritative source and version are recorded in
`sdk/source.json`. Compilation needs no engine executable, source checkout,
game assets or Python runner. Game integration uses a compatible engine.

## What runs in the component

For a playable six-zone test map and vanilla/compatible/overload launch modes,
see the [Chinese lab guide](examples/sdk-lab/README.zh-CN.md).

- Each reached Rules pass reads the `ShieldTypes` catalog through the host's
  existing INI readers. The final resolved pass binds object and warhead names
  and reads global values. The engine does not interpret shield tags.
- `attach` constructs component-owned points and timers.
- `damage` returns a new component state and receiver damage. It calls the
  host's shared warhead numeric kernel rather than copying that algorithm.
- `tick` owns recovery, powered/temporal pause behavior and cleanup.
- Each plan also returns generic meter/tint presentation data.
- `weapon_resource` returns immutable weapon-cost decisions;
  `type_resource` returns immutable per-type reload, low-ammo weapon and local input policies.
  Both queries run at admission and are rebuilt from saved frozen configuration.
  The original ammo owner, reload clock and canonical weapon selector consume
  these policies; the component does not hold duplicate ammo state.

Code is grouped by domain and complete mechanism:

```text
src/features/
├─ defense/shield/           # configuration, state, damage and recovery
├─ units/manual_control.rs  # NoManualFire/Unload/Enter, NoSelfGuardArea
└─ weapons/
   ├─ ammo_cost.rs           # weapon Ammo cost
   ├─ reload.rs              # ReloadAmount / EmptyReloadAmount
   └─ empty_weapon.rs        # NoAmmoWeapon / NoAmmoAmount
```

Every feature uses the same `src/support/host.rs` facade and configuration
readers. Shared static query/policy types live in `sdk/api/src/resource.rs`;
`type_resource` adds no Host service or WIT import.

New matches with this component enable AresManualControl by default; the four
manual-action tags themselves still default to no. Set
`[VeraFeatures] AresManualControl=no` to opt out of this module.
See the [Chinese guide](MANUAL.zh-CN.md) and [INI example](examples/manual-control.ini).
These policies change local action admission; they do not add simulation order filters.

Persistent state and configuration are opaque, bounded host-managed bytes.
Every floating-point configuration value is encoded as its IEEE-754 bits.
No module-global mutable value is authoritative. The signed countdown timer
implementation is shared with the engine through `vera-mod-api::timer`.

The port retains the existing scope: idle/hit/break/respawn animations and
weapons, warhead attach/remove/break operations, type-conversion transfer and
the `SelfHealing.EnabledBy` consumer were not completed in the old owner and
are not newly claimed here. The retained original PIPS/PIPBRD presentation is
used; configurable alternative Pips.Background resources are not newly added.

## One build, INI-selected features

Install `wasm32-unknown-unknown`, then build from the `plugin-dev` root:

```sh
cargo build --locked -p vera20k-ares --target wasm32-unknown-unknown --release
```

Create the output parent directory, then package into a new child directory:

```sh
cargo run --locked --release -p vera-mod-pack -- target/wasm32-unknown-unknown/release/vera20k_ares.wasm dist/release-new --program vera20k-ares.wasm
```

The output directory must be new and its parent must exist. `--program` sets
the publication filename and declaration together; the generic packager retains
its `gameplay.wasm` default for invocations without this option. Reuse the same
`vera20k-ares.wasm` and `vera-mod.json` for each work; Cargo feature flags are no
longer used for gameplay selection. In 0.7.0, new matches enable AresAmmo,
AresReload, AresNoAmmoWeapon and AresManualControl without opt-in keys, so the
implemented original Ares tags work directly. Shield and Overload still default
to no. For example, add a compatible shield in a loaded Rules/map INI:

```ini
[VeraFeatures]
Shield=yes
Overload=no
```

The four Ares modules can be disabled independently for comparison:

```ini
[VeraFeatures]
AresAmmo=no
AresReload=no
AresNoAmmoWeapon=no
AresManualControl=no
```

These are VERA controls, not additional original Ares tags. Missing, empty or
invalid values retain the current layer's prior selection; an explicit no stays
disabled until a later valid yes. Overload requires Shield. Existing
`ShieldTypes`, `ShieldType`, weapon `Ammo` and
per-type `ReloadAmount`, `EmptyReloadAmount`, `NoAmmoWeapon`, `NoAmmoAmount`
settings retain their syntax. Layered selection
is frozen with the configuration and saved/checksummed; it cannot change during
a match. `examples/features.ini` shows the combined selection. The component
identity is `vera.reference-gameplay` 0.7.0, config_version=5, state_version=1.
The new defaults apply only to a new match's empty initial configuration.
Nonempty frozen configurations are not upgraded. Saved component bytes and
configuration retain their original behavior; restore also requires the current
work's component digest to match the saved digest exactly. Keep the old artifact
and its declaration for old saves. New defaults do not complete the remaining
Ares compatibility gaps.

`ReloadAmount` defaults to 1 and `EmptyReloadAmount` defaults to the normal
amount. The empty amount is selected only at current ammo exactly zero, even
without an `EmptyReload` interval. Its existing completion path applies signed
32-bit wrapping addition, stores zero for a negative sum, otherwise caps the
result at the type's capacity. Aircraft dock/radio reload is excluded; ordinary
infantry/buildings use the shared reload owner with Hospital/Armory exceptions.

`NoAmmoWeapon` defaults to -1. Every negative value disables this override;
every nonnegative signed 32-bit value is published unchanged, matching Ares
hook10047B08..10047B1D. The host applies it when the type capacity is
nonnegative and current signed ammo is at or below `NoAmmoAmount` (default 0).
This threshold is not a weapon-cost test. Slots 0/1 select primary/secondary;
slots 2..17 use the existing weapon array and live elite-slot fallback.

`WeaponN`/`EliteWeaponN` are read only in the `TurretCount>0` Rules branch;
`Weapon3` is index 2. The non-gattling gunner branch returns its current slot
before the NoAmmoWeapon hook, so it cannot be overridden. A type using those
higher slots normally needs `IsGattling=yes` as well as positive `TurretCount`.
Earlier DeployFire, occupied-building, missing primary/secondary, NeverUse,
null-target and open-transport selection returns retain priority. Selection
never searches other slots after an empty or unusable chosen weapon.

The host supports the fixed 0..17 array. Indices at or above 18 still reach the
policy unchanged, but host lookup safely returns no weapon; the original DLL
would read outside its arrays. This is a retained safety difference, not an
Ares configuration rejection. Building animation-delayed shots require the
matching host that retains the complete selected index through its delayed-fire
state and save/restore path; an older host can collapse higher indices to slot 0.
Version 0.6.0 changes no SDK fields or configuration/state schemas.

Aircraft may use this selection policy. Their weapon-0-specific approach,
strafe, ROF and projectile-course reads keep their original ownership. To fire
a free backup at zero ammo for a supported ground category, keep AresAmmo
enabled (the new-match default) and give that weapon `Ammo=0`. Aircraft retain their own release,
return and dock-reload mechanisms; weapon Ammo=0 does not make aircraft shots
free. See the [Ares resource guide](AMMO.zh-CN.md) and
[combined example](examples/resource-cycle.ini).

The packager uses the embedded WIT metadata to encode and validate a standalone
component; it never executes the guest or adds WASI adapters. A direct
`wasm32-wasip2` build is unsuitable because Rust std can introduce implicit WASI
imports. The runtime links only the SDK imports; no WASI filesystem, clock, entropy,
stdio or network import is available. Native unit tests exercise component
decisions, while the engine acceptance harness must execute the actual built
WASM artifacts and production INI/damage owners.

```sh
cargo test --locked -p vera20k-ares --lib
```

## Authored overload behavior

`[VeraFeatures] Overload=yes` enables this behavior in the same component. It increases heat by the
shield points actually consumed by a hit. Reaching HeatLimit suspends shield
absorption and recovery for Duration simulation frames. Cooling points are
removed on each eligible object AI visit. Temporal, chrono and limbo admission
retain their existing pause/skip semantics. All four recovery countdowns,
including the healing and respawn combat delays, remain paused during overload.
Hits may restart these countdowns, but they stay frozen until an eligible AI
visit ends overload; the existing power rules then apply. For example, overload
from frame 10 through 85 followed by a frame-11 hit resumes a 90-frame healing
countdown at 85, healing at 175. A twelve-frame combat delay moves it to 187.
This behavior is a VERA SDK example, not an Ares/Phobos compatibility
claim, and no engine host operation is specialized for it.

```ini
[VeraOverload]
HeatLimit=60
Duration=75
Cooling=1
```

With Overload=no, the component ignores that parameter section. `examples/shield.ini` uses only
original-game unit names and fictional shield data; it contains no artwork.
