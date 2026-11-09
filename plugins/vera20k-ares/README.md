# vera20k-ares — external gameplay reference component

Version 0.37 / config26 adds optional common SuperWeapon targeting declarations, physical range retention and ordinary-host AI/cameo integration. Existing simulation owners execute selection, activation, charge, settlement and restoration. Historical absent declarations retain their prior route; matched host support is required. See the [Chinese configuration guide](SUPERWEAPON.zh-CN.md). Complete Ares compatibility remains open.

Version 0.35 / config24 adds optional `MatchRulesPolicy.shrapnel_weapon_assignment`. Fresh configuration and real reset boundaries select it; an older nonempty frozen cohort omits it. `ShrapnelWeapon` retains the actual Weapon allocation, which the installed child keeps through Fire and detonation for Range and RadLevel. Its bare constructor travel budget stays INT_MAX, without a Weapon ProjectileRange copy. This is an admission-time declaration requiring the paired host, with no per-frame Wasm call. Validation covers real-component firing, fragment flight, save/restore and receivers; complete Ares compatibility remains open.

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
filename is `vera20k-ares.wasm`. The internal `vera.reference-gameplay` identity is retained. Current source is
version 0.37.0, config_version=26 and state_version=1. API v1 and WIT are unchanged. The filename
does not claim complete Ares compatibility; the retained Phobos-compatible
shield and authored overload remain examples in this first plugin.

Build from this project's root using standard Cargo, as described in the
[development guide](../../sdk/dev-kit/README.zh-CN.md). API/WIT dependencies
are local SDK snapshots whose authoritative source and version are recorded in
`sdk/source.json`. Compilation needs no engine executable, source checkout,
game assets or Python runner. Game integration uses a compatible engine.

Current development candidate is 0.35.0, configuration version 24 and entity state
version 1. The 0.7.0 details below describe the retained lab delivery. The
startup-resource increment lives in `src/features/resources/startup.rs` and uses
optional `startup_resources` queries before Rules configuration. It returns ordered
Include/audio/CSF recipes without Host calls or opaque gameplay state; existing
host parsers and catalogs consume them. It requires a matching newer host and
early resource visibility. See the [Chinese development guide](../../sdk/dev-kit/README.zh-CN.md#启动资源)
for admission, recovery and validation limits. The overall migration remains open.

The shared faction/gameplay increment lives in `src/features/factions/` and
uses the separate optional `eva_catalog`, `configure_country_read`,
`configure_faction_pass` and `faction_resource` capabilities. One initialization
and parent-attempt history feeds existing host gameplay owners. `AresFaction`
defaults on for new configurations; its actual Country prelude selects the reader,
while the earlier fixed EVA layout remains separate. Real-component/save,
consumer, application, legacy and retail loading checks passed for this bounded
increment. The 0.17 candidate connects live Country menu/flag and early/late loading
presentation plus optional `loading_theme_resource`. Version 0.18 connects sidebar,
EVA SetSide, message colors, GraphicalText and score consumption through the same
frozen declarations and original presentation owners. Actual-component, legacy and real-retail
loading checks passed for the bounded Country presentation chain; see the [Chinese guide](../../sdk/dev-kit/README.zh-CN.md#阵营共享配置与玩法).

Version 0.32 adds the independent `features/projectiles/ballistic_scatter.rs`
domain. It composes optional `ballistic_scatter` with Airburst in the same
`projectile_resource` response without coupling either domain's ownership.
Fresh configuration and real reset boundaries own the new domain; nonempty old
configuration omits it. Physical Rules passes read Min then Max from retained
Projectile history through the host's canonical `ares-nullable-leptons` reader.
Absent limits remain None; valid zero remains an explicit override. No guest
parser, clamp, live registry query or per-frame Wasm call is introduced.

The bounded host path is ordinary FireAt with Inaccurate and Arcing, including
plain and FlakScatter-without-Inviso branches. The existing launch owner resolves
unset global defaults and owns RNG and coordinates; Bullet::Fire's additional
Inviso+FlakScatter placement scatter is outside these two keys. Config22/state1
keeps API v1, WIT and Hook identities unchanged but needs the new policy field
and nullable reader. Candidate acceptance follows the paired host checks.

Version 0.31 adds `features/projectiles/airburst.rs` and the optional static
`projectile_resource` hook. Completed physical Rules passes read AirburstSpread,
RetargetAccuracy, Splits, RetargetSelf and nullable AroundTarget from the actual
Projectile body history. Host double readers supply raw binary64 bits without
guest parsing or clamping. Constructor defaults are spread 1.5, accuracy 0,
Splits false and RetargetSelf true; absent AroundTarget resolves to final Splits.
Final binding retains these declarations without opening a physical registry.

Fresh configuration and real reset boundaries own the new airburst domain and
select `ares_replacement`; nonempty old configuration keeps the earlier
`unsupported_replacement` tail and omits the new resource response. The domain
requires range/firing ownership for child budgets and the host's admitted
Warhead relationship bindings. Targets, RNG, constructors,
flight, detonation and persistence remain with the existing host owners, with no
per-frame Wasm call. Config21/state1 keeps API v1 and WIT unchanged but needs the
new hook and scoped reader services. This candidate does not claim complete AR268
compatibility or acceptance of every exceptional value and explosion entrypoint;
the paired host checks determine the supported production scope.

Version 0.30 adds the independent `features/weapons/projectile_range.rs` domain.
It composes optional `projectile_range` into `weapon_firing_resource` and selects
the host's travel-budget runtime once through `match_rules.projectile_fuel`.
Fresh configuration and actual reset boundaries own the domain; nonempty older
configuration, including 0.29 firing configuration, remains unowned. There is no
extra INI enable flag. Completed physical Rules passes use the retained Weapon
history and canonical host `ares-leptons` reader; final binding only freezes the
result. The constructor default is exactly 100000 leptons (390.625 cells), and
signed host results are retained without a positive clamp.

That increment covers ordinary FireAt's initial weapon budget and shared flight
and exhaustion detonation for `Ranged=yes`. Bare/shrapnel constructors keep an
INT_MAX initial budget; other special creators and the replacement Airburst fan
are not claimed complete. The host owns the live counter, motion, detonation,
save and checksum, with no per-tick Wasm call or duplicate guest counter. Missing
or false `projectile_fuel` selects retail behavior without fuel, including older
guests. Config20/state1 keeps API v1 and WIT unchanged but requires the matching
host fields and reader. Preserve exact component/engine pairs for full snapshots;
candidate acceptance is tracked with the paired host checks.

Version 0.29 adds optional `weapon_firing_resource` queries. The component
selects directional muzzle-animation bins and reads Ares `ApplyDamage` through
the shared nullable-bool reader over the retained Weapon body history. A later
same-ID Projectile body cannot replace that source. Completed physical Rules
passes update the declaration; final binding retains it without rereading. No
extra enable flag is required. The host retains
animation identities, coordinates, damage stages and the original AI estimate;
`ApplyDamage` changes ordinary FireAt damage, not that separate estimate. The
new capability requires its matching host. Older guests keep their original
request shape and old frozen configurations do not acquire the new domain.

The paired 0.29 host has a new full-snapshot layout. The retained 0.28 guest
loads and saves/resumes under the new host, while complete snapshots written
by the retained older host require that original host/component pair. Entity
state version 1 does not imply a stable full-engine snapshot schema.

Version 0.28 moves the existing SuperWeapon extensions into
`features/superweapons/`. `configure_super_read` runs at the original Rules
visit, preserving first-visit initialization and SW.Warhead reference allocation;
`superweapon_resource` publishes one immutable batch for providers, availability,
initial/virtual charge, payments, GenericWarhead, UnitDelivery and presentation.
Authors use the existing Ares INI keys without an extra enable tag. Shared Host
readers and fixed audio catalogs remain authoritative; the original engine owns
charge/history, money, two-stage selection and delivery queues, without per-frame
Wasm mutations.

The 0.28 candidate uses config18/state1 and the same API v1/WIT name, with a matching
newer host required for the new hooks. Absent declarations preserve the host
profile; an explicit default policy selects stock behavior. Restore uses the
exact saved component and frozen configuration; preserve old host/component
pairs for old saves. Scope remains the previously implemented subset, excluding
full charge-drain, additional payload kinds and additional AI targeting modes.

Version 0.27 adds the pure `sound_sample_admission` declaration before the
fixed SOUNDMD catalog is first read. The reference selects unindexed sample
admission without an INI switch; absence keeps the host profile, while explicit
false requires indexed names when an index exists. The original catalog, sample
selection and WAV decoder remain host-owned. Reacquiring assets retains the same
frozen catalog. An equal raw INI hash does not prove equal admission semantics;
exact component identity and the fixed source recipe remain authoritative.
Config17/state1/API v1/WIT are unchanged; the new declaration requires a matching
host. Audio-device output and cross-platform execution were not tested.

Version 0.26 binds the ordinary Warhead House filter and existing Airburst
boundary through `match_rules.warhead_house_filter` and `airburst_tail`.
The reference declares the House filter independently of EMP activation; explicit
true without effective Warhead relationships is rejected before publication.
`unsupported_replacement` preserves the unimplemented replacement-fan boundary
while parent damage still executes. Absence keeps the host profile; explicit
false / `retail_fixed_nine` selects stock behavior. Actual component damage,
formal firing, saved rebinding, legacy controls and real retail loading passed.
Config17/state1/API v1/WIT are unchanged and require a matching host. This does
not implement the replacement fan or claim GUI, audio-device, network,
cross-platform or performance validation.

Version 0.25 adds the pure `prism_support_defaults` declaration before the first
Rules read. The reference component selects the existing default-unit correction
without an INI switch; absence keeps the host profile and explicit false selects
retail behavior. A present General section with an absent modifier key retains
the intended percentage across layers. Numeric readers, support counts and firing
remain with their existing owners. Actual-component layer/save/delayed-shot, legacy
and retail loading checks passed. Config17/state1/API v1/WIT stay unchanged, but a
matching host is required. The native rules save layout changed: retain old host
and component pairs for old saves. This adds no PrismForwarding; no GUI or
cross-platform run was performed.

Version 0.24 adds the pure `spawn_identity` declaration for the existing
CnCNet spawn.ini Country and campaign Side width gates. The component selects
extended Country identities and byte Side metadata before gameplay configuration.
The single parser reads the request once; the original Ready transition approves
it against the admitted process recipe. Absent declarations retain the host
profile, while explicit false selects retail limits even in a legacy host.
Country allocation, map/peer/color checks and campaign record0 remain with their
existing owners. Actual component-to-House/Side/MCV, legacy and seven retail-mode
checks passed. Community decoding and the shared approval helper were tested
without installing the global Snapshot; no window/network startup or cross-platform
run was performed. The independent network descriptor keeps its original bounds.
Config17/state1 and API v1/WIT stay unchanged; a matching newer host is required.

Version 0.23 adds one `type_sound_resource` catalog for
`Promote.VeteranSound` / `Promote.EliteSound`. Reached Techno reads retain
inherit-global, explicit silence and registered-cue states. The host borrows the
process-selected SOUNDMD registry, validates its source hash and native Type
identities, then binds the catalog once; promotion state, positional SFX and EVA
remain in their existing owners. Registered literal `none` remains a cue, while
silence suppresses only SFX. Absence keeps the old profile; an empty catalog
selects global cues even in a legacy host. Actual AI-to-dispatcher, save/continued
promotion, changed-source rejection, legacy, shared PCX GPU and seven retail-mode
checks passed. Playback on an audio device was not tested. This needs a matching
host; API v1/WIT do not imply compatibility with older hosts. See the
[type sound API](../../sdk/api/src/audio.rs).

Version 0.22 adds one `cameo_resource` catalog for normal, promoted and
SuperWeapon PCX declarations. The component retains names at the actual physical
Rules/ART read points; the host keeps the shared filename reader, PCX/SHP decoder,
60x48 logical canvas and atlas. Missing promoted art falls back to normal PCX,
then the existing SHP path; a fully transparent PCX stays selected. No new INI
switch or per-icon guest call is required. An absent catalog preserves the old
profile; an explicit empty catalog selects stock. Actual-component GPU, exact
save restoration, legacy and seven retail-mode checks passed, including the
previous component. Formal resource binding follows the completed scenario and
common TypeReset; intermediate host configuration is not proof that native
SuperWeapon extensions survive ClearScene. See the [catalog API](../../sdk/api/src/cameo.rs).
This remains a bounded baseline migration, not full Ares or native pixel parity.

Version 0.21 declares both `match_rules.anim_damage` selectors independently
of feature INI switches. Existing animation/attached-Techno House and ART
Warhead inputs feed the original accumulator and area receiver. Current, absent,
previous and explicit-off components passed actual AnimAI checks; legacy and
targeted retail loading passed too. Snapshot fields and Program adoption were
checked; this does not claim full restored AnimAI replay or all animation
extensions. Weapon= and Damage.Delay= remain separate.

Version 0.20 declares the two installed `match_rules.building_presentation`
selectors independently of feature INI switches. Retained ART readers, palette
decoding and AnimRemap remain host-owned; final binding refreshes the existing
catalog before constructors. Building SHP/MAKE palettes and surviving DestroyAnim
colors passed actual-component, save, indexed GPU, legacy and retail controls.
Animation damage ownership and further building/voxel extensions are separate.

Version 0.19 declares `long_list_readers = Extended2048` before gameplay
configuration. This installed fix covers only House Allies and the two Techno
prerequisite readers, independently of feature INI switches. General buffers and
the physical INI lexer retain their existing limits. Actual cold/mode/map,
directed-alliance, saved-state and lost-lease controls passed with a matching host.

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
├─ defense/                 # shield, overload and armor
├─ units/                   # manual controls, effects, EMP, lights and conversion
├─ weapons/                 # ammo cost, reload and low-ammo weapon choice
├─ production/              # eligibility, factories and country plans
├─ buildings/               # custom foundations
├─ houses/                  # survival policy
├─ factions/                # Side/Country gameplay and presentation
├─ resources/               # startup resources and loading themes
├─ presentation/            # cameos and promotion sounds
└─ superweapons/            # read history, targeting and frozen declarations
```

Every feature uses the same `src/support/host.rs` facade and configuration
readers. Shared query/policy types live in `sdk/api/src/`, including
`resource.rs` and `superweapon.rs`;
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

The 0.34 host retains the actual `AirburstWeapon` allocation through rule
persistence and the original/Ares child factories. It consumes this plugin's
existing instance policies without a new callback or configuration schema. The
Ares child keeps its installed Weapon null; other type/factory identity gaps
remain separate.

Version 0.33 adds the optional `weapon_instance_resource` query. Its stable WeaponType
array ordinal selects retained Ammo, ApplyDamage and ProjectileRange configuration;
stored IDs remain descriptive names and may repeat. The three existing feature owners
retain their policies, while the host keeps actual weapon selection, flight and damage.
A present combined result owns the instance even when both policies are disabled; an
absent result preserves the historical configuration cohort. Old strict query shapes
remain unchanged for unique names. A matching new host must reject ambiguous legacy
bindings instead of assigning one same-name declaration to several native objects.
This candidate uses config23/state1 and API v1; its WIT imports remain unchanged.
Other resource families and full Ares compatibility remain separate work.
