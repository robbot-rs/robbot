use std::collections::HashMap;

use parking_lot::Mutex;
use robbot::model::id::{GuildId, RoleId};

use super::utils::ApiGuildMember;
use super::{GuildLink, GuildRank};

pub static PREDIATE_CACHE: Mutex<PredicateCache> = Mutex::new(PredicateCache::new());

/// A lazy-initialized cache of [`RolePredicate`] lists for each server.
///
/// It should be avoided to rebuild the predicates commonly. Instead [`PREDICATE_CACHE`] can be
/// used to reuse the predicates from the last full synchronisation run.
pub struct PredicateCache(Option<HashMap<GuildId, Vec<RolePredicates>>>);

impl PredicateCache {
    /// Creates a new, empty `PredicateCache`.
    pub const fn new() -> Self {
        Self(None)
    }

    /// Inserts a new list of [`RolePredicates`] for the given `guild_id` into the cache.
    pub fn insert(&mut self, guild_id: GuildId, predicates: Vec<RolePredicates>) {
        match &mut self.0 {
            Some(map) => {
                map.insert(guild_id, predicates);
            }
            None => {
                let mut map = HashMap::new();
                map.insert(guild_id, predicates);
                self.0 = Some(map);
            }
        }
    }

    /// Returns a list of [`RolePredicates`] for the requested `guild_id`. Returns `None` if the
    /// guild was never synchronised.
    pub fn get(&self, guild_id: GuildId) -> Option<Vec<RolePredicates>> {
        match &self.0 {
            Some(map) => map.get(&guild_id).cloned(),
            None => None,
        }
    }
}

/// A predicate that must be satisfied for a user to have a role.
#[derive(Clone, Debug)]
struct RolePredicate {
    /// The id of the guild for which this [`RankPredicate`] must be satisfied.
    guild: String,
    /// The required rank.
    rank: RankPredicate,
}

/// A predicate for a ingame guild rank.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RankPredicate {
    /// A predicate that is satisfied no matter what rank a member has.
    Any,
    /// A predicate that is only satisfied when a member has a specific rank.
    Rank(String),
}

/// A intersecting list of [`RolePredicate`]s.
///
/// The predicate is satiesfied if any [`RolePredicate`] is satisfied. [`RolePredicates`] only
/// maps to a single role, for multiple roles `Vec<RolePredicates>` should be used.
#[derive(Clone, Debug)]
pub struct RolePredicates {
    role_id: RoleId,
    predicates: Vec<RolePredicate>,
}

impl RolePredicates {
    /// Returns `true` if a `ApiMember` satisfies the predicates to have this role.
    pub fn is_satisfied<'b, I>(&self, members: I) -> bool
    where
        I: Iterator<Item = &'b ApiGuildMember<'b>>,
    {
        for member in members {
            for pred in &self.predicates {
                if pred.guild == member.guild_id {
                    match &pred.rank {
                        RankPredicate::Any => return true,
                        RankPredicate::Rank(rank) => {
                            if &member.member.rank == rank {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    /// Modify a `Vec<RoleId>` depending on whether the `RolePredicates` are satisfied. Returns
    /// whether the roles have changed.
    ///
    /// Note that `update_roles` only operates on a single role. It will only add or remove a
    /// single [`RoleId`].
    pub fn update_roles<'b, I>(&self, roles: &mut Vec<RoleId>, guilds: I) -> bool
    where
        I: Iterator<Item = &'b ApiGuildMember<'b>>,
    {
        let has_role = roles.contains(&self.role_id);
        let should_have_role = self.is_satisfied(guilds);

        log::trace!(
            "User({}) HAS {} SHOULD {}",
            self.role_id,
            has_role,
            should_have_role
        );

        // Member doesn't have role, but should.
        if !has_role && should_have_role {
            roles.push(self.role_id);
            return true;
        }

        // Member has role, but shouldn't.
        if has_role && !should_have_role {
            roles.retain(|role| *role != self.role_id);
            return true;
        }

        false
    }
}

/// A builder for a list of [`RolePredicates`].
#[derive(Clone, Debug)]
pub struct PredicatesBuilder {
    predicates: Vec<RolePredicates>,
}

impl PredicatesBuilder {
    /// Creates a new, empty `PredicatesBuilder`.
    pub fn new() -> Self {
        Self {
            predicates: Vec::new(),
        }
    }

    /// Inserts a new [`RolePredicate`], created from the given `link` and `rank`.
    pub(super) fn insert(&mut self, link: &GuildLink, rank: &GuildRank) -> &mut Self {
        for pred in &mut self.predicates {
            // If a role already has a predicate, we push it instead of creating a new one.
            if pred.role_id == rank.role_id {
                pred.predicates.push(RolePredicate {
                    guild: link.gw_guild_id.clone(),
                    rank: RankPredicate::Rank(rank.rank_name.clone()),
                });

                return self;
            }
        }

        // Create a new predicate for the role.
        self.predicates.push(RolePredicates {
            role_id: rank.role_id,
            predicates: vec![RolePredicate {
                guild: link.gw_guild_id.clone(),
                rank: RankPredicate::Rank(rank.rank_name.clone()),
            }],
        });

        self
    }

    /// Consumes this `PredicateBuilder`, returning the constructed `Vec<RolePredicates>`.
    pub fn build(self) -> Vec<RolePredicates> {
        self.predicates
    }
}
