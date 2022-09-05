use anyhow::{bail, Result};
use git_repository as git;
use git_repository::bstr::BString;

use crate::OutputFormat;

pub fn list(
    repo: git::Repository,
    filters: Vec<String>,
    overrides: Vec<BString>,
    format: OutputFormat,
    mut out: impl std::io::Write,
) -> Result<()> {
    if format != OutputFormat::Human {
        bail!("Only human output format is supported at the moment");
    }
    let mut repo = git::open_opts(repo.git_dir(), repo.open_options().clone().lossy_config(false))?;
    repo.config_snapshot_mut().apply_cli_overrides(overrides.into_iter())?;
    let config = repo.config_snapshot();
    let config = config.plumbing();
    if let Some(frontmatter) = config.frontmatter() {
        for event in frontmatter {
            event.write_to(&mut out)?;
        }
    }
    let filters: Vec<_> = filters.into_iter().map(Filter::new).collect();
    let mut last_meta = None;
    let mut it = config.sections_and_postmatter().peekable();
    while let Some((section, matter)) = it.next() {
        if !filters.is_empty() && !filters.iter().any(|filter| filter.matches_section(section)) {
            continue;
        }

        let meta = section.meta();
        if last_meta.map_or(true, |last| last != meta) {
            write_meta(meta, &mut out)?;
        }
        last_meta = Some(meta);

        section.write_to(&mut out)?;
        for event in matter {
            event.write_to(&mut out)?;
        }
        if it.peek().map_or(false, |(next_section, _)| {
            next_section.header().name() != section.header().name()
        }) {
            writeln!(&mut out)?;
        }
    }
    Ok(())
}

struct Filter {
    name: String,
    subsection: Option<String>,
}

impl Filter {
    fn new(input: String) -> Self {
        match git::config::parse::key(&input) {
            Some(key) => Filter {
                name: key.section_name.into(),
                subsection: key.subsection_name.map(ToOwned::to_owned),
            },
            None => Filter {
                name: input,
                subsection: None,
            },
        }
    }

    fn matches_section(&self, section: &git::config::file::Section<'_>) -> bool {
        let ignore_case = git::glob::wildmatch::Mode::IGNORE_CASE;

        if !git::glob::wildmatch(self.name.as_bytes().into(), section.header().name(), ignore_case) {
            return false;
        }
        match (self.subsection.as_deref(), section.header().subsection_name()) {
            (Some(filter), Some(name)) => {
                if !git::glob::wildmatch(filter.as_bytes().into(), name, ignore_case) {
                    return false;
                }
            }
            (None, None) | (None, Some(_)) => {}
            _ => return false,
        };
        true
    }
}

fn write_meta(meta: &git::config::file::Metadata, out: &mut impl std::io::Write) -> std::io::Result<()> {
    writeln!(
        out,
        "# From '{}' ({:?}{}{})",
        meta.path
            .as_deref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "memory".into()),
        meta.source,
        (meta.level != 0)
            .then(|| format!(", include level {}", meta.level))
            .unwrap_or_default(),
        (meta.trust != git::sec::Trust::Full)
            .then(|| ", untrusted")
            .unwrap_or_default()
    )
}
