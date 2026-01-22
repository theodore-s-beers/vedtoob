use crate::app::{App, Pane, View};

pub trait Navigation {
    fn move_up(&mut self);
    fn move_down(&mut self);
    fn select(&mut self);
    fn go_back(&mut self);
    fn back_to_courses(&mut self);
}

impl Navigation for App {
    fn move_up(&mut self) {
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

    fn move_down(&mut self) {
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
                    // Allow scrolling until the last line is at the top
                    if self.readme_scroll < line_count.saturating_sub(1) {
                        self.readme_scroll = self.readme_scroll.saturating_add(1);
                    }
                }
            },
        }
    }

    fn select(&mut self) {
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

    fn go_back(&mut self) {
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

    fn back_to_courses(&mut self) {
        self.view = View::CourseList;
        self.chapters.clear();
        self.chapters_highlighted.clear();
        self.reset_lesson_content(false);
        self.selected_course_slug = None;
        self.selected_course_title = None;
        self.selected_course_uuid = None;
        self.status = format!("Loaded {} courses", self.courses.len());
    }
}
