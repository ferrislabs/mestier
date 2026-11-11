use super::action::ActionSpec;

mod achats;
mod clients;
mod devis;
mod discussions;
mod factures;
mod membres;
mod planning_a;
mod planning_b;
mod referentiel;
mod reporting;
mod support;
mod tables;
mod terrain;

pub(super) const TABLES: &[&[ActionSpec]] = &[
    support::SPECS,
    devis::SPECS,
    clients::SPECS,
    factures::SPECS,
    reporting::SPECS,
    discussions::SPECS,
    achats::SPECS,
    terrain::SPECS,
    planning_a::SPECS,
    planning_b::SPECS,
    referentiel::SPECS,
    membres::SPECS,
    tables::SPECS,
];
