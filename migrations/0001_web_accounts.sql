CREATE TABLE IF NOT EXISTS users (
  id UUID PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sessions (
  token_hash TEXT PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS sessions_user_idx ON sessions(user_id);
CREATE INDEX IF NOT EXISTS sessions_expiry_idx ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS users_created_idx ON users(created_at DESC);

CREATE TABLE IF NOT EXISTS study_progress (
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  material_id TEXT NOT NULL,
  quiz_completed BOOLEAN NOT NULL DEFAULT FALSE,
  material_completed BOOLEAN NOT NULL DEFAULT FALSE,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (user_id, material_id)
);

CREATE TABLE IF NOT EXISTS exams (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  mode TEXT NOT NULL,
  started_at BIGINT NOT NULL,
  deadline BIGINT NOT NULL,
  finished BOOLEAN NOT NULL DEFAULT FALSE,
  questions JSONB NOT NULL,
  answers JSONB NOT NULL DEFAULT '{}'::jsonb,
  unsure JSONB NOT NULL DEFAULT '[]'::jsonb,
  result JSONB,
  attempt_no INTEGER NOT NULL DEFAULT 1,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX IF NOT EXISTS exams_one_active_per_user_idx ON exams(user_id) WHERE finished=false;
CREATE INDEX IF NOT EXISTS exams_user_history_idx ON exams(user_id, finished, created_at DESC);

CREATE TABLE IF NOT EXISTS exam_history (
  exam_id UUID PRIMARY KEY REFERENCES exams(id) ON DELETE CASCADE,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  result JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS exam_history_user_idx ON exam_history(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS practice_history (
  id UUID PRIMARY KEY,
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  result JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS practice_history_user_idx ON practice_history(user_id, created_at DESC);

CREATE TABLE IF NOT EXISTS global_content (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  body JSONB NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE IF NOT EXISTS app_metadata (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS exam_attempts (
  exam_id UUID NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
  attempt_no INTEGER NOT NULL,
  result JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (exam_id, attempt_no)
);