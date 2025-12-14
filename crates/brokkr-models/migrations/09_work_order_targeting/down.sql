-- Migration: 09_work_order_targeting (rollback)
-- Description: Remove work order label and annotation targeting tables

DROP TABLE IF EXISTS work_order_annotations;
DROP TABLE IF EXISTS work_order_labels;
