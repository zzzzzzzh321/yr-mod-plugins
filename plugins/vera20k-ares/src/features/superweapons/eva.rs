//! Retained SWTypeExt Vox references; the host owns the fixed EVAMD catalog,
//! native reference reader and queue. No audio samples or live state live here.
use super::Kind;
use crate::support::{ConfigView, Host};
use vera_mod_api::{NullableResourceName as Name, SuperWeaponEvaPolicy};

pub(super) fn constructor<H: Host>(
    view: &ConfigView<'_, H>,
) -> Result<SuperWeaponEvaPolicy, String> {
    // Ctor100316C5..10031700 clears all six;100319CF resolves SelectTarget.
    Ok(SuperWeaponEvaPolicy {
        ready: Name::Clear,
        activated: Name::Clear,
        detected: Name::Clear,
        impatient: Name::Clear,
        select_target: find(view, "EVA_SelectTarget")?,
    })
}

pub(super) fn initialize<H: Host>(
    eva: &mut SuperWeaponEvaPolicy,
    kind: Option<Kind>,
    view: &ConfigView<'_, H>,
) -> Result<(), String> {
    // First mapped Initialize only, before common Load10033370. Original
    // initializers: Nuclear10073360, Protect10074B60, Storm10072C50,
    // Chronosphere1006EF10, ParaDrop10073DC0, Dominator10070990,
    // Spy10075800, Mutator10072090, Reveal100752F0; ChronoWarp100700B0
    // and the two custom handlers leave the three ctor ordinals at -1.
    // This retained table does not admit unported redirected effect consumers.
    let (ready, activated, detected) = match kind {
        Some(Kind::Retail(0)) => (
            "EVA_NuclearMissileReady",
            "EVA_NuclearMissileLaunched",
            "EVA_NuclearSiloDetected",
        ),
        Some(Kind::Retail(1)) => (
            "EVA_IronCurtainReady",
            "EVA_IronCurtainActivated",
            "EVA_IronCurtainDetected",
        ),
        Some(Kind::Retail(2)) => (
            "EVA_LightningStormReady",
            "EVA_LightningStormCreated",
            "EVA_WeatherDeviceReady",
        ),
        Some(Kind::Retail(3)) => (
            "EVA_ChronosphereReady",
            "EVA_ChronosphereActivated",
            "EVA_ChronosphereDetected",
        ),
        Some(Kind::Retail(5 | 6)) => ("EVA_ReinforcementsReady", "", ""),
        Some(Kind::Retail(7)) => (
            "EVA_PsychicDominatorReady",
            "EVA_PsychicDominatorActivated",
            "EVA_PsychicDominatorDetected",
        ),
        Some(Kind::Retail(8)) => ("EVA_SpyPlaneReady", "", ""),
        Some(Kind::Retail(9)) => (
            "EVA_GeneticMutatorReady",
            "EVA_GeneticMutatorActivated",
            "EVA_GeneticMutatorDetected",
        ),
        Some(Kind::Retail(10)) => ("EVA_ForceShieldReady", "", ""),
        Some(Kind::Retail(11)) => ("EVA_PsychicRevealReady", "", ""),
        _ => return Ok(()),
    };
    eva.ready = find(view, ready)?;
    eva.activated = find(view, activated)?;
    eva.detected = find(view, detected)?;
    Ok(())
}

fn find<H: Host>(view: &ConfigView<'_, H>, name: &str) -> Result<Name, String> {
    if name.is_empty() {
        return Ok(Name::Clear);
    }
    Ok(view.eva_find(name)?.map_or(Name::Clear, Name::Named))
}

pub(super) fn read<H: Host>(
    eva: Option<&mut SuperWeaponEvaPolicy>,
    insufficient_funds: &mut Name,
    read: u32,
    owner: &str,
    view: &ConfigView<'_, H>,
) -> Result<(), String> {
    // Common10033392..10033415: six fixed-catalog reads, in this order.
    // Older saved cohorts retain their original single-field read.
    if let Some(eva) = eva {
        for (key, value) in [
            ("EVA.Ready", &mut eva.ready),
            ("EVA.Activated", &mut eva.activated),
            ("EVA.Detected", &mut eva.detected),
            ("EVA.Impatient", &mut eva.impatient),
            ("EVA.InsufficientFunds", insufficient_funds),
            ("EVA.SelectTarget", &mut eva.select_target),
        ] {
            *value =
                view.superweapon_read(read, owner, key, "super-eva-reference", value.clone())?;
        }
    } else {
        *insufficient_funds = view.superweapon_read(
            read,
            owner,
            "EVA.InsufficientFunds",
            "super-eva-reference",
            insufficient_funds.clone(),
        )?;
    }
    Ok(())
}
