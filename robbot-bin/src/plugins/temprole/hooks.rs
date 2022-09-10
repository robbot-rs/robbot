use super::{AssignedRole, TempRole};
use crate::bot::GuildMemberUpdateContext;
use crate::core::store::StoreData;
use robbot::Result;

pub async fn guild_member_update(ctx: GuildMemberUpdateContext) -> Result {
    let member = ctx.event.1;

    let roles = ctx
        .state
        .store
        .as_ref()
        .unwrap()
        .get(Some(TempRole::query().guild_id(member.guild_id)))
        .await?;

    let now = chrono::Utc::now();

    for role in roles {
        if member.roles.contains(&role.role_id) {
            #[cfg(debug_assertions)]
            println!("Temprole assigned to user: {:?}", member.user);

            let remove_timestamp = now.timestamp() + role.lifetime as i64;

            let assigned = ctx
                .state
                .store
                .as_ref()
                .unwrap()
                .get(Some(
                    AssignedRole::query()
                        .guild_id(member.guild_id)
                        .role_id(role.role_id),
                ))
                .await?;

            // Insert if not already found.
            if let None = assigned.iter().find(|a| a.user_id == member.user.id) {
                let assigned_role = AssignedRole::new(
                    member.guild_id,
                    role.role_id,
                    member.user.id,
                    remove_timestamp,
                );

                ctx.state
                    .store
                    .as_ref()
                    .unwrap()
                    .insert(assigned_role)
                    .await?;
            }
        }
    }

    Ok(())
}
