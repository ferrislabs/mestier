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
    fn a_client_actor_is_distinguishable_from_a_user() {
        let id = Uuid::from_u128(1);

        assert_eq!(actor_columns(Actor::client(id)), ("client", Some(id)));
    }
}
