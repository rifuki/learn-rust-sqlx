-- Add migration script here
CREATE OR REPLACE USER 'user'@'localhost' IDENTIFIED BY 'password';
CREATE OR REPLACE USER 'user'@'127.0.0.1' IDENTIFIED BY 'password';
GRANT ALL PRIVILEGES ON sqlx.* TO 'user'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON sqlx.* TO 'user'@'127.0.0.1' WITH GRANT OPTION;
FLUSH PRIVILEGES;

CREATE TABLE IF NOT EXISTS users (
    id INT AUTO_INCREMENT,
    username VARCHAR(15) NOT NULL,
    email VARCHAR(255) NOT NULL,
    PRIMARY KEY (id)
);

INSERT INTO users(username, email)
    VALUES ('john', 'johnsmith@outlook.com'),
           ('suzuki', 'suzukihope@yahoo.com')