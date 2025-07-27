CREATE TABLE users (
	    userid VARCHAR(255) PRIMARY KEY,
	    password VARCHAR(255) NOT NULL
);
CREATE TABLE sessions (
	    session_id VARCHAR(255) PRIMARY KEY,
	    userid VARCHAR(255) NOT NULL,
	    last_activity TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
	    CONSTRAINT fk_sessions_userid FOREIGN KEY (userid) REFERENCES users(userid)
);
CREATE TABLE score_sessions (
	    session_id VARCHAR(255) PRIMARY KEY,
	    score_id VARCHAR(255),
	    userid VARCHAR(255),
	    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
	    CONSTRAINT fk_score_sessions_userid FOREIGN KEY (userid) REFERENCES users(userid)
);
CREATE TABLE score (
	    id INT AUTO_INCREMENT PRIMARY KEY,
	    userid VARCHAR(255),
	    scoreid VARCHAR(255),
	    score INT,
	    submitted_at DATETIME DEFAULT CURRENT_TIMESTAMP,
	    CONSTRAINT fk_score_userid FOREIGN KEY (userid) REFERENCES users(userid)
);
