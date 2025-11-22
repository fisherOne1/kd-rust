use colored::Colorize;

pub struct Theme {
    pub title: fn(&str) -> String,
    pub pron: fn(&str) -> String,
    pub line: fn(&str) -> String,
    #[allow(dead_code)]
    pub property: fn(&str) -> String,
    pub idx: fn(&str) -> String,
    pub addi: fn(&str) -> String,
    pub para: fn(&str) -> String,
    pub collins_para: fn(&str) -> String,
    pub eg: fn(&str) -> String,
    #[allow(dead_code)]
    pub eg_pref: fn(&str) -> String,
    pub rank: fn(&str) -> String,
}

impl Theme {
    pub fn from_name(name: &str) -> Self {
        match name {
            "temp" | "" => Self::temp(),
            "wudao" => Self::wudao(),
            "canvas" => Self::canvas(),
            _ => {
                eprintln!("{}", format!("✘ Unknown theme: {}", name).red());
                Self::temp() // Fallback to default
            }
        }
    }

    fn temp() -> Self {
        Self {
            title: |s| s.bright_magenta().italic().bold().underline().to_string(),
            pron: |s| s.normal().to_string(),
            line: |s| s.bright_black().dimmed().to_string(),
            property: |s| s.green().to_string(),
            idx: |s| s.bright_white().to_string(),
            addi: |s| s.cyan().italic().to_string(),
            para: |s| s.white().to_string(),
            collins_para: |s| s.yellow().to_string(),
            eg: |s| s.bright_white().dimmed().italic().to_string(),
            eg_pref: |s| s.bright_white().dimmed().italic().to_string(),
            rank: |s| s.bright_white().dimmed().italic().to_string(),
        }
    }

    fn wudao() -> Self {
        Self {
            title: |s| s.red().italic().bold().underline().to_string(),
            pron: |s| s.cyan().to_string(),
            line: |s| s.bright_black().dimmed().to_string(),
            property: |s| s.normal().to_string(),
            idx: |s| s.bright_white().to_string(),
            addi: |s| s.green().italic().to_string(),
            para: |s| s.white().to_string(),
            collins_para: |s| s.bright_white().to_string(),
            eg: |s| s.bright_yellow().dimmed().italic().to_string(),
            eg_pref: |s| s.green().italic().to_string(),
            rank: |s| s.red().italic().to_string(),
        }
    }

    fn canvas() -> Self {
        Self {
            title: |s| s.blue().bold().underline().to_string(),
            pron: |s| s.magenta().to_string(),
            line: |s| s.bright_black().dimmed().to_string(),
            property: |s| s.bright_cyan().bold().to_string(),
            idx: |s| s.cyan().to_string(),
            addi: |s| s.green().italic().to_string(),
            para: |s| s.black().to_string(),
            collins_para: |s| s.black().to_string(),
            eg: |s| s.bright_black().italic().to_string(),
            eg_pref: |s| s.bright_blue().to_string(),
            rank: |s| s.red().bold().to_string(),
        }
    }
}
