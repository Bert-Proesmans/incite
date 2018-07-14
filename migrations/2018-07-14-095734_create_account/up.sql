CREATE TABLE "accounts" (
	"id" integer primary key AUTOINCREMENT NOT NULL,
	"email" varchar(255) NOT NULL,
	"web_credential" varchar(255) NOT NULL,
	"battle_tag" varchar(255) NOT NULL,
	"full_name" varchar(255),
	"last_online" datetime DEFAULT (datetime('now')) NOT NULL,
	"last_away_time" datetime DEFAULT (datetime('now')) NOT NULL,
	"last_invisible_time" datetime DEFAULT (datetime('now')) NOT NULL,
	"dnd" boolean,
	"flags" bigint
)
