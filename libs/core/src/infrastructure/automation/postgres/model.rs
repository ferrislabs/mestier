use common::CoreError;
use events::Actor;
use uuid::Uuid;

/// Flattens the [`Actor`] enum into the two columns the log stores.
///
/// The conversion lives here, at the persistence boundary, rather than in
/// `libs/events`: the enum is the domain's shape, and a pair of nullable
/// columns is the database's. A `CHECK` constraint on `automation.event`
/// enforces the same pairing on the SQL side.
pub fn actor_columns(actor: Actor) -> (&'static str, Option<Uuid>) {
    match actor {
        Actor::User { id } => ("user", Some(id)),
        Actor::Client { id } => ("client", Some(id)),
        Actor::System => ("system", None),
    }
}

/// Rebuilds the [`Actor`] the two columns describe.
///
/// The `CHECK` constraint on `automation.event` guarantees the pairing, so a
/// row that cannot be mapped means the constraint was dropped or bypassed —
/// worth an error rather than a silent `System`.
pub fn actor_from_columns(kind: &str, id: Option<Uuid>) -> Result<Actor, CoreError> {
    match (kind, id) {
        ("user", Some(id)) => Ok(Actor::user(id)),
        ("client", Some(id)) => Ok(Actor::client(id)),
        ("system", None) => Ok(Actor::system()),
        _ => Err(CoreError::Internal(format!(
            "event row has an actor the domain cannot represent: kind `{kind}`, id {id:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_system_actor_has_no_id_column() {
        assert_eq!(actor_columns(Actor::system()), ("system", None));
    }

    #[test]
    fn a_user_actor_carries_its_id() {
        let id = Uuid::from_u128(1);

        assert_eq!(actor_columns(Actor::user(id)), ("user", Some(id)));
    }

    #[test]
    fn every_actor_survives_the_round_trip_through_its_columns() {
        for actor in [
            Actor::system(),
            Actor::user(Uuid::from_u128(1)),
            Actor::client(Uuid::from_u128(2)),
        ] {
            let (kind, id) = actor_columns(actor);

            assert_eq!(actor_from_columns(kind, id).unwrap(), actor);
        }
    }

    /// The CHECK constraint makes this unreachable from the application. If it
    /// ever happens the constraint was dropped, and guessing `System` would
    /// attribute somebody's action to nobody.
    #[test]
    fn a_pairing_the_constraint_forbids_is_an_error_not_a_guess() {
        assert!(actor_from_columns("user", None).is_err());
        assert!(actor_from_columns("system", Some(Uuid::from_u128(1))).is_err());
        assert!(actor_from_columns("wat", None).is_err());
    }

    #[test]
    fn a_client_actor_is_distinguishable_from_a_user() {
        let id = Uuid::from_u128(1);

        assert_eq!(actor_columns(Actor::client(id)), ("client", Some(id)));
    }
}
