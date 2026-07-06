-- Separate apparatus (equipment) from load semantics (load_type).
ALTER TABLE exercises ADD COLUMN load_type TEXT NOT NULL DEFAULT 'external';

UPDATE exercises SET load_type = 'body_mass' WHERE equipment = 'bodyweight';
UPDATE exercises SET load_type = 'body_mass' WHERE equipment IN ('rings', 'parallel bars');
UPDATE exercises SET load_type = 'none' WHERE equipment = 'none';
UPDATE exercises
SET load_type = 'body_mass'
WHERE equipment IS NULL
  AND category IN ('calisthenics', 'flexibility');

UPDATE exercises SET equipment = NULL WHERE equipment = 'bodyweight';