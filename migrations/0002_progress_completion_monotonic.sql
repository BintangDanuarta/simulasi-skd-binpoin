UPDATE study_progress SET quiz_completed=FALSE WHERE material_completed=FALSE AND quiz_completed=TRUE;
INSERT INTO exam_attempts(exam_id,attempt_no,result,created_at)
SELECT exam_id,1,result,created_at FROM exam_history
ON CONFLICT(exam_id,attempt_no) DO NOTHING;
-- Existing sessions are intentionally invalidated on this schema change.
TRUNCATE sessions;
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS client_ip TEXT NOT NULL DEFAULT '';
ALTER TABLE exams ADD COLUMN IF NOT EXISTS attempt_no INTEGER NOT NULL DEFAULT 1;
