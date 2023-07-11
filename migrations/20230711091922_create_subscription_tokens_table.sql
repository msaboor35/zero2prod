-- Add migration script here
CREATE TABLE subscription_tokens(
    id uuid NOT NULL REFERENCES subscriptions(id),
    token TEXT NOT NULL,
    PRIMARY KEY (token)
);
