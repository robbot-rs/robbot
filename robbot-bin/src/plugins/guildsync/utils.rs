use crate::plugins::guildsync::predicates::PREDIATE_CACHE;

use super::{GuildLink, GuildMember, GuildRank};

use futures::pin_mut;
use futures::StreamExt;
use gw2api::Client;
use robbot::builder::EditMember;
use robbot::model::guild::Member;
use robbot::model::id::UserId;
use robbot::model::id::{GuildId, RoleId};
use robbot::store::{delete, get};
use robbot::Result;
use robbot_core::context::Context;

use ::gw2api::v2::guild::GuildMember as ApiMember;

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
    fn is_satisfied<'b, I>(&self, members: I) -> bool
    where
        I: Iterator<Item = &'b ApiGuildMember<'b>>,
    {
        for member in members {
            for pred in &self.predicates {
                if pred.guild == member.guild_id {
                    match pred.rank {
                        RankPredicate::Any => return true,
                        RankPredicate::Rank(rank) => {
                            if member.member.rank == rank {
                                return true;
                            }
                        }
                    }
                }
            }
        }

        false
    }

    fn update_roles<'b, I>(&self, roles: &mut Vec<RoleId>, guilds: I)
    where
        I: Iterator<Item = &'b ApiGuildMember<'b>>,
    {
        let should_have_role = self.is_satisfied(guilds);

        // Member doesn't have role, but should.
        if !roles.contains(&self.role_id) && should_have_role {
            roles.push(self.role_id);
            return;
        }

        // Member has role, but shouldn't.
        if roles.contains(&self.role_id) && !should_have_role {
            roles.retain(|role| *role != self.role_id);
        }
    }
}

#[derive(Clone, Debug)]
pub struct PredicatesBuilder {
    predicates: Vec<RolePredicates>,
}

impl PredicatesBuilder {
    fn new() -> Self {
        Self {
            predicates: Vec::new(),
        }
    }

    fn insert(&mut self, link: &GuildLink, rank: &GuildRank) -> &mut Self {
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

    fn build(self) -> Vec<RolePredicates> {
        self.predicates
    }
}

pub fn patch_member(
    preds: &Vec<RolePredicates>,
    member: Member,
    guilds: &ApiGuildMembers,
) -> EditMember {
    let mut roles = member.roles;

    for pred in preds {
        pred.update_roles(&mut roles, guilds.filter(|m| m.user_id == member.user.id));
    }

    EditMember::new(|m| {
        m.roles(roles);
    })
}

/// Update all links in a single server.
///
// 1. Fetch the list of linked members from the database, compare against current API. Remove any
// members from the database no longer present in the API.
// 2. Map all database members to api members.

pub(super) async fn update_links<T>(ctx: &Context<T>, guild_id: GuildId) -> Result
where
    T: Sync + Send,
{
    let links = get!(ctx.state.store(), GuildLink => {
        guild_id == guild_id,
    })
    .await?;

    // Collect all members of all guilds.
    let mut api_members = ApiGuildMembers::new();
    for link in &links {
        let client: Client = Client::builder().access_token(&link.api_token).into();
        let members = ::gw2api::v2::guild::GuildMembers::get(&client, &link.gw_guild_id).await?;

        let linked_members = link.members(ctx).await?;

        for linked_member in linked_members {
            match members
                .0
                .iter()
                .find(|member| member.name == linked_member.account_name)
            {
                Some(member) => api_members.push(ApiGuildMember::new(
                    link,
                    linked_member.user_id,
                    member.clone(),
                )),
                // The member is not longer in the guild. Remove it from the database.
                None => {
                    ctx.state.store().delete(linked_member).await?;
                }
            }
        }
    }

    // Build the predicates.
    let mut ranks = Vec::new();
    for link in &links {
        ranks.push(link.ranks(ctx).await?);
    }

    let mut predicates = PredicatesBuilder::new();
    for link in &links {
        for rank in ranks.last().unwrap() {
            predicates.insert(link, rank);
        }
    }

    let predicates = predicates.build();

    let ctx = ctx.guild(guild_id);
    let stream = ctx.members_iter();
    pin_mut!(stream);

    while let Some(res) = stream.next().await {
        let member = res?;

        patch_member(&predicates, member, &api_members);
    }

    PREDIATE_CACHE.lock().insert(guild_id, predicates);

    Ok(())
}

/// Run a sync task for a single link.
/// Runs the following tasks:
/// 1. Fetch all members from the store.
/// 2. Fetch all members from the GW2 API.
/// 3. Compare members from the store with members from the API. If a user is
/// in the store but not in the API, it is removed from the store.
pub(super) async fn update_link<T>(ctx: &Context<T>, guildlink: GuildLink) -> Result
where
    T: Sync + Send,
{
    // Fetch members from database.
    let members = guildlink.members(ctx).await?;

    // Fetch memebers from GW2API.
    let api_members =
        super::gw2api::GuildMember::get(&guildlink.gw_guild_id, &guildlink.api_token).await?;

    // Filter out any members from the store that are not
    // in `api_members` and remove them from the store.
    let members = {
        let mut member_new = Vec::with_capacity(members.len());

        for member in members {
            match api_members
                .iter()
                .find(|api_member| member.account_name == api_member.name)
            {
                Some(api_member) => member_new.push((member, &api_member.rank)),
                None => {
                    // Delete the member from the store.
                    delete!(ctx.state.store(), GuildMember => {
                        id == member.id,
                    })
                    .await?;
                }
            }
        }

        member_new
    };

    let ranks = guildlink.ranks(ctx).await?;

    let users = serenity::model::guild::Guild::get(&ctx.raw_ctx, guildlink.guild_id)
        .await?
        .members(&ctx.raw_ctx, None, None)
        .await?;

    for user in users.into_iter() {
        let mut user: Member = user.into();

        // Find the users associated member and their rank.
        let rank = members
            .iter()
            .find(|(member, _)| member.user_id == user.user.id)
            .map(|(_, rank)| rank);

        // User is not a guild member.
        update_user(ctx, &mut user, rank, &ranks).await?;
    }

    Ok(())
}

/// Update a user's roles.
pub(super) async fn update_user<T>(
    ctx: &Context<T>,
    member: &mut Member,
    member_rank: Option<impl AsRef<str>>,
    ranks: &[GuildRank],
) -> Result
where
    T: Send + Sync,
{
    let mut roles = member.roles.clone();

    match member_rank {
        // `member` is a guild member. Remove all roles not matching `member_rank`
        // and add matching roles for `member_rank`.
        Some(member_rank) => {
            let member_rank = ranks
                .iter()
                .find(|&rank| rank.rank_name == member_rank.as_ref());

            match member_rank {
                // Found associated rank. Remove all roles in `ranks`
                // except for `member_rank.role_id`.
                Some(member_rank) => {
                    // Remove all roles in `ranks` except the ones that
                    // match with `member_rank`.
                    for rank in ranks {
                        // if rank.role_id != member_rank.role_id
                        //     && member.roles.contains(&rank.role_id)
                        // {
                        //     remove_roles.push(member_rank.role_id);
                        // }

                        match rank.role_id == member_rank.role_id {
                            // Role is not associated with current rank.
                            false => {
                                if roles.contains(&member_rank.role_id) {
                                    roles.retain(|role| *role != rank.role_id);
                                }
                            }
                            // Role is member rank.
                            true => {
                                if !roles.contains(&member_rank.role_id) {
                                    roles.push(rank.role_id);
                                }
                            }
                        }
                    }

                    // Add the `member_rank` role if the user doesn't
                    // have it already.
                    // if !member.roles.contains(&member_rank.role_id) {
                    //     add_roles.push(member_rank.role_id);
                    // }
                }
                // No rank found. Remove all roles in `ranks`.
                None => {
                    for rank in ranks {
                        if let Some(role_id) =
                            member.roles.iter().find(|&role| *role == rank.role_id)
                        {
                            roles.retain(|role| role != role_id);
                        }
                    }
                }
            }
        }

        // `member` is not a guild member. Remove all roles in `ranks`.
        None => {
            for rank in ranks {
                if let Some(role_id) = member.roles.iter().find(|&role| rank.role_id == *role) {
                    roles.retain(|role| role != role_id);
                }
            }
        }
    }

    if roles != member.roles {
        ctx.edit_member(
            member.guild_id,
            member.user.id,
            EditMember::new(|m| {
                m.roles(roles);
            }),
        )
        .await?;
    }

    Ok(())
}

#[derive(Clone, Debug)]
pub struct ApiGuildMember<'a> {
    /// Id of the user in the server.
    user_id: UserId,
    /// Uid of the GW2 guild.
    guild_id: &'a str,
    /// The member as reported by the API.
    member: ApiMember,
}

impl<'a> ApiGuildMember<'a> {
    pub fn new(link: &'a GuildLink, user_id: UserId, member: ApiMember) -> Self {
        Self {
            guild_id: &link.gw_guild_id,
            member,
            user_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiGuildMembers<'a> {
    members: Vec<ApiGuildMember<'a>>,
}

impl<'a> ApiGuildMembers<'a> {
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    pub fn push(&mut self, value: ApiGuildMember<'a>) {
        self.members.push(value);
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an `Iterator` over all [`ApiGuildMember`]s matching the `predicate`.
    pub fn filter<F>(&'a self, predicate: F) -> GuildMemberIter<'a, F>
    where
        F: FnMut(&ApiGuildMember) -> bool,
    {
        GuildMemberIter {
            predicate,
            cursor: 0,
            members: self,
        }
    }
}

/// An `Iterator` over [`ApiGuildMember`]s.
#[derive(Clone, Debug)]
pub struct GuildMemberIter<'a, F>
where
    F: FnMut(&ApiGuildMember) -> bool,
{
    predicate: F,
    cursor: usize,
    members: &'a ApiGuildMembers<'a>,
}

impl<'a, F> Iterator for GuildMemberIter<'a, F>
where
    F: FnMut(&ApiGuildMember) -> bool,
{
    type Item = &'a ApiGuildMember<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while self.cursor < self.members.members.len() {
            let member = &self.members.members[self.cursor];
            if (self.predicate)(member) {
                return Some(member);
            }

            self.cursor += 1;
        }

        None
    }
}
