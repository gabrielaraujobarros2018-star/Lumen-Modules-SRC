-- NotifEngine Database Extension: User Accounts + JSON Notifications
-- Target: /lumen-motonexus6/system/notif/notif_engine.db
-- Features: User accounts, notification history, achievement tracking, JSON payloads

PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = 10000;

-- Users table (Lumen OS user accounts)
CREATE TABLE IF NOT EXISTS users (
    user_id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    display_name TEXT NOT NULL,
    avatar_path TEXT DEFAULT '/lumen-motonexus6/system/user/default_avatar.png',
    notification_prefs JSON DEFAULT '{"achievements":true,"random":true,"system":true,"priority_filter":5}',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_active DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT 1
);

-- Notification Types lookup table
CREATE TABLE IF NOT EXISTS notification_types (
    type_id INTEGER PRIMARY KEY AUTOINCREMENT,
    type_name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    priority_range_min INTEGER DEFAULT 1,
    priority_range_max INTEGER DEFAULT 5,
    color_code TEXT DEFAULT '#FFFFFF',
    icon_name TEXT DEFAULT 'bell'
);

-- Insert default notification types
INSERT OR IGNORE INTO notification_types (type_name, display_name, priority_range_min, priority_range_max, color_code, icon_name) VALUES
('achievement', 'Achievement Unlocked', 4, 5, '#FFD700', 'trophy'),
('random', 'Sweet Message', 1, 3, '#4CAF50', 'heart'),
('system', 'System Update', 2, 4, '#2196F3', 'info'),
('kernel', 'Kernel Event', 3, 5, '#FF5722', 'cpu'),
('wayland', 'Graphics Event', 2, 4, '#9C27B0', 'screen');

-- Main notifications table with JSON payload
CREATE TABLE IF NOT EXISTS notifications (
    notif_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    type_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    raw_json_payload JSON NOT NULL,  -- Full JSON from SweetEngine {"type":"","message":"","priority":,"timestamp":}
    content_extracted TEXT,
    priority INTEGER NOT NULL DEFAULT 1 CHECK (priority >= 1 AND priority <= 5),
    is_read BOOLEAN DEFAULT 0,
    is_trashed BOOLEAN DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    read_at DATETIME,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    FOREIGN KEY (type_id) REFERENCES notification_types(type_id),
    INDEX idx_user_priority (user_id, priority DESC),
    INDEX idx_user_unread (user_id, is_read ASC),
    INDEX idx_created_at (created_at DESC),
    INDEX idx_type_priority (type_id, priority DESC)
);

-- User achievements tracking
CREATE TABLE IF NOT EXISTS user_achievements (
    ach_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    achievement_id TEXT NOT NULL,  -- "boot_master", "wayland_pro"
    achievement_name TEXT NOT NULL,
    description TEXT,
    progress INTEGER DEFAULT 0,
    target_value INTEGER NOT NULL,
    unlocked BOOLEAN DEFAULT 0,
    unlocked_at DATETIME,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    UNIQUE(user_id, achievement_id),
    INDEX idx_user_unlocked (user_id, unlocked DESC)
);

-- Notification delivery log (for debugging + analytics)
CREATE TABLE IF NOT EXISTS notification_deliveries (
    delivery_id INTEGER PRIMARY KEY AUTOINCREMENT,
    notif_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    delivery_status TEXT DEFAULT 'sent',  -- sent, failed, displayed
    display_method TEXT DEFAULT 'system',  -- system, wayland, console
    latency_ms INTEGER,
    delivered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (notif_id) REFERENCES notifications(notif_id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(user_id),
    INDEX idx_delivery_status (delivery_status),
    INDEX idx_user_delivery (user_id, delivered_at DESC)
);

-- User notification preferences history (tracks changes)
CREATE TABLE IF NOT EXISTS user_prefs_history (
    pref_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    old_prefs JSON,
    new_prefs JSON,
    changed_by TEXT DEFAULT 'system',
    changed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE,
    INDEX idx_user_prefs (user_id, changed_at DESC)
);

-- Default user for system notifications (SweetEngine)
INSERT OR IGNORE INTO users (user_id, username, display_name, notification_prefs) VALUES
(0, 'system', 'Lumen OS System', '{"achievements":true,"random":true,"system":true,"priority_filter":1}'),
(1, 'sweetengine', 'Sweet Experiences', '{"achievements":true,"random":true,"system":true,"priority_filter":1}');

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_notifications_user_time ON notifications(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_notifications_unread_user ON notifications(user_id, is_read) WHERE is_read = 0;
CREATE INDEX IF NOT EXISTS idx_achievements_user_progress ON user_achievements(user_id, progress DESC);

-- Views for common queries

-- View: Unread notifications per user
CREATE VIEW IF NOT EXISTS user_unread_notifications AS
SELECT 
    u.username,
    u.display_name,
    n.notif_id,
    nt.type_name,
    nt.display_name as type_display,
    n.title,
    n.content_extracted,
    n.priority,
    n.created_at
FROM notifications n
JOIN users u ON n.user_id = u.user_id
JOIN notification_types nt ON n.type_id = nt.type_id
WHERE n.is_read = 0 AND u.is_active = 1
ORDER BY n.priority DESC, n.created_at DESC;

-- View: User achievement progress
CREATE VIEW IF NOT EXISTS user_achievement_progress AS
SELECT 
    u.username,
    ua.achievement_id,
    ua.achievement_name,
    ua.description,
    ua.progress,
    ua.target_value,
    ua.unlocked,
    ROUND((ua.progress * 100.0 / ua.target_value), 1) as progress_pct
FROM user_achievements ua
JOIN users u ON ua.user_id = u.user_id
WHERE u.is_active = 1;

-- View: Recent notifications (last 24h)
CREATE VIEW IF NOT EXISTS recent_notifications AS
SELECT 
    n.notif_id,
    u.username,
    nt.type_name,
    n.title,
    n.priority,
    n.created_at,
    n.is_read
FROM notifications n
JOIN users u ON n.user_id = u.user_id
JOIN notification_types nt ON n.type_id = nt.type_id
WHERE n.created_at > datetime('now', '-1 day')
ORDER BY n.created_at DESC;

-- Trigger: Auto-mark old notifications as read (30 days)
CREATE TRIGGER IF NOT EXISTS auto_archive_old_notifications
AFTER INSERT ON notifications
FOR EACH ROW
WHEN NEW.created_at < datetime('now', '-30 days')
BEGIN
    UPDATE notifications SET is_read = 1 WHERE notif_id = NEW.notif_id;
END;

-- Trigger: Update user last_active on new notification
CREATE TRIGGER IF NOT EXISTS update_user_activity
AFTER INSERT ON notifications
FOR EACH ROW
BEGIN
    UPDATE users SET last_active = CURRENT_TIMESTAMP WHERE user_id = NEW.user_id;
END;

-- Trigger: Log preference changes
CREATE TRIGGER IF NOT EXISTS log_prefs_changes
AFTER UPDATE OF notification_prefs ON users
FOR EACH ROW
WHEN OLD.notification_prefs != NEW.notification_prefs
BEGIN
    INSERT INTO user_prefs_history (user_id, old_prefs, new_prefs)
    VALUES (NEW.user_id, OLD.notification_prefs, NEW.notification_prefs);
END;

-- Stored procedure: Insert notification with user lookup
CREATE PROCEDURE IF NOT EXISTS insert_user_notification(
    p_username TEXT,
    p_type_name TEXT,
    p_title TEXT,
    p_json_payload JSON,
    p_priority INTEGER
)
LANGUAGE SQL
BEGIN
    INSERT INTO notifications (
        user_id, type_id, title, raw_json_payload, content_extracted, priority
    )
    SELECT 
        u.user_id, 
        nt.type_id, 
        p_title,
        p_json_payload,
        json_extract(p_json_payload, '$.message'),
        p_priority
    FROM users u
    JOIN notification_types nt ON nt.type_name = p_type_name
    WHERE u.username = p_username;
END;

-- Example usage for SweetEngine integration:
-- CALL insert_user_notification('system', 'achievement', 'Boot Master Unlocked!', '{"type":"achievement","message":"Boot 10 times successfully","priority":5,"timestamp":1234567890}', 5);

