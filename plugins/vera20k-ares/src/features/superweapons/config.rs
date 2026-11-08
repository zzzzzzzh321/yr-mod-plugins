//! Existing retained SWType configuration; no live charge, selection or wallet.
use super::targeting::{HOUSE_TOKENS, Targeting};
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use vera_mod_api::{ConfigureSuperReadContext, NullableResourceName, ProductionTypeRef};

#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum Kind {
    #[default]
    NoEffect,
    Retail(u8),
    GenericWarhead,
    UnitDelivery,
}

impl Kind {
    /// ReadINI6CEC2D..6CEC7E's recognized Type history is separate from the
    /// last raw text; the two existing custom kinds extend those twelve slots.
    pub(super) fn parse(name: &str) -> Option<Self> {
        const RETAIL: [&str; 12] = [
            "MultiMissile",
            "IronCurtain",
            "LightningStorm",
            "ChronoSphere",
            "ChronoWarp",
            "ParaDrop",
            "AmerParaDrop",
            "PsychicDominator",
            "SpyPlane",
            "GeneticConverter",
            "ForceShield",
            "PsychicReveal",
        ];
        if name.eq_ignore_ascii_case("GenericWarhead") {
            Some(Self::GenericWarhead)
        } else if name.eq_ignore_ascii_case("UnitDelivery") {
            Some(Self::UnitDelivery)
        } else {
            RETAIL
                .iter()
                .position(|kind| kind.eq_ignore_ascii_case(name))
                .map(|n| Self::Retail(n as u8))
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Availability {
    pub(crate) allow_player: bool,
    pub(crate) allow_ai: bool,
    pub(crate) required_houses: u32,
    pub(crate) forbidden_houses: u32,
    pub(crate) auxiliary: Vec<u32>,
    pub(crate) negative: Vec<u32>,
    pub(crate) shots: i32,
}
impl Default for Availability {
    fn default() -> Self {
        Self {
            allow_player: true,
            allow_ai: true,
            required_houses: u32::MAX,
            forbidden_houses: 0,
            auxiliary: Vec::new(),
            negative: Vec::new(),
            shots: -1,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Charge {
    pub(crate) initial_ready: bool,
    pub(crate) virtual_charge: bool,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Presentation {
    pub(crate) animation: NullableResourceName,
    pub(crate) height: i32,
    pub(crate) visibility: u8,
    pub(crate) sound: NullableResourceName,
}
impl Default for Presentation {
    fn default() -> Self {
        Self {
            animation: NullableResourceName::Inherit,
            height: 0,
            visibility: 7,
            sound: NullableResourceName::Inherit,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Generic {
    pub(crate) damage: i32,
    pub(crate) fire_into_shroud: bool,
    pub(crate) range_minimum_bits: u64,
    pub(crate) range_maximum_bits: u64,
}
impl Default for Generic {
    fn default() -> Self {
        Self {
            damage: 0,
            fire_into_shroud: true,
            range_minimum_bits: (-1.0_f64).to_bits(),
            range_maximum_bits: (-1.0_f64).to_bits(),
        }
    }
}

#[derive(Clone, Copy, Default, Serialize, Deserialize)]
pub(crate) enum DeliveryOwner {
    #[default]
    Invoker,
    Neutral,
    Special,
    Civilian,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Delivery {
    pub(crate) types: Vec<ProductionTypeRef>,
    pub(crate) deferment: i32,
    pub(crate) owner: DeliveryOwner,
    pub(crate) base_normal: bool,
}
impl Default for Delivery {
    fn default() -> Self {
        Self {
            types: Vec::new(),
            deferment: 20,
            owner: DeliveryOwner::Invoker,
            base_normal: true,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct TypeState {
    pub(crate) stored_id: String,
    pub(crate) initialized: bool,
    pub(crate) kind: Kind,
    /// Custom handlers override action selection; None leaves stock parsing.
    pub(crate) action: Option<String>,
    pub(crate) targeting: Targeting,
    pub(crate) availability: Availability,
    pub(crate) charge: Charge,
    pub(crate) presentation: Presentation,
    pub(crate) money_amount: i32,
    pub(crate) insufficient_funds_eva: NullableResourceName,
    pub(crate) use_ai_targeting: bool,
    pub(crate) generic: Generic,
    pub(crate) delivery: Option<Delivery>,
    /// The old final sound binder sees earlier physical values even if the
    /// delivery descriptor is first created by a later Type read.
    pub(crate) activation_sound: NullableResourceName,
    /// Ext+4D4 remembers the stock handler mapped by the first reached
    /// Initialize. Later recognized stock Type values do not remap it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) nuclear: Option<super::nuclear::Nuclear>,
}

impl TypeState {
    pub(super) fn new(stored_id: &str) -> Self {
        Self {
            stored_id: stored_id.to_owned(),
            initialized: false,
            kind: Kind::NoEffect,
            action: None,
            targeting: Targeting::default(),
            availability: Availability::default(),
            charge: Charge::default(),
            presentation: Presentation::default(),
            money_amount: 0,
            // The constructor default is resolved against fixed EVAMD before
            // the first visit, independently of whether an INI body exists.
            insufficient_funds_eva: NullableResourceName::Inherit,
            use_ai_targeting: false,
            generic: Generic::default(),
            delivery: None,
            activation_sound: NullableResourceName::Inherit,
            nuclear: None,
        }
    }

    pub(super) fn selects_nuclear(&self) -> bool {
        matches!(self.kind, Kind::Retail(_)) && self.nuclear.is_some()
    }

    pub(super) fn initialize_eva_default<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        if !self.initialized && matches!(self.insufficient_funds_eva, NullableResourceName::Inherit)
        {
            self.insufficient_funds_eva = view
                .eva_find("EVA_InsufficientFunds")?
                .map_or(NullableResourceName::Clear, NullableResourceName::Named);
        }
        Ok(())
    }

    pub(super) fn read<H: Host>(
        &mut self,
        context: &ConfigureSuperReadContext,
        owner: &str,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        let read = context.read;
        let raw = view.section_or_empty(owner);
        // SWTypeExt1003354E/64/7A: pointer, signed height, AffectedHouse.
        self.presentation.animation = view.superweapon_read(
            read,
            owner,
            "SW.Animation",
            "super-animation-reference",
            self.presentation.animation.clone(),
        )?;
        self.presentation.height =
            raw.read_ares_int("SW.AnimationHeight", self.presentation.height);
        if let Some(mask) = raw.read_comma_mask("SW.AnimationVisibility", &HOUSE_TOKENS) {
            self.presentation.visibility = u8::try_from(mask).map_err(|e| e.to_string())?;
        }
        self.presentation.sound =
            raw.read_sound_reference("SW.Sound", self.presentation.sound.clone());
        self.targeting.read(&raw)?;
        //1003349E..100334C5 and10033A0F..10033AB2: retained availability
        // and charge. Scoped lists/House masks use the actual native prefixes.
        self.availability.allow_player =
            raw.read_ares_bool("SW.AllowPlayer", self.availability.allow_player);
        self.availability.allow_ai = raw.read_ares_bool("SW.AllowAI", self.availability.allow_ai);
        if let Some(value) = view.superweapon_read::<_, Option<u32>>(
            read,
            owner,
            "SW.RequiredHouses",
            "super-country-mask",
            (),
        )? {
            self.availability.required_houses = value;
        }
        if let Some(value) = view.superweapon_read::<_, Option<u32>>(
            read,
            owner,
            "SW.ForbiddenHouses",
            "super-country-mask",
            (),
        )? {
            self.availability.forbidden_houses = value;
        }
        if let Some(value) = view.superweapon_read::<_, Option<Vec<u32>>>(
            read,
            owner,
            "SW.AuxBuildings",
            "super-building-list",
            (),
        )? {
            self.availability.auxiliary = value;
        }
        if let Some(value) = view.superweapon_read::<_, Option<Vec<u32>>>(
            read,
            owner,
            "SW.NegBuildings",
            "super-building-list",
            (),
        )? {
            self.availability.negative = value;
        }
        self.availability.shots = raw.read_ares_int("SW.Shots", self.availability.shots);
        self.charge.initial_ready =
            raw.read_ares_bool("SW.InitialReady", self.charge.initial_ready);
        self.charge.virtual_charge =
            raw.read_ares_bool("SW.VirtualCharge", self.charge.virtual_charge);

        // Final RuleSet projected-section readers. Unknown Type retains
        // recognized dispatch but does not authorize a WH allocation.
        if let Some(kind) = context.effective_type.as_deref().and_then(Kind::parse) {
            self.kind = kind;
        }
        self.action = match self.kind {
            Kind::GenericWarhead => Some("Attack".into()),
            Kind::UnitDelivery => {
                let value: Option<String> = view.superweapon_read(
                    read,
                    owner,
                    "Action",
                    "super-effective:name:32",
                    None::<String>,
                )?;
                Some(
                    value
                        .filter(|name| !name.eq_ignore_ascii_case("Custom"))
                        .unwrap_or_else(|| "ParaDrop".into()),
                )
            }
            _ => None,
        };
        self.money_amount =
            view.superweapon_read(read, owner, "Money.Amount", "super-effective:ares-int", 0)?;
        self.use_ai_targeting = view.superweapon_read(
            read,
            owner,
            "SW.UseAITargeting",
            "super-effective:bool",
            false,
        )?;
        self.insufficient_funds_eva = view.superweapon_read(
            read,
            owner,
            "EVA.InsufficientFunds",
            "super-eva-reference",
            self.insufficient_funds_eva.clone(),
        )?;
        self.activation_sound =
            raw.read_sound_reference("SW.ActivationSound", self.activation_sound.clone());
        if self.kind == Kind::GenericWarhead {
            self.generic.damage =
                view.superweapon_read(read, owner, "SW.Damage", "super-effective:int", 0)?;
            self.generic.fire_into_shroud = view.superweapon_read(
                read,
                owner,
                "SW.FireIntoShroud",
                "super-effective:bool",
                true,
            )?;
            self.generic.range_minimum_bits = range_bits(view, read, owner, "SW.RangeMinimum")?;
            self.generic.range_maximum_bits = range_bits(view, read, owner, "SW.RangeMaximum")?;
        }
        // Preserve the existing raw-Type-or-existing-state gate; do not
        // enter it from the final recognized dispatch alone.
        if raw
            .read_name("Type", 0x28)
            .is_some_and(|kind| kind.eq_ignore_ascii_case("UnitDelivery"))
            || self.delivery.is_some()
        {
            let delivery = self.delivery.get_or_insert_with(Delivery::default);
            if let Some(types) = view.superweapon_read::<_, Option<Vec<ProductionTypeRef>>>(
                read,
                owner,
                "Deliver.Types",
                "super-delivery-list",
                (),
            )? {
                delivery.types = types;
            }
            let delay = raw.read_int("SW.Deferment", delivery.deferment);
            delivery.deferment = if delay < 0 { 20 } else { delay };
            delivery.base_normal = raw.read_bool("Deliver.BaseNormal", delivery.base_normal);
            if let Some(owner) = raw.read_name("Deliver.Owner", 2048) {
                delivery.owner = match owner.to_ascii_lowercase().as_str() {
                    "invoker" => DeliveryOwner::Invoker,
                    "neutral" => DeliveryOwner::Neutral,
                    "special" => DeliveryOwner::Special,
                    "civilian" => DeliveryOwner::Civilian,
                    _ => delivery.owner,
                };
            }
        }
        Ok(())
    }
}

fn range_bits<H: Host>(
    view: &ConfigView<'_, H>,
    read: u32,
    owner: &str,
    key: &str,
) -> Result<u64, String> {
    // Bad present values remain NaN and fail admission; absence disables (-1).
    if view.superweapon_read::<_, bool>(read, owner, key, "super-effective:presence", ())? {
        view.superweapon_read(
            read,
            owner,
            key,
            "super-effective:double-bits",
            f64::NAN.to_bits(),
        )
    } else {
        Ok((-1.0_f64).to_bits())
    }
}
