ALTER TABLE user
    ADD UNIQUE KEY uk_user_uuid (uuid);

CREATE INDEX idx_session_last_activity
    ON session (last_activity);

CREATE INDEX idx_score_session_created_at
    ON score_session (created_at);

CREATE INDEX idx_score_session_user_score
    ON score_session (userid, score_id);
