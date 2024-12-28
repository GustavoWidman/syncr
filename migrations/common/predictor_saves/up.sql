-- Your SQL goes here
CREATE TABLE `predictor_saves`(
	`id` INTEGER NOT NULL PRIMARY KEY,
	`save` BINARY NOT NULL,
	`created_at` TIMESTAMP NOT NULL,
	`updated_at` TIMESTAMP NOT NULL
);

