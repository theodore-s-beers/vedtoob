use anyhow::{Context, anyhow};
use reqwest::blocking::get;
use serde_json::{Map, Value};
use std::io::Write;

//
// Public functions
//

pub fn get_chapters(slug: &str) -> Result<Vec<String>, anyhow::Error> {
    let course_uuid = get_course_id(slug)?;
    let course_url = format!("https://api.boot.dev/v1/courses/{}", course_uuid);
    let course_data: Map<String, Value> = get(course_url)?.json()?;

    let chapters: Vec<Map<String, Value>> = course_data
        .get("Chapters")
        .and_then(|v| v.as_array())
        .context("No chapters found in this course")?
        .iter()
        .filter_map(|chapter| chapter.as_object().cloned())
        .collect();

    let results: Vec<String> = chapters
        .iter()
        .map(|chapter| {
            chapter
                .get("Title")
                .and_then(|v| v.as_str())
                .context("No title found for a chapter")
                .map(|s| s.to_owned())
        })
        .collect::<Result<Vec<String>, _>>()?;

    Ok(results)
}

pub fn get_course_slugs() -> Result<Vec<(String, String)>, anyhow::Error> {
    let courses_data: Vec<Map<String, Value>> =
        get("https://api.boot.dev/v1/static/courses/overview")?.json()?;

    let mut results: Vec<(String, String)> = Vec::new();

    for course in courses_data {
        let slug = course
            .get("Slug")
            .and_then(|v| v.as_str())
            .context("No slug found for a course")?;

        let title = course
            .get("Title")
            .and_then(|v| v.as_str())
            .context("No title found for a course")?;

        results.push((slug.to_owned(), title.to_owned()));
    }

    results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

pub fn get_lesson_id(course_slug: &str, ch_no: u8, lesson_no: u8) -> Result<String, anyhow::Error> {
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

pub fn get_readme_by_id(id: &str) -> Result<String, anyhow::Error> {
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

pub fn prettify(readme: &str) -> Result<String, anyhow::Error> {
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

//
// Private functions
//

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
