use super::{SurchargeInput, SurchargeKind};

#[derive(Debug, Clone, Copy)]
pub struct DefaultSurcharge {
    pub name: &'static str,
    pub rate_bp: i32,
    pub kind: SurchargeKind,
}

impl From<&DefaultSurcharge> for SurchargeInput<'static> {
    fn from(value: &DefaultSurcharge) -> Self {
        Self {
            name: value.name,
            rate_bp: value.rate_bp,
            kind: value.kind,
        }
    }
}

const DEFAULT_SURCHARGES: [DefaultSurcharge; 5] = [
    DefaultSurcharge {
        name: "Heures supplémentaires",
        rate_bp: 2500,
        kind: SurchargeKind::Overtime,
    },
    DefaultSurcharge {
        name: "Heures supplémentaires au-delà de 8 h",
        rate_bp: 5000,
        kind: SurchargeKind::Overtime,
    },
    DefaultSurcharge {
        name: "Travail du dimanche",
        rate_bp: 10000,
        kind: SurchargeKind::Sunday,
    },
    DefaultSurcharge {
        name: "Jour férié travaillé",
        rate_bp: 10000,
        kind: SurchargeKind::Holiday,
    },
    DefaultSurcharge {
        name: "Travail de nuit",
        rate_bp: 2500,
        kind: SurchargeKind::Night,
    },
];

pub fn default_surcharges() -> &'static [DefaultSurcharge] {
    &DEFAULT_SURCHARGES
}
