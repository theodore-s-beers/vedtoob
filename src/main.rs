use anyhow::{Context, anyhow};
use clap::Parser;
use reqwest::blocking::get;
use serde_json::{Map, Value};
use std::io::Write;

/// Fetch a Boot.dev lesson readme by UUID or course/chapter/lesson
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Course slug
    #[arg(short, long, requires_all = &["chapter", "lesson"])]
    course: String,
    /// Chapter number
    #[arg(short = 'p', long, requires_all = &["course", "lesson"])]
    chapter: u8,
    /// Lesson number
    #[arg(short, long, requires_all = &["course", "chapter"])]
    lesson: u8,
}

fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let id = get_lesson_id(&args.course, args.chapter, args.lesson)
        .context("Failed to get lesson ID")?;

    let readme = get_readme_by_id(&id).context("Failed to get lesson readme")?;
    let prettified = prettify(&readme).context("Failed to prettify readme")?;

    bat::PrettyPrinter::new()
        .input_from_bytes(prettified.as_bytes())
        .language("markdown")
        .print()?;

    Ok(())
}

fn get_lesson_id(course_slug: &str, ch_no: u8, lesson_no: u8) -> Result<String, anyhow::Error> {
    let course_uuid = get_course_id(course_slug)?;
    let course_url = format!("https://api.boot.dev/v1/courses/{}", course_uuid);
    let course_data: Map<String, Value> = get(course_url)?.json()?;

    let chapters: Vec<Map<String, Value>> = course_data
        .get("Chapters")
        .and_then(|v| v.as_array())
        .context("No chapters found in this course")?
        .iter()
        .filter_map(|chapter| chapter.as_object().cloned())
        .collect();

    if chapters.len() < ch_no as usize {
        return Err(anyhow!("No chapter {} in course '{}'", ch_no, course_slug));
    }

    let chapter = &chapters[(ch_no - 1) as usize];

    let lessons: Vec<Map<String, Value>> = chapter
        .get("Lessons")
        .and_then(|v| v.as_array())
        .context("No lessons found in this chapter")?
        .iter()
        .filter_map(|lesson| lesson.as_object().cloned())
        .collect();

    if lessons.len() < lesson_no as usize {
        return Err(anyhow!(
            "No lesson {} in chapter {} of course '{}'",
            lesson_no,
            ch_no,
            course_slug
        ));
    }

    let lesson = &lessons[(lesson_no - 1) as usize];
    let lesson_id = lesson
        .get("UUID")
        .and_then(|v| v.as_str())
        .context("No UUID found for this lesson")?;

    Ok(lesson_id.to_owned())
}

fn get_course_id(slug: &str) -> Result<String, anyhow::Error> {
    let url = format!("https://api.boot.dev/v1/static/courses/slug/{}", slug);
    let data: Map<String, Value> = get(url)?.json()?;
    let course_data = data
        .get("Course")
        .and_then(|v| v.as_object())
        .context("No course found with this slug")?;
    let uuid = course_data
        .get("UUID")
        .and_then(|v| v.as_str())
        .context("No ID found for this course")?;
    Ok(uuid.to_owned())
}

fn get_readme_by_id(id: &str) -> Result<String, anyhow::Error> {
    let lookup_url = format!("https://api.boot.dev/v1/static/lessons/{}", id);
    let response_text = get(lookup_url)?.text()?;
    let data: Map<String, Value> = serde_json::from_str(&response_text)?;

    let lesson = data
        .get("Lesson")
        .and_then(|v| v.as_object())
        .context("No lesson found with this UUID")?;

    let lesson_data = lesson
        .iter()
        .find(|(k, _)| k.starts_with("LessonData"))
        .map(|(_, v)| v)
        .context("No lesson data found")?;

    let readme = lesson_data
        .get("Readme")
        .and_then(|v| v.as_str())
        .context("No readme found in lesson data")?;

    Ok(readme.to_owned())
}

fn prettify(readme: &str) -> Result<String, anyhow::Error> {
    let mut input_file = tempfile::NamedTempFile::new()?;
    write!(input_file, "{}", readme)?;

    let pandoc = std::process::Command::new("pandoc")
        .arg(input_file.path())
        .arg("-t")
        .arg("markdown")
        .arg("--columns=80")
        .output()
        .context("Failed to call Pandoc")?;

    let output = str::from_utf8(&pandoc.stdout)?;
    Ok(output.to_owned())
}
