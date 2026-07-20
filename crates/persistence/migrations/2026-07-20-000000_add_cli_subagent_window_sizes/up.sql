ALTER TABLE windows ADD cli_subagent_width FLOAT CHECK (cli_subagent_width >= 0);
ALTER TABLE windows ADD cli_subagent_height FLOAT CHECK (cli_subagent_height >= 0);
