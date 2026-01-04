-- Rollback: Remove deployment_health table

DROP TRIGGER IF EXISTS update_deployment_health_timestamp ON deployment_health;
DROP TABLE IF EXISTS deployment_health;
