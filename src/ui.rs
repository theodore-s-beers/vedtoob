use crate::app::{App, Pane, View};
use ansi_to_tui::IntoText;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let [main_area, status_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(frame.area());

    match app.view {
        View::CourseList => render_course_list_view(app, frame, main_area),
        View::CourseContent => render_course_content_view(app, frame, main_area),
    }

    render_status(app, frame, status_area);
}

fn render_course_list_view(app: &mut App, frame: &mut Frame, area: Rect) {
    let [list_area, search_area] =
        Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(area);

    let items: Vec<ListItem> = {
        let filtered_courses = app.get_filtered_courses();
        filtered_courses
            .iter()
            .map(|(_, title)| ListItem::new(title.clone()))
            .collect()
    };

    let block = make_block("Courses", !app.is_search_mode);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, list_area, &mut app.course_state);

    let search_block = make_block("Search", app.is_search_mode);
    let search_text = Paragraph::new(app.search_query.as_str()).block(search_block);
    frame.render_widget(search_text, search_area);

    if app.is_search_mode {
        // Position cursor at end of text
        // (x + 1 for border, + length of query)
        // (y + 1 for border)
        frame.set_cursor_position((
            search_area.x + 1 + u16::try_from(app.search_query.chars().count()).unwrap(),
            search_area.y + 1,
        ));
    }
}

fn render_course_content_view(app: &mut App, frame: &mut Frame, area: Rect) {
    let [title_area, content_area] =
        Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(area);

    render_course_title(app, frame, title_area);

    // Calculate dynamic widths for chapters and lessons
    // Add 2 for borders, 2 for "> ", 3 or 4 for no., 2 for padding

    let ch_padding = if app.chapters.len() >= 10 { 10 } else { 9 };
    let max_chapter_len = app
        .chapters
        .iter()
        .map(|c| c.chars().count())
        .max()
        .unwrap_or(0)
        + ch_padding;

    let lesson_padding = if app.lessons.len() >= 10 { 10 } else { 9 };
    let max_lesson_len = app
        .lessons
        .iter()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0)
        + lesson_padding;

    let max_allowed = (area.width / 5).max(20); // min 20 chars, max 20%

    let panes = Layout::horizontal([
        Constraint::Length(
            (u16::try_from(max_chapter_len).unwrap_or(20))
                .max(20)
                .min(max_allowed),
        ),
        Constraint::Length(
            (u16::try_from(max_lesson_len).unwrap_or(20))
                .max(20)
                .min(max_allowed),
        ),
        Constraint::Min(0),
    ])
    .split(content_area);

    render_chapters(app, frame, panes[0]);
    render_lessons(app, frame, panes[1]);
    render_readme(app, frame, panes[2]);
}

fn render_course_title(app: &App, frame: &mut Frame, area: Rect) {
    let title = app
        .selected_course_title
        .as_deref()
        .unwrap_or("Unknown Course");

    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(title).block(block).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    frame.render_widget(paragraph, area);
}

fn render_chapters(app: &mut App, frame: &mut Frame, area: Rect) {
    let fallback = numbered_items(&app.chapters, None);
    let items = items_from_highlighted(&app.chapters_highlighted, fallback, None, Color::Green);

    let is_active = app.active_pane == Pane::Chapters;
    let block = make_block("Chapters", is_active);
    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut app.chapter_state);
}

fn render_lessons(app: &mut App, frame: &mut Frame, area: Rect) {
    let is_active = app.active_pane == Pane::Lessons;
    let block = make_block("Lessons", is_active);

    if app.lessons.is_empty() {
        let content = Text::from("Select a chapter").style(Style::default().fg(Color::DarkGray));
        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    } else {
        let fallback = numbered_items(&app.lessons, app.selected_lesson_no);
        let items = items_from_highlighted(
            &app.lessons_highlighted,
            fallback,
            app.selected_lesson_no,
            Color::Green,
        );

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, area, &mut app.lesson_state);
    }
}

fn render_readme(app: &App, frame: &mut Frame, area: Rect) {
    let is_active = app.active_pane == Pane::Readme;
    let block = make_block("Readme", is_active);

    let content = if app.readme.is_empty() {
        Text::from("Select a lesson").style(Style::default().fg(Color::DarkGray))
    } else {
        app.readme
            .as_bytes()
            .into_text()
            .unwrap_or_else(|_| Text::from(app.readme.as_str()))
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((u16::try_from(app.readme_scroll).unwrap(), 0));

    frame.render_widget(paragraph, area);
}

fn render_status(app: &App, frame: &mut Frame, area: Rect) {
    let help = if app.is_search_mode {
        " Esc: cancel | Enter: finish | Typing... "
    } else {
        match app.view {
            View::CourseList => " q: quit | /: search | j/k: down/up | l: select ",
            View::CourseContent => match app.active_pane {
                Pane::Chapters | Pane::Lessons => {
                    " q: quit | Esc: courses | h/l: back/forward | j/k: down/up "
                }
                Pane::Readme => " q: quit | Esc: courses | h: back | j/k: scroll ",
            },
        }
    };
    let status_line = Line::from(vec![
        ratatui::text::Span::styled(&app.status, Style::default().fg(Color::Cyan)),
        ratatui::text::Span::raw(" | "),
        ratatui::text::Span::styled(help, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default().borders(Borders::ALL);
    let paragraph = Paragraph::new(status_line).block(block);
    frame.render_widget(paragraph, area);
}

fn numbered_items(titles: &[String], selected: Option<usize>) -> Vec<ListItem<'static>> {
    titles
        .iter()
        .enumerate()
        .map(|(i, title)| {
            let mut item = ListItem::new(format!("{}. {}", i + 1, title));
            let item_no = i + 1;
            if Some(item_no) == selected {
                item = item.style(Style::default().fg(Color::Green));
            }
            item
        })
        .collect()
}

fn items_from_highlighted<'a>(
    highlighted: &'a str,
    fallback: Vec<ListItem<'a>>,
    selected: Option<usize>,
    selected_color: Color,
) -> Vec<ListItem<'a>> {
    if highlighted.is_empty() {
        return fallback;
    }

    highlighted.as_bytes().into_text().map_or_else(
        |_| fallback,
        |text| {
            text.lines
                .into_iter()
                .enumerate()
                .map(|(i, line)| {
                    let mut line = line;
                    let line_no = i + 1;
                    if Some(line_no) == selected {
                        for span in &mut line.spans {
                            span.style = span.style.fg(selected_color);
                        }
                    }
                    ListItem::new(line)
                })
                .collect()
        },
    )
}

fn make_block(title: &str, is_active: bool) -> Block<'_> {
    let border_style = if is_active {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title_style = if is_active {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    Block::default()
        .title(format!(" {} ", title))
        .title_style(title_style)
        .borders(Borders::ALL)
        .border_style(border_style)
}
