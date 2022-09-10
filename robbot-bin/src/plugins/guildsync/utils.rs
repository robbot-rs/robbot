use super::{GuildLink, GuildMember, GuildRank};

use robbot::{Result, StoreData};
use robbot_core::context::Context;

/// Run a sync task for a single link.
/// Runs the following tasks:
/// 1. Fetch all members from the store.
/// 2. Fetch all members from the GW2 API.
/// 3. Compare members from the store with members from the API. If a user is
/// in the store but not in the API, it is removed from the store.
pub(super) async fn update_link<T>(ctx: &Context<T>, guildlink: GuildLink) -> Result
where
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
                    ctx.state
                        .store()
                        .delete(GuildMember::query().id(member.id))
                        .await?;
                }
            }
        }

        member_new
    };

    let ranks = guildlink.ranks(ctx).await?;

    let mut users = serenity::model::guild::Guild::get(&ctx.raw_ctx, guildlink.guild_id)
        .await?
        .members(&ctx.raw_ctx, None, None)
        .await?;

    for user in users.iter_mut() {
        // Find the users associated member and their rank.
        let rank = members
            .iter()
            .find(|(member, _)| member.user_id == user.user.id)
            .map(|(_, rank)| rank);

        // User is not a guild member.
        update_user(ctx, user, rank, &ranks).await?;
    }

    Ok(())
}

/// Update a user's roles.
pub(super) async fn update_user<T>(
    ctx: &Context<T>,
    member: &mut serenity::model::guild::Member,
    member_rank: Option<impl AsRef<str>>,
    ranks: &[GuildRank],
) -> Result {
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
        member
            .edit(&ctx.raw_ctx, |edit_user| {
                edit_user.roles(roles);
                edit_user
            })
            .await?;
    }

    Ok(())
}
