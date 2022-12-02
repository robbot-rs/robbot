use crate::plugins::guildsync::predicates::{PredicatesBuilder, PREDIATE_CACHE};

use super::predicates::RolePredicates;
use super::GuildLink;

use futures::pin_mut;
use futures::StreamExt;
use gw2api::Client;
use robbot::builder::EditMember;
use robbot::model::guild::Member;
use robbot::model::id::GuildId;
use robbot::model::id::UserId;
use robbot::store::get;
use robbot::Result;
use robbot_core::context::Context;

use ::gw2api::v2::guild::GuildMember as ApiMember;

pub fn patch_member(
    preds: &Vec<RolePredicates>,
    member: Member,
    guilds: &ApiGuildMembers,
) -> Option<EditMember> {
    let mut changed = false;
    let mut roles = member.roles;

    for pred in preds {
        if pred.update_roles(&mut roles, guilds.filter(|m| m.user_id == member.user.id)) {
            changed = true;
        }
    }

    if changed {
        Some(EditMember::new(|m| {
            m.roles(roles);
        }))
    } else {
        None
    }
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
    let mut predicates = PredicatesBuilder::new();
    for link in &links {
        for rank in link.ranks(&ctx).await? {
            predicates.insert(link, &rank);
        }
    }

    let predicates = predicates.build();

    let ctx = ctx.guild(guild_id);
    let stream = ctx.members_iter();
    pin_mut!(stream);

    while let Some(res) = stream.next().await {
        let member = res?;
        log::trace!("Checking user {}", member.user.id);

        let user_id = member.user.id;
        if let Some(edit) = patch_member(&predicates, member, &api_members) {
            ctx.as_ref().edit_member(guild_id, user_id, edit).await?;
        }
    }

    PREDIATE_CACHE.lock().insert(guild_id, predicates);

    Ok(())
}

#[derive(Clone, Debug)]
pub struct ApiGuildMember<'a> {
    /// Id of the user in the server.
    pub user_id: UserId,
    /// Uid of the GW2 guild.
    pub guild_id: &'a str,
    /// The member as reported by the API.
    pub member: ApiMember,
}

impl<'a> ApiGuildMember<'a> {
    pub(crate) fn new(link: &'a GuildLink, user_id: UserId, member: ApiMember) -> Self {
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
            self.cursor += 1;

            if (self.predicate)(member) {
                return Some(member);
            }
        }

        None
    }
}
