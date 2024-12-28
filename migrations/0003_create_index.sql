-- Migration number: 0003 	 2024-12-28T07:00:53.712Z
CREATE INDEX idx_will_merged_at_merged ON merge (will_merged_at, merged);
