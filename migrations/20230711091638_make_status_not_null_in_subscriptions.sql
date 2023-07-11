-- Add migration script here
BEGIN;
    UPDATE subscriptions
        set status = 'confirmed'
        WHERE status IS NULL;
    
    ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;
COMMIT;
