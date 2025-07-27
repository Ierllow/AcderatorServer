CREATE TABLE user (
    userid VARCHAR(255) PRIMARY KEY,
    password VARCHAR(255) NOT NULL
);

CREATE TABLE session (
    session_id VARCHAR(255) PRIMARY KEY,
    userid VARCHAR(255) NOT NULL,
    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    CONSTRAINT fk_session_userid FOREIGN KEY (userid) REFERENCES user(userid)
);

CREATE TABLE score_session (
    session_id VARCHAR(255) PRIMARY KEY,
    score_id INT NOT NULL,
    userid VARCHAR(255) NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_score_session_userid FOREIGN KEY (userid) REFERENCES user(userid)
);

CREATE TABLE score (
    id INT AUTO_INCREMENT PRIMARY KEY,
    userid VARCHAR(255) NOT NULL,
    score_id INT NOT NULL,
    score INT NOT NULL,
    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT fk_score_userid FOREIGN KEY (userid) REFERENCES user(userid),
    UNIQUE KEY uk_userid_score_id (userid, score_id)
);

