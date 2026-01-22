use anyhow::{Context, anyhow};
use reqwest::blocking::get;
use serde::Deserialize;
use serde_json::Value;

// Response types for /v1/static/courses/overview
#[derive(Deserialize)]
struct CourseOverview {
    #[serde(rename = "Slug")]
    slug: String,
    #[serde(rename = "Title")]
    title: String,
}

// Response types for /v1/static/courses/slug/{slug}
#[derive(Deserialize)]
struct StaticCourseResponse {
    #[serde(rename = "Course")]
    course: StaticCourse,
}

#[derive(Deserialize)]
struct StaticCourse {
    #[serde(rename = "UUID")]
    uuid: String,
    #[serde(rename = "Chapters")]
    chapters: Vec<StaticChapter>,
}

#[derive(Deserialize)]
struct StaticChapter {
    #[serde(rename = "Title")]
    title: String,
}

// Response types for /v1/courses/{uuid}
#[derive(Deserialize)]
struct CourseResponse {
    #[serde(rename = "Chapters")]
    chapters: Vec<Chapter>,
}

#[derive(Deserialize)]
struct Chapter {
    #[serde(rename = "Lessons")]
    lessons: Vec<Lesson>,
}

#[derive(Deserialize)]
struct Lesson {
    #[serde(rename = "UUID")]
    uuid: String,
    #[serde(rename = "Title")]
    title: String,
}

// Response types for /v1/static/lessons/{id}
#[derive(Deserialize)]
struct LessonResponse {
    #[serde(rename = "Lesson")]
    lesson: Value,
}

pub fn get_chapters(slug: &str) -> Result<Vec<String>, anyhow::Error> {
    let url = format!("https://api.boot.dev/v1/static/courses/slug/{}", slug);
    let response: StaticCourseResponse = get(url)?.json()?;
    let titles = response
        .course
        .chapters
        .into_iter()
        .map(|ch| ch.title)
        .collect();
    Ok(titles)
}

pub fn get_course_id(slug: &str) -> Result<String, anyhow::Error> {
    let url = format!("https://api.boot.dev/v1/static/courses/slug/{}", slug);
    let response: StaticCourseResponse = get(url)?.json()?;
    Ok(response.course.uuid)
}

pub fn get_course_slugs() -> Result<Vec<(String, String)>, anyhow::Error> {
    let courses: Vec<CourseOverview> =
        get("https://api.boot.dev/v1/static/courses/overview")?.json()?;

    let mut results: Vec<(String, String)> =
        courses.into_iter().map(|c| (c.slug, c.title)).collect();

    results.sort_unstable_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

pub fn get_lesson_id_by_course_id(
    course_uuid: &str,
    ch_no: usize,
    lesson_no: usize,
) -> Result<String, anyhow::Error> {
    let url = format!("https://api.boot.dev/v1/courses/{}", course_uuid);
    let response: CourseResponse = get(url)?.json()?;

    if ch_no == 0 {
        return Err(anyhow!("Chapter number must be >= 1"));
    }
    let chapter = response
        .chapters
        .get(ch_no - 1)
        .context(format!("No chapter {} in course '{}'", ch_no, course_uuid))?;

    if lesson_no == 0 {
        return Err(anyhow!("Lesson number must be >= 1"));
    }
    let lesson = chapter.lessons.get(lesson_no - 1).context(format!(
        "No lesson {} in chapter {} of course '{}'",
        lesson_no, ch_no, course_uuid
    ))?;

    Ok(lesson.uuid.clone())
}

pub fn get_lessons_by_course_id(
    course_uuid: &str,
    ch_no: usize,
) -> Result<Vec<String>, anyhow::Error> {
    let url = format!("https://api.boot.dev/v1/courses/{}", course_uuid);
    let response: CourseResponse = get(url)?.json()?;

    if ch_no == 0 {
        return Err(anyhow!("Chapter number must be >= 1"));
    }
    let chapter = response
        .chapters
        .get(ch_no - 1)
        .context(format!("No chapter {} in course '{}'", ch_no, course_uuid))?;

    let titles = chapter.lessons.iter().map(|l| l.title.clone()).collect();
    Ok(titles)
}

pub fn get_readme_by_id(id: &str) -> Result<String, anyhow::Error> {
    let url = format!("https://api.boot.dev/v1/static/lessons/{}", id);
    let response: LessonResponse = get(url)?.json()?;

    let lesson = response
        .lesson
        .as_object()
        .context("Lesson is not an object")?;

    let readme = lesson
        .iter()
        .find(|(k, _)| k.starts_with("LessonData"))
        .and_then(|(_, v)| v.get("Readme"))
        .and_then(|v| v.as_str())
        .context("No readme found in lesson data")?;

    Ok(readme.to_owned())
}
