-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    'f47784d9-0709-46e5-8c7a-3caf0d2d06fa',
    'admin',
    '$argon2id$v=19$m=1500,t=2,p=1$ykdR7AUh3fsGsLWqfPV1+Q$KMg3J7eXgbT5vkG2qigCf+ISC9ZN519JrjBl/HyZn04'
)