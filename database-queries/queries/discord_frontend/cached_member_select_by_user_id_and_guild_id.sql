--! cached_member_select_by_user_id_and_guild_id : (user_id, guild_id, roles)
SELECT
    *
FROM
    "DiscordFrontendNightly".public."CachedMembers"
WHERE
    user_id = :user_id AND
    guild_id = :guild_id;
