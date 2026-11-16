//! `generate` subcommand - generates a default `zebrad.toml` config.

use abscissa_core::{Command, Runnable};
use clap::Parser;
use documented::DocumentedFields;

use crate::config::ZebradConfig;

/// Generate a default `zebrad.toml` configuration
#[derive(Command, Debug, Default, Parser)]
pub struct GenerateCmd {
    /// The file to write the generated config to.
    //
    // TODO: use PathBuf here instead, to support non-UTF-8 paths
    #[clap(
        long,
        short,
        help = "The file to write the generated config to (stdout if unspecified)"
    )]
    output_file: Option<String>,
}

impl Runnable for GenerateCmd {
    /// Start the application.
    #[allow(clippy::print_stdout)]
    fn run(&self) {
        let output = generate_config(&ZebradConfig::default());

        match self.output_file {
            Some(ref output_file) => {
                use std::{fs::File, io::Write};
                File::create(output_file)
                    .expect("must be able to open output file")
                    .write_all(output.as_bytes())
                    .expect("must be able to write output");
            }
            None => {
                println!("{output}");
            }
        }
    }
}

/// The comment at the top of the generated config.
const HEADER: &str = r"# Default configuration for zebrad.
#
# This file can be used as a skeleton for custom configs.
#
# Unspecified fields use default values. Optional fields are Some(field) if the
# field is present and None if it is absent.
#
# This file is generated as an example using zebrad's current defaults.
# You should set only the config options you want to keep, and delete the rest.
# Optional fields whose default is None are commented out, with an example value.
# To use one of them, uncomment it and change the value.
#
# The config format (including a complete list of sections and fields) is
# documented here:
# https://docs.rs/zebrad/latest/zebrad/config/struct.ZebradConfig.html
#
# CONFIGURATION SOURCES (in order of precedence, highest to lowest):
#
# 1. Environment variables with ZEBRA_ prefix (highest precedence)
#    - Format: ZEBRA_SECTION__KEY (double underscore for nested keys)
#    - Examples:
#      - ZEBRA_NETWORK__NETWORK=Testnet
#      - ZEBRA_RPC__LISTEN_ADDR=127.0.0.1:8232
#      - ZEBRA_STATE__CACHE_DIR=/path/to/cache
#      - ZEBRA_TRACING__FILTER=debug
#      - ZEBRA_METRICS__ENDPOINT_ADDR=0.0.0.0:9999
#
# 2. Configuration file (TOML format)
#    - At the path specified via -c flag, e.g. `zebrad -c myconfig.toml start`, or
#    - At the default path in the user's preference directory (platform-dependent, see below)
#
# 3. Hard-coded defaults (lowest precedence)
#
# The user's preference directory and the default path to the `zebrad` config are platform dependent,
# based on `dirs::preference_dir`, see https://docs.rs/dirs/latest/dirs/fn.preference_dir.html :
#
# | Platform | Value                                 | Example                                        |
# | -------- | ------------------------------------- | ---------------------------------------------- |
# | Linux    | `$XDG_CONFIG_HOME` or `$HOME/.config` | `/home/alice/.config/zebrad.toml`              |
# | macOS    | `$HOME/Library/Preferences`           | `/Users/Alice/Library/Preferences/zebrad.toml` |
# | Windows  | `{FOLDERID_RoamingAppData}`           | `C:\Users\Alice\AppData\Local\zebrad.toml`     |
";

/// Example values for the config fields that are absent from the default config,
/// as `(section, field, TOML value)`.
///
/// `generate` writes these fields as commented-out lines, so every option is listed.
/// The tests check that this list covers every field that is absent from the default
/// config, and that the generated config still parses when these lines are uncommented.
const OPTIONAL_FIELD_EXAMPLES: &[(&str, &str, &str)] = &[
    ("network", "external_addr", "'203.0.113.1:8233'"),
    ("state", "debug_stop_at_height", "3000000"),
    ("state", "debug_validity_check_interval", "'1h'"),
    ("tracing", "filter", "'info'"),
    ("tracing", "endpoint_addr", "'127.0.0.1:3000'"),
    ("tracing", "flamegraph", "'flamegraph'"),
    ("tracing", "progress_bar", "'summary'"),
    ("tracing", "log_file", "'/var/log/zebrad.log'"),
    (
        "tracing",
        "opentelemetry_endpoint",
        "'http://localhost:4318'",
    ),
    ("tracing", "opentelemetry_service_name", "'zebrad'"),
    ("tracing", "opentelemetry_sample_percent", "10"),
    ("metrics", "endpoint_addr", "'127.0.0.1:9999'"),
    ("mempool", "debug_enable_at_height", "0"),
    ("notify", "block_notify_command", "'echo %s'"),
    ("rpc", "listen_addr", "'127.0.0.1:8232'"),
    ("rpc", "indexer_listen_addr", "'127.0.0.1:8230'"),
    ("rpc", "lightwalletd_listen_addr", "'127.0.0.1:9067'"),
    (
        "mining",
        "miner_address",
        "'t1T92bmyoPM7PTSWUnaWnLGcxkF6Jp1AwMY'",
    ),
    ("mining", "extra_coinbase_data", "'my mining pool'"),
    ("mining", "miner_memo", "'Mined by Zebra'"),
    ("health", "listen_addr", "'127.0.0.1:8080'"),
    ("zcashd_compat", "zcashd_path", "'/usr/local/bin/zcashd'"),
    ("zcashd_compat", "zcashd_datadir", "'/var/lib/zcashd'"),
    ("zcashd_compat", "p2p_connect_addr", "'127.0.0.1:8233'"),
];

/// The names and docs of a config section's fields.
struct Section {
    /// The section name, which is also the [`ZebradConfig`] field name.
    name: &'static str,

    /// The names of the fields in the section, in declaration order.
    field_names: &'static [&'static str],

    /// The docs of the fields in the section, in the same order as `field_names`.
    field_docs: &'static [&'static str],
}

/// Returns the [`Section`] for a config section type.
macro_rules! section {
    ($name:ident, $config:ty) => {
        Section {
            name: stringify!($name),
            field_names: <$config>::FIELD_NAMES,
            field_docs: <$config>::FIELD_DOCS,
        }
    };
}

/// Returns the config sections, in the same order as the [`ZebradConfig`] fields.
fn sections() -> [Section; 12] {
    [
        section!(consensus, zebra_consensus::config::Config),
        section!(metrics, crate::components::metrics::Config),
        section!(network, zebra_network::config::Config),
        section!(state, zebra_state::config::Config),
        // The tracing section is serialized from its inner config.
        section!(tracing, crate::components::tracing::InnerConfig),
        section!(sync, crate::components::sync::Config),
        section!(mempool, crate::components::mempool::Config),
        section!(notify, crate::components::notify::Config),
        section!(rpc, zebra_rpc::config::rpc::Config),
        section!(mining, zebra_rpc::config::mining::Config),
        section!(health, crate::components::health::Config),
        section!(zcashd_compat, crate::components::zcashd_compat::Config),
    ]
}

/// Returns the example value for a field that is absent from the default config.
fn optional_field_example(section: &str, field: &str) -> Option<&'static str> {
    OPTIONAL_FIELD_EXAMPLES
        .iter()
        .find(|(example_section, example_field, _)| {
            *example_section == section && *example_field == field
        })
        .map(|(_, _, example)| *example)
}

/// Returns `config` as a documented TOML file.
///
/// Fields are listed in declaration order, with their docs as comments.
/// Fields that are absent from `config` are commented out, with an example value.
pub fn generate_config(config: &ZebradConfig) -> String {
    let mut output = HEADER.to_owned();

    let mut config_table =
        toml::Table::try_from(config).expect("the zebrad config is serializable as a TOML table");

    for section in sections() {
        let mut section_table = match config_table.remove(section.name) {
            Some(toml::Value::Table(section_table)) => section_table,
            Some(value) => panic!(
                "the {} config section is a struct, so it is serialized as a table, not {value:?}",
                section.name
            ),
            None => toml::Table::new(),
        };

        output.push('\n');
        if let Ok(section_docs) = ZebradConfig::get_field_docs(section.name) {
            push_comment(&mut output, section_docs);
        }
        output.push_str(&format!("[{}]\n", section.name));

        for (field, docs) in section.field_names.iter().zip(section.field_docs) {
            if let Some(value) = section_table.remove(*field) {
                output.push('\n');
                push_comment(&mut output, docs);
                output.push_str(&key_value(field, value));
            } else if let Some(example) = optional_field_example(section.name, field) {
                output.push('\n');
                push_comment(&mut output, docs);
                output.push_str(&format!("# {field} = {example}\n"));
            }
        }

        // Serialized fields without docs, if a section's serialized fields
        // don't match its struct fields.
        for (field, value) in section_table {
            output.push('\n');
            output.push_str(&key_value(&field, value));
        }
    }

    // Sections that aren't in `sections()`.
    if !config_table.is_empty() {
        output.push('\n');
        output.push_str(
            &toml::to_string_pretty(&config_table)
                .expect("values from a serialized config are serializable"),
        );
    }

    output
}

/// Appends `docs` to `output` as a TOML comment.
fn push_comment(output: &mut String, docs: &str) {
    for line in docs.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            output.push_str("#\n");
        } else {
            output.push_str(&format!("# {line}\n"));
        }
    }
}

/// Returns `field = value` as a TOML line, or several lines for a long array.
fn key_value(field: &str, value: toml::Value) -> String {
    if let toml::Value::Table(_) = value {
        // A sub-table would become a separate TOML table after its section,
        // so write it as an inline table instead.
        return format!("{field} = {value}\n");
    }

    let mut table = toml::Table::new();
    table.insert(field.to_owned(), value);

    toml::to_string_pretty(&table).expect("values from a serialized config are serializable")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_match_zebrad_config_fields() {
        let section_names: Vec<_> = sections().iter().map(|section| section.name).collect();

        assert_eq!(section_names, ZebradConfig::FIELD_NAMES);
    }

    #[test]
    fn generated_config_parses_to_default() {
        let default_config = ZebradConfig::default();
        let generated = generate_config(&default_config);

        let parsed: ZebradConfig =
            toml::from_str(&generated).expect("the generated config is valid");

        assert_eq!(parsed, default_config);
    }

    #[test]
    fn every_field_is_generated() {
        let default_table = toml::Table::try_from(ZebradConfig::default())
            .expect("the zebrad config is serializable as a TOML table");

        for section in sections() {
            let default_section = default_table
                .get(section.name)
                .and_then(toml::Value::as_table);

            for field in section.field_names {
                let is_in_default = default_section
                    .is_some_and(|default_section| default_section.contains_key(*field));
                let has_example = optional_field_example(section.name, field).is_some();

                assert!(
                    is_in_default != has_example,
                    "{}.{field} must be in the default config or in OPTIONAL_FIELD_EXAMPLES, \
                     but not both: is_in_default: {is_in_default}, has_example: {has_example}",
                    section.name,
                );
            }
        }

        for (section, field, _) in OPTIONAL_FIELD_EXAMPLES {
            assert!(
                sections()
                    .iter()
                    .any(|s| s.name == *section && s.field_names.contains(field)),
                "OPTIONAL_FIELD_EXAMPLES has an unknown field: {section}.{field}",
            );
        }
    }

    #[test]
    fn optional_field_examples_are_valid() {
        let generated = generate_config(&ZebradConfig::default());
        let mut config_table = toml::Table::try_from(ZebradConfig::default())
            .expect("the zebrad config is serializable as a TOML table");

        for (section, field, example) in OPTIONAL_FIELD_EXAMPLES {
            assert!(
                generated.contains(&format!("\n# {field} = {example}\n")),
                "the generated config has a commented-out line for {section}.{field}",
            );

            let example: toml::Table = toml::from_str(&format!("{field} = {example}"))
                .unwrap_or_else(|e| panic!("the {section}.{field} example is valid TOML: {e}"));

            config_table
                .get_mut(*section)
                .and_then(toml::Value::as_table_mut)
                .expect("every config section is in the default config")
                .extend(example);
        }

        let config_with_examples: ZebradConfig = config_table
            .try_into()
            .expect("the config is valid with every example set");

        assert_ne!(config_with_examples, ZebradConfig::default());
    }
}
