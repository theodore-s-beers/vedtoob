use crate::{fetch, highlight, prettify};
use anyhow::anyhow;
use ratatui::widgets::ListState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    CourseList,
    CourseContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Chapters,
    Lessons,
    Readme,
}

pub struct App {
    pub view: View,
    pub active_pane: Pane,

    // Data
    pub courses: Vec<(String, String)>, // (slug, title)
    pub chapters: Vec<String>,
    pub lessons: Vec<String>,
    pub readme: String,

    // Search
    pub search_query: String,
    pub is_search_mode: bool,

    // Highlighted versions (ANSI strings)
    pub chapters_highlighted: String,
    pub lessons_highlighted: String,

    // Selection state
    pub course_state: ListState,
    pub chapter_state: ListState,
    pub lesson_state: ListState,

    // Track what's currently loaded
    pub selected_course_slug: Option<String>,
    pub selected_course_title: Option<String>,
    pub selected_course_uuid: Option<String>,
    pub selected_chapter_no: Option<usize>,
    pub selected_lesson_no: Option<usize>,

    // Readme scroll position
    pub readme_scroll: usize,

    // Status/error message
    pub status: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        let mut app = Self {
            view: View::CourseList,
            active_pane: Pane::Chapters,
            courses: Vec::new(),
            chapters: Vec::new(),
            lessons: Vec::new(),
            readme: String::new(),
            search_query: String::new(),
            is_search_mode: false,
            chapters_highlighted: String::new(),
            lessons_highlighted: String::new(),
            course_state: ListState::default(),
            chapter_state: ListState::default(),
            lesson_state: ListState::default(),
            selected_course_slug: None,
            selected_course_title: None,
            selected_course_uuid: None,
            selected_chapter_no: None,
            selected_lesson_no: None,
            readme_scroll: 0,
            status: String::from("Loading courses..."),
        };
        app.load_courses();
        app
    }

    pub fn load_courses(&mut self) {
        match fetch::get_course_slugs() {
            Ok(courses) => {
                self.courses = courses;
                if !self.get_filtered_courses().is_empty() {
                    self.course_state.select(Some(0));
                }
                self.status = format!("Loaded {} courses", self.courses.len());
            }
            Err(e) => {
                self.status = format!("Error loading courses: {}", e);
            }
        }
    }

    #[must_use]
    pub fn get_filtered_courses(&self) -> Vec<&(String, String)> {
        if self.search_query.is_empty() {
            self.courses.iter().collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.courses
                .iter()
                .filter(|(_, title)| title.to_lowercase().contains(&query))
                .collect()
        }
    }

    pub fn load_chapters(&mut self) {
        let (slug, title) = if let Some(idx) = self.course_state.selected() {
            let filtered = self.get_filtered_courses();
            if idx >= filtered.len() {
                return;
            }
            let (slug, title) = filtered[idx];
            (slug.clone(), title.clone())
        } else {
            return;
        };

        self.selected_course_slug = Some(slug.clone());
        self.selected_course_title = Some(title);
        self.selected_course_uuid = None;

        match fetch::get_chapters(&slug) {
            Ok(chapters) => {
                self.chapters_highlighted = Self::highlight_numbered_list(&chapters);

                self.chapters = chapters;
                self.chapter_state.select(Some(0));
                self.reset_lesson_content(true);
                self.status = format!("Loaded {} chapters", self.chapters.len());
            }
            Err(e) => {
                self.status = format!("Error loading chapters: {}", e);
            }
        }
    }

    pub fn load_lessons(&mut self) {
        if let (Some(_), Some(ch_idx)) = (&self.selected_course_slug, self.chapter_state.selected())
        {
            let ch_no = ch_idx + 1;
            self.selected_chapter_no = Some(ch_no);
            self.selected_lesson_no = None;

            let course_uuid = match self.ensure_course_uuid() {
                Ok(uuid) => uuid,
                Err(e) => {
                    self.status = format!("Error loading course: {}", e);
                    return;
                }
            };

            match fetch::get_lessons_by_course_id(&course_uuid, ch_no) {
                Ok(lessons) => {
                    self.lessons_highlighted = Self::highlight_numbered_list(&lessons);

                    self.lessons = lessons;
                    self.lesson_state.select(Some(0));
                    self.readme.clear();
                    self.status = format!("Loaded {} lessons", self.lessons.len());
                }
                Err(e) => {
                    self.status = format!("Error loading lessons: {}", e);
                }
            }
        }
    }

    pub fn load_readme(&mut self) {
        if let (Some(ch_no), Some(lesson_idx)) =
            (self.selected_chapter_no, self.lesson_state.selected())
        {
            let lesson_no = lesson_idx + 1;
            self.selected_lesson_no = Some(lesson_no);
            self.status = String::from("Loading lesson...");

            let course_uuid = match self.ensure_course_uuid() {
                Ok(uuid) => uuid,
                Err(e) => {
                    self.status = format!("Error loading course: {}", e);
                    return;
                }
            };

            match fetch::get_lesson_id_by_course_id(&course_uuid, ch_no, lesson_no) {
                Ok(lesson_id) => match fetch::get_readme_by_id(&lesson_id) {
                    Ok(readme) => {
                        self.readme = Self::highlight_markdown(&readme);
                        self.status = String::from("Lesson loaded");
                    }
                    Err(e) => {
                        self.status = format!("Error loading readme: {}", e);
                    }
                },
                Err(e) => {
                    self.status = format!("Error getting lesson ID: {}", e);
                }
            }
        }
    }

    fn ensure_course_uuid(&mut self) -> Result<String, anyhow::Error> {
        if let Some(uuid) = self.selected_course_uuid.clone() {
            return Ok(uuid);
        }

        let slug = self
            .selected_course_slug
            .as_deref()
            .ok_or_else(|| anyhow!("No course selected"))?;
        let uuid = fetch::get_course_id(slug)?;
        self.selected_course_uuid = Some(uuid.clone());
        Ok(uuid)
    }

    fn highlight_numbered_list(items: &[String]) -> String {
        let md: String = items
            .iter()
            .enumerate()
            .map(|(i, title)| format!("{}. {}", i + 1, title))
            .collect::<Vec<_>>()
            .join("\n");
        highlight(&md, "markdown").unwrap_or(md)
    }

    fn highlight_markdown(content: &str) -> String {
        let prettified = prettify(content).unwrap_or_else(|_| content.to_owned());
        highlight(&prettified, "markdown").unwrap_or(prettified)
    }

    pub const fn move_up(&mut self) {
        match self.view {
            View::CourseList => {
                if let Some(idx) = self.course_state.selected()
                    && idx > 0
                {
                    self.course_state.select(Some(idx - 1));
                }
            }
            View::CourseContent => match self.active_pane {
                Pane::Chapters => {
                    if let Some(idx) = self.chapter_state.selected()
                        && idx > 0
                    {
                        self.chapter_state.select(Some(idx - 1));
                    }
                }
                Pane::Lessons => {
                    if let Some(idx) = self.lesson_state.selected()
                        && idx > 0
                    {
                        self.lesson_state.select(Some(idx - 1));
                    }
                }
                Pane::Readme => {
                    self.readme_scroll = self.readme_scroll.saturating_sub(1);
                }
            },
        }
    }

    pub fn move_down(&mut self) {
        match self.view {
            View::CourseList => {
                let len = self.get_filtered_courses().len();
                if let Some(idx) = self.course_state.selected()
                    && idx + 1 < len
                {
                    self.course_state.select(Some(idx + 1));
                }
            }
            View::CourseContent => match self.active_pane {
                Pane::Chapters => {
                    if let Some(idx) = self.chapter_state.selected()
                        && idx + 1 < self.chapters.len()
                    {
                        self.chapter_state.select(Some(idx + 1));
                    }
                }
                Pane::Lessons => {
                    if let Some(idx) = self.lesson_state.selected()
                        && idx + 1 < self.lessons.len()
                    {
                        self.lesson_state.select(Some(idx + 1));
                    }
                }
                Pane::Readme => {
                    let line_count = self.readme.lines().count();
                    // Prevent scrolling past the end (approximate)
                    // We allow scrolling until the last line is at the top.
                    if self.readme_scroll < line_count.saturating_sub(1) {
                        self.readme_scroll = self.readme_scroll.saturating_add(1);
                    }
                }
            },
        }
    }

    pub fn select(&mut self) {
        match self.view {
            View::CourseList => {
                let filtered = self.get_filtered_courses();
                if !filtered.is_empty() {
                    self.load_chapters();
                    if !self.chapters.is_empty() {
                        self.view = View::CourseContent;
                        self.active_pane = Pane::Chapters;
                        self.exit_search();
                    }
                }
            }
            View::CourseContent => match self.active_pane {
                Pane::Chapters => {
                    self.load_lessons();
                    if !self.lessons.is_empty() {
                        self.active_pane = Pane::Lessons;
                    }
                }
                Pane::Lessons => {
                    self.load_readme();
                    if !self.readme.is_empty() {
                        self.active_pane = Pane::Readme;
                        self.readme_scroll = 0;
                    }
                }
                Pane::Readme => {} // Already viewing content
            },
        }
    }

    pub fn back_to_courses(&mut self) {
        self.view = View::CourseList;
        self.chapters.clear();
        self.chapters_highlighted.clear();
        self.reset_lesson_content(false);
        self.selected_course_slug = None;
        self.selected_course_title = None;
        self.selected_course_uuid = None;
        self.status = format!("Loaded {} courses", self.courses.len());
    }

    pub fn go_back(&mut self) {
        match self.view {
            View::CourseList => {} // Already at root
            View::CourseContent => match self.active_pane {
                Pane::Chapters => {
                    self.back_to_courses();
                }
                Pane::Lessons => {
                    self.active_pane = Pane::Chapters;
                    self.reset_lesson_content(false);
                    self.status = format!("Loaded {} chapters", self.chapters.len());
                }
                Pane::Readme => {
                    self.active_pane = Pane::Lessons;
                }
            },
        }
    }

    fn reset_lesson_content(&mut self, reset_state: bool) {
        self.lessons.clear();
        self.lessons_highlighted.clear();
        self.readme.clear();
        self.selected_chapter_no = None;
        self.selected_lesson_no = None;
        if reset_state {
            self.lesson_state.select(None);
        }
    }

    pub const fn enter_search(&mut self) {
        self.is_search_mode = true;
    }

    pub fn exit_search(&mut self) {
        self.is_search_mode = false;
        self.search_query.clear();
        self.course_state.select(Some(0));
    }

    pub const fn submit_search(&mut self) {
        self.is_search_mode = false;
    }

    pub fn append_search(&mut self, c: char) {
        self.search_query.push(c);
        // Reset selection to top when filtering changes to avoid out-of-bounds
        if self.get_filtered_courses().is_empty() {
            self.course_state.select(None);
        } else {
            self.course_state.select(Some(0));
        }
    }

    pub fn pop_search(&mut self) {
        self.search_query.pop();
        if self.get_filtered_courses().is_empty() {
            self.course_state.select(None);
        } else {
            self.course_state.select(Some(0));
        }
    }
}
