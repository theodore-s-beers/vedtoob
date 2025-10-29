use anyhow::Context;
use clap::{Parser, Subcommand};
use vedtoob::{
    get_chapters, get_course_slugs, get_lesson_id, get_lessons, get_readme_by_id, prettify,
};

/// View Boot.dev lesson readmes in the terminal
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show the readme for a given lesson
    Show {
        /// Course slug
        #[arg(short, long)]
        course: String,
        /// Chapter number
        #[arg(short = 'p', long)]
        chapter: u8,
        /// Lesson number
        #[arg(short, long)]
        lesson: u8,
    },
    /// List the slugs of all available courses
    ListCourses,
    /// List the chapters of a given course
    ListChapters {
        /// Course slug
        #[arg(short, long)]
        course: String,
    },
    /// List the lessons in a given course chapter
    ListLessons {
        /// Course slug
        #[arg(short, long)]
        course: String,
        /// Chapter number
        #[arg(short = 'p', long)]
        chapter: u8,
    },
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    match args.command {
        Commands::Show {
            course,
            chapter,
            lesson,
        } => {
            let id = get_lesson_id(&course, chapter, lesson).context("Failed to get lesson ID")?;
            let readme = get_readme_by_id(&id).context("Failed to get lesson readme")?;
            let prettified = prettify(&readme).context("Failed to prettify readme")?;

            bat::PrettyPrinter::new()
                .input_from_bytes(prettified.as_bytes())
                .language("markdown")
                .print()?;
        }
        Commands::ListCourses => {
            let courses = get_course_slugs().context("Failed to get course slugs")?;
            let output: String = courses
                .iter()
                .map(|(slug, title)| format!("{} = \"{}\"\n", slug, title))
                .collect();

            bat::PrettyPrinter::new()
                .input_from_bytes(output.as_bytes())
                .language("toml")
                .print()?;
        }
        Commands::ListChapters { course } => {
            let chapters = get_chapters(&course).context("Failed to get chapters")?;
            let output: String = chapters
                .iter()
                .enumerate()
                .map(|(i, title)| format!("{}: {}\n", i + 1, title))
                .collect();

            bat::PrettyPrinter::new()
                .input_from_bytes(output.as_bytes())
                .language("yaml")
                .print()?;
        }
        Commands::ListLessons { course, chapter } => {
            let lessons = get_lessons(&course, chapter).context("Failed to get lessons")?;
            let output: String = lessons
                .iter()
                .enumerate()
                .map(|(i, title)| format!("{}: {}\n", i + 1, title))
                .collect();

            bat::PrettyPrinter::new()
                .input_from_bytes(output.as_bytes())
                .language("yaml")
                .print()?;
        }
    }

    Ok(())
}
