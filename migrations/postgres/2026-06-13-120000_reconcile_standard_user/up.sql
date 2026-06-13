-- Reconcile the no-auth standard user (`user123`) onto its canonical id.
--
-- The uuid primary-keys migration reassigned EVERY user a random uuid
-- (`gen_random_uuid()`), so on upgraded databases the seeded standard user no
-- longer carries the canonical STANDARD_USER_ID
-- (`00000000-0000-7000-8000-000000009999`). Startup then fails to find it by id,
-- tries to re-insert `user123`, and hits `users_username_key`. Move the existing
-- row (and everything it owns) onto the canonical id so the fixed-id invariant
-- the app relies on holds after an upgrade too.
--
-- Every foreign key that references users(id) is dropped (captured dynamically so
-- this survives later FK additions), the user's rows are repointed, the user row
-- is moved, then the FKs are re-added against the final state.
DO $$
DECLARE
    v_canon CONSTANT text := '00000000-0000-7000-8000-000000009999';
    v_old   text;
    v_fk    record;
    v_defs  text[] := ARRAY[]::text[];
    v_i     int;
BEGIN
    SELECT id INTO v_old FROM users WHERE username = 'user123' AND id <> v_canon;
    IF v_old IS NULL THEN
        RETURN; -- fresh install / already canonical: nothing to do.
    END IF;

    -- 1. Drop & remember every foreign key that references users(id).
    FOR v_fk IN
        SELECT conrelid::regclass::text AS tbl, conname, pg_get_constraintdef(oid) AS def
        FROM pg_constraint
        WHERE contype = 'f' AND confrelid = 'users'::regclass
    LOOP
        v_defs := array_append(
            v_defs,
            format('ALTER TABLE %s ADD CONSTRAINT %I %s', v_fk.tbl, v_fk.conname, v_fk.def)
        );
        EXECUTE format('ALTER TABLE %s DROP CONSTRAINT %I', v_fk.tbl, v_fk.conname);
    END LOOP;

    -- 2. Repoint every column that stores the standard user's id (FK-backed and
    --    the audiobookshelf columns, which have no FK constraint).
    UPDATE device_sync_groups               SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE devices                          SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE episodes                         SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE favorite_podcast_episodes        SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE favorites                        SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE filters                          SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE gpodder_settings                 SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE listening_events                 SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE playlists                        SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE podcasts                         SET added_by = v_canon WHERE added_by = v_old;
    UPDATE sessions                         SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE subscriptions                    SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE tags                             SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE sponsorblock_user_settings       SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE audiobookshelf_listening_sessions SET user_id = v_canon WHERE user_id  = v_old;
    UPDATE audiobookshelf_media_progress    SET user_id  = v_canon WHERE user_id  = v_old;
    UPDATE audiobookshelf_playback_sessions SET user_id  = v_canon WHERE user_id  = v_old;

    -- 3. Move the user row onto the canonical id.
    UPDATE users SET id = v_canon WHERE id = v_old;

    -- 4. Re-add every foreign key we dropped, now that both ends line up.
    FOR v_i IN 1 .. coalesce(array_length(v_defs, 1), 0) LOOP
        EXECUTE v_defs[v_i];
    END LOOP;
END $$;
