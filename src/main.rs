mod models;
mod registry;
mod exporters;

use colored::Colorize;
use inquire::{Select, Text};
use uuid::Uuid;
use chrono::Local;
use std::{io, process, env, path::{Path, PathBuf}};

use models::{Project, ProjectMetadata};
use registry::StructureRegistry;
use exporters::Exporters;

use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style, Stylize},
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

fn display_header(title: &str) {
    clear_screen();
    println!("{}", "=".repeat(60).cyan());
    println!("  {}", Colorize::bold(title).cyan());
    println!("{}", "=".repeat(60).cyan());
    println!();
}

fn main() {
    let registry = StructureRegistry::new();
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let path = PathBuf::from(&args[1]);
        if path.exists() {
            if let Some(project) = load_from_path(&path) {
                zen_wizard(project, path);
            } else {
                println!("{}", Colorize::red("Error: Provided file is not a valid project JSON."));
            }
        } else {
            // New project with specific filename
            if let Some(project) = new_project_wizard(&registry, &path) {
                save_to_path(&project, &path);
                zen_wizard(project, path);
            }
        }
    } else {
        // No args: Show menu
        loop {
            display_header("STRUCTURE YOUR STORY");
            let options = vec!["New Project", "Browse Structures", "Quit"];
            match Select::new("Choose an option:", options).prompt() {
                Ok("New Project") => {
                    let title = match Text::new("Project Filename (e.g. story.json):").prompt() {
                        Ok(t) if !t.trim().is_empty() => t,
                        _ => continue,
                    };
                    let path = PathBuf::from(title);
                    if let Some(project) = new_project_wizard(&registry, &path) {
                        save_to_path(&project, &path);
                        zen_wizard(project, path);
                    }
                }
                Ok("Browse Structures") => browse_structures(&registry),
                Ok("Quit") | Err(_) => process::exit(0),
                _ => {}
            }
        }
    }
}

fn load_from_path(path: &Path) -> Option<Project> {
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str::<Project>(&content).ok()
    } else {
        None
    }
}

fn save_to_path(project: &Project, path: &Path) {
    let mut updated_project = project.clone();
    updated_project.updated_at = Local::now();
    if let Ok(json) = serde_json::to_string_pretty(&updated_project) {
        let _ = std::fs::write(path, json);
    }
}

fn new_project_wizard(registry: &StructureRegistry, _path: &Path) -> Option<Project> {
    display_header("New Project Setup");
    
    let mediums = vec!["Screenplay", "Novel", "Back"];
    let medium_choice = Select::new("Select Medium:", mediums).prompt().ok()?;
    if medium_choice == "Back" { return None; }
    
    let medium = medium_choice.to_lowercase();
    let structures = registry.get_by_medium(&medium);
    let structure_options: Vec<String> = structures.iter()
        .map(|s| format!("{} ({} steps)", s.name, s.steps.len()))
        .collect();
    
    let ans = Select::new("Select Structure:", structure_options).prompt().ok()?;
    let s_idx = structures.iter().position(|s| format!("{} ({} steps)", s.name, s.steps.len()) == ans)?;
    let structure = &structures[s_idx];
    
    let title = Text::new("Project Title:").prompt().ok()?;
    if title.trim().is_empty() { return None; }
    
    let genre = Text::new("Genre (optional):").prompt().unwrap_or_default();
    let length = Text::new("Estimated Length (optional):").prompt().unwrap_or_default();
    let logline = Text::new("Logline / Premise (optional):").prompt().unwrap_or_default();
    
    let metadata = ProjectMetadata {
        title,
        genre,
        logline,
        estimated_length: length,
        notes: String::new(),
    };
    
    let steps = structure.steps.iter().map(|s| {
        let mut step = s.clone();
        step.status = "empty".to_string();
        step.content = String::new();
        step
    }).collect();

    Some(Project {
        id: Uuid::new_v4().simple().to_string(),
        app_name: "Structure Your Story".to_string(),
        app_version: "0.1.0".to_string(),
        medium,
        structure_id: structure.id.clone(),
        structure_name: structure.name.clone(),
        metadata,
        steps,
        created_at: Local::now(),
        updated_at: Local::now(),
    })
}

fn browse_structures(registry: &StructureRegistry) {
    display_header("Story Structures");
    let structures = registry.get_all();
    let mut options: Vec<String> = structures.iter().map(|s| format!("{} ({})", s.name, s.mediums[0])).collect();
    options.push("Back".to_string());
    
    loop {
        match Select::new("Select to view details:", options.clone()).prompt() {
            Ok(ans) if ans != "Back" => {
                let idx = structures.iter().position(|s| format!("{} ({})", s.name, s.mediums[0]) == ans).unwrap();
                let s = &structures[idx];
                display_header(&s.name.to_uppercase());
                println!("{}: {}", Colorize::bold("Author"), s.author);
                println!("{}: {}", Colorize::bold("Type"), s.structure_type);
                println!("{}: {}", Colorize::bold("Complexity"), s.complexity);
                println!("\n{}\n", Colorize::italic(s.description.as_str()));
                println!("{}: {}", Colorize::bold(Colorize::green("Best For")), s.best_for);
                println!("{}: {}", Colorize::bold(Colorize::red("Avoid If")), s.avoid_if);
                let _ = Text::new("Press Enter to return...").prompt();
            }
            _ => break,
        }
    }
}

fn export_project(project: &Project, path: &Path) {
    let options = vec!["PDF Summary", "Markdown", "JSON", "Plain Text", "Cancel"];
    if let Ok(choice) = Select::new("Choose export format:", options).prompt() {
        if choice == "Cancel" { return; }
        
        let target_dir = path.parent().unwrap_or_else(|| Path::new("."));
        let exported_path = match choice {
            "PDF Summary" => Exporters::export_pdf_summary(project, target_dir),
            "Markdown" => Exporters::export_markdown(project, target_dir),
            "JSON" => Exporters::export_json(project, target_dir),
            "Plain Text" => Exporters::export_text(project, target_dir),
            _ => None,
        };
        if let Some(p) = exported_path {
            println!("\n{} {}", Colorize::green(Colorize::bold("Successfully exported to:")), p);
            let _ = Text::new("Press Enter to continue...").prompt();
        }
    }
}

fn zen_wizard(mut project: Project, path: PathBuf) {
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    let mut current_idx = 0;
    
    loop {
        let total_steps = project.steps.len();
        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints([Constraint::Length(3), Constraint::Length(7), Constraint::Min(10), Constraint::Length(3)])
                .split(size);

            let title_text = format!(" STRUCTURE YOUR STORY — {} ", project.metadata.title.to_uppercase());
            let title = Paragraph::new(title_text)
                .alignment(ratatui::layout::Alignment::Center)
                .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Cyan)))
                .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
            f.render_widget(title, chunks[0]);

            let step = &project.steps[current_idx];
            let step_info = vec![
                ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled("BEAT: ", Style::default().fg(Color::DarkGray)),
                    ratatui::text::Span::styled(&step.name, Style::default().add_modifier(Modifier::BOLD).fg(Color::White)),
                    ratatui::text::Span::styled(format!("  ({})", step.target), Style::default().fg(Color::Yellow)),
                ]),
                ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled("PROMPT: ", Style::default().fg(Color::DarkGray)),
                    ratatui::text::Span::styled(&step.prompt, Style::default().italic().fg(Color::Gray)),
                ]),
                ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled("PROGRESS: ", Style::default().fg(Color::DarkGray)),
                    ratatui::text::Span::styled(format!("{}/{}", current_idx + 1, total_steps), Style::default().fg(Color::Cyan)),
                ]),
            ];
            let info = Paragraph::new(step_info)
                .block(Block::default().title(" CURRENT BEAT ").borders(Borders::ALL).border_style(Style::default().fg(Color::Cyan)));
            f.render_widget(info, chunks[1]);

            let mut display_content = step.content.clone();
            display_content.push('█');
            let editor = Paragraph::new(display_content)
                .block(Block::default().title(" WRITING SPACE ").borders(Borders::ALL).border_style(Style::default().fg(Color::White)))
                .wrap(Wrap { trim: false });
            f.render_widget(editor, chunks[2]);

            let footer_text = " TAB: Next  SHIFT+TAB: Prev  ENTER: New Line  ESC: Save & Quit  CTRL+X: Export ";
            let footer = Paragraph::new(footer_text).alignment(ratatui::layout::Alignment::Center).style(Style::default().fg(Color::DarkGray));
            f.render_widget(footer, chunks[3]);
        }).unwrap();

        if event::poll(std::time::Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Esc => {
                        save_to_path(&project, &path);
                        break;
                    }
                    KeyCode::Tab => {
                        save_to_path(&project, &path);
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            if current_idx > 0 { current_idx -= 1; }
                        } else {
                            if current_idx < total_steps - 1 { current_idx += 1; }
                        }
                    }
                    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        save_to_path(&project, &path);
                        disable_raw_mode().unwrap();
                        execute!(io::stdout(), LeaveAlternateScreen).unwrap();
                        export_project(&project, &path);
                        enable_raw_mode().unwrap();
                        execute!(io::stdout(), EnterAlternateScreen).unwrap();
                    }
                    KeyCode::Char(c) => {
                        project.steps[current_idx].content.push(c);
                        project.steps[current_idx].status = "done".to_string();
                    }
                    KeyCode::Enter => {
                        project.steps[current_idx].content.push('\n');
                    }
                    KeyCode::Backspace => {
                        if !project.steps[current_idx].content.is_empty() {
                            project.steps[current_idx].content.pop();
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    disable_raw_mode().unwrap();
    execute!(io::stdout(), LeaveAlternateScreen).unwrap();
}
