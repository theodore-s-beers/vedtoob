use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChapterKey {
    course_uuid: String,
    chapter_no: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LessonKey {
    course_uuid: String,
    chapter_no: usize,
    lesson_no: usize,
}

#[derive(Debug, Default)]
pub struct Cache {
    courses: Option<Vec<(String, String)>>,
    chapters: HashMap<String, Vec<String>>,
    course_uuids: HashMap<String, String>,
    lessons: HashMap<ChapterKey, Vec<String>>,
    lesson_ids: HashMap<LessonKey, String>,
    readmes: HashMap<String, String>,
}

impl Cache {
    pub const fn courses(&self) -> Option<&Vec<(String, String)>> {
        self.courses.as_ref()
    }

    pub fn set_courses(&mut self, courses: Vec<(String, String)>) {
        self.courses = Some(courses);
    }

    pub fn chapters(&self, slug: &str) -> Option<&Vec<String>> {
        self.chapters.get(slug)
    }

    pub fn set_chapters(&mut self, slug: String, chapters: Vec<String>) {
        self.chapters.insert(slug, chapters);
    }

    pub fn course_uuid(&self, slug: &str) -> Option<&String> {
        self.course_uuids.get(slug)
    }

    pub fn set_course_uuid(&mut self, slug: String, uuid: String) {
        self.course_uuids.insert(slug, uuid);
    }

    pub fn lessons(&self, course_uuid: &str, chapter_no: usize) -> Option<&Vec<String>> {
        self.lessons.get(&ChapterKey {
            course_uuid: course_uuid.to_owned(),
            chapter_no,
        })
    }

    pub fn set_lessons(&mut self, course_uuid: String, chapter_no: usize, lessons: Vec<String>) {
        self.lessons.insert(
            ChapterKey {
                course_uuid,
                chapter_no,
            },
            lessons,
        );
    }

    pub fn lesson_id(
        &self,
        course_uuid: &str,
        chapter_no: usize,
        lesson_no: usize,
    ) -> Option<&String> {
        self.lesson_ids.get(&LessonKey {
            course_uuid: course_uuid.to_owned(),
            chapter_no,
            lesson_no,
        })
    }

    pub fn set_lesson_id(
        &mut self,
        course_uuid: String,
        chapter_no: usize,
        lesson_no: usize,
        lesson_id: String,
    ) {
        self.lesson_ids.insert(
            LessonKey {
                course_uuid,
                chapter_no,
                lesson_no,
            },
            lesson_id,
        );
    }

    pub fn readme(&self, lesson_id: &str) -> Option<&String> {
        self.readmes.get(lesson_id)
    }

    pub fn set_readme(&mut self, lesson_id: String, readme: String) {
        self.readmes.insert(lesson_id, readme);
    }
}
