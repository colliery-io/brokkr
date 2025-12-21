-- Drop diagnostic tables in reverse order (results first due to FK)
DROP TABLE IF EXISTS diagnostic_results;
DROP TABLE IF EXISTS diagnostic_requests;
