use std::collections::HashMap;

use parking_lot::Mutex;
use robbot::model::id::{GuildId, RoleId};

use super::utils::ApiGuildMember;
use super::{GuildLink, GuildRank};

pub static PREDIATE_CACHE: Mutex<PredicateCache> = Mutex::new(PredicateCache::new());

pub struct PredicateCache(Option<HashMap<GuildId, Vec<RolePredicates>>>);

impl PredicateCache {
    pub const fn new() -> Self {
        Self(None)
    }

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
    guild: String,
    rank: RankPredicate,
}

/// A predicate for a ingame guild rank.
#[derive(Clone, Debug, PartialEq, Eq)]
enum RankPredicate {
    /// A predicate that is given no matter what rank is active.
    Any,
    /// A predicate that is only given when a specific rank is active.
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

    pub fn update_roles<'b, I>(&self, roles: &mut Vec<RoleId>, guilds: I)
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
            return;
        }

        // Member has role, but shouldn't.
        if has_role && !should_have_role {
            roles.retain(|role| *role != self.role_id);
        }
    }
}

#[derive(Clone, Debug)]
pub struct PredicatesBuilder {
    predicates: Vec<RolePredicates>,
}

impl PredicatesBuilder {
    pub fn new() -> Self {
        Self {
            predicates: Vec::new(),
        }
    }

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

    pub fn build(self) -> Vec<RolePredicates> {
        self.predicates
    }
}
