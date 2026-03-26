use crate::models::Project;
use std::fs;
use std::path::Path;
use printpdf::*;
use std::io::BufWriter;
use std::fs::File;

pub struct Exporters;

impl Exporters {
    // (ensure_exports_dir removed as we use user-selected directories)

    pub fn export_markdown(project: &Project, target_dir: &Path) -> Option<String> {
        let filename = format!("{}.md", project.id);
        let path = target_dir.join(filename);
        
        let mut lines = Vec::new();
        lines.push(format!("# {}", if project.metadata.title.is_empty() { "Untitled Project" } else { &project.metadata.title }));
        lines.push(String::new());
        lines.push(format!("**App:** {}", project.app_name));
        lines.push(format!("**Medium:** {}", project.medium));
        lines.push(format!("**Structure:** {}", project.structure_name));
        lines.push(format!("**Genre:** {}", project.metadata.genre));
        lines.push(format!("**Estimated Length:** {}", project.metadata.estimated_length));
        lines.push(String::new());
        lines.push("## Logline / Premise".to_string());
        lines.push(if project.metadata.logline.is_empty() { "N/A".to_string() } else { project.metadata.logline.clone() });
        lines.push(String::new());
        lines.push("## Story Beats".to_string());
        lines.push(String::new());

        for (i, step) in project.steps.iter().enumerate() {
            lines.push(format!("### {}. {} ({})", i + 1, step.name, step.target));
            lines.push(format!("*Prompt: {}*", step.prompt));
            lines.push(String::new());
            lines.push(if step.content.is_empty() { "*(No content)*".to_string() } else { step.content.clone() });
            lines.push(String::new());
        }

        if fs::write(&path, lines.join("\n")).is_ok() {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }

    pub fn export_text(project: &Project, target_dir: &Path) -> Option<String> {
        let filename = format!("{}.txt", project.id);
        let path = target_dir.join(filename);
        
        let mut lines = Vec::new();
        lines.push(format!("STRUCTURE YOUR STORY: {}", project.metadata.title.to_uppercase()));
        lines.push("=".repeat(50));
        lines.push(format!("Medium: {}", project.medium));
        lines.push(format!("Structure: {}", project.structure_name));
        lines.push(format!("Genre: {}", project.metadata.genre));
        lines.push(format!("Length: {}", project.metadata.estimated_length));
        lines.push(String::new());
        lines.push("LOGLINE:".to_string());
        lines.push(project.metadata.logline.clone());
        lines.push(String::new());
        lines.push("STORY BEATS:".to_string());
        lines.push("-".repeat(50));
        lines.push(String::new());

        for (i, step) in project.steps.iter().enumerate() {
            lines.push(format!("{}. {} ({})", i + 1, step.name.to_uppercase(), step.target));
            lines.push(format!("Prompt: {}", step.prompt));
            lines.push("-".repeat(20));
            lines.push(if step.content.is_empty() { "(No content)".to_string() } else { step.content.clone() });
            lines.push(String::new());
        }

        if fs::write(&path, lines.join("\n")).is_ok() {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }

    pub fn export_json(project: &Project, target_dir: &Path) -> Option<String> {
        let filename = format!("{}_export.json", project.id);
        let path = target_dir.join(filename);
        if let Ok(json) = serde_json::to_string_pretty(project) {
            if fs::write(&path, json).is_ok() {
                return Some(path.to_string_lossy().to_string());
            }
        }
        None
    }
    
    fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
        let mut lines = Vec::new();
        for paragraph in text.lines() {
            if paragraph.is_empty() {
                lines.push(String::new());
                continue;
            }
            let mut current_line = String::new();
            for word in paragraph.split_whitespace() {
                if current_line.len() + word.len() + 1 > max_chars {
                    lines.push(current_line.trim().to_string());
                    current_line = word.to_string();
                } else {
                    if !current_line.is_empty() {
                        current_line.push(' ');
                    }
                    current_line.push_str(word);
                }
            }
            if !current_line.is_empty() {
                lines.push(current_line.trim().to_string());
            }
        }
        lines
    }

    fn get_text_width(text: &str, font_size: f32) -> f32 {
        let mut total_width = 0.0;
        for c in text.chars() {
            total_width += match c {
                'W' | 'M' => 0.85,
                'w' | 'm' => 0.70,
                'i' | 'l' | 'j' | 'f' | 't' | 'I' | ' ' | '.' | ',' | ':' | ';' | '!' | '\'' => 0.25,
                'A'..='Z' => 0.65,
                'a'..='z' => 0.45,
                '0'..='9' => 0.50,
                _ => 0.50,
            };
        }
        total_width * font_size * 0.3527 // mm per point
    }

    pub fn export_pdf_summary(project: &Project, target_dir: &Path) -> Option<String> {
        let filename = format!("{}_summary.pdf", project.id);
        let path = target_dir.join(filename);
        
        let (doc, page1, layer1) = PdfDocument::new(&project.metadata.title, Mm(210.0), Mm(297.0), "Base");
        
        let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();
        let font_italic = doc.add_builtin_font(BuiltinFont::HelveticaOblique).unwrap();

        let left_margin = 25.0;
        let right_margin = 185.0;
        let mut page_count = 1;

        // Helper to draw structure name at top right (on every page)
        let draw_structure_header = |layer: &PdfLayerReference, structure: &str| {
            layer.set_fill_color(Color::Rgb(Rgb::new(0.6, 0.6, 0.6, None)));
            let width = Self::get_text_width(structure, 10.0);
            layer.use_text(structure, 10.0, Mm(right_margin - width), Mm(285.0), &font_italic);
        };

        // Helper to draw footer
        let draw_footer = |layer: &PdfLayerReference, page_num: usize| {
            layer.set_fill_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.7, None)));
            // Centered Page Number
            let p_text = format!("Page {}", page_num);
            let p_width = Self::get_text_width(&p_text, 8.0);
            layer.use_text(p_text, 8.0, Mm(105.0 - p_width / 2.0), Mm(10.0), &font);
            
            // Subtle GitHub Link (Left)
            layer.use_text("github.com/BeetleBot/StructureYourStory", 8.0, Mm(left_margin), Mm(10.0), &font);
        };

        let mut current_page = page1;
        let mut current_layer = doc.get_page(current_page).get_layer(layer1);
        draw_structure_header(&current_layer, &project.structure_name);
        
        let mut y = 265.0;

        // 1. Centered Title Section (First Page Only)
        // Huge Title
        current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        let title = project.metadata.title.to_uppercase();
        let title_width = Self::get_text_width(&title, 28.0);
        current_layer.use_text(title, 28.0, Mm(105.0 - title_width / 2.0), Mm(y), &font_bold);
        y -= 8.0; // Aggressively condensed

        // Subtitle: Medium | Genre | Length
        let mut subtitle_parts = vec![project.medium.to_uppercase()];
        if !project.metadata.genre.is_empty() { subtitle_parts.push(project.metadata.genre.clone()); }
        if !project.metadata.estimated_length.is_empty() { subtitle_parts.push(project.metadata.estimated_length.clone()); }
        
        let subtitle = subtitle_parts.join("  |  ");
        let sub_width = Self::get_text_width(&subtitle, 12.0);
        current_layer.use_text(subtitle, 12.0, Mm(105.0 - sub_width / 2.0), Mm(y), &font_bold);
        y -= 6.0; // Aggressively condensed

        // Logline
        if !project.metadata.logline.is_empty() {
            let wrapped_logline = Self::wrap_text(&project.metadata.logline, 75);
            for line in wrapped_logline {
                let text_width = Self::get_text_width(&line, 11.0);
                current_layer.use_text(line, 11.0, Mm(105.0 - text_width / 2.0), Mm(y), &font_italic);
                y -= 4.8; // Aggressively condensed line height
            }
            y -= 4.0; // Aggressively condensed bottom gap
        }

        // Thick Separator
        current_layer.set_outline_thickness(1.0);
        current_layer.add_line(Line {
            points: vec![(Point::new(Mm(left_margin), Mm(y)), false), (Point::new(Mm(right_margin), Mm(y)), false)],
            is_closed: false,
        });
        y -= 10.0; // Aggressively reduced gap to first beat

        // 2. Beats
        for (i, step) in project.steps.iter().enumerate() {
            // Check for page break
            if y < 30.0 {
                draw_footer(&current_layer, page_count);
                let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                current_page = new_page;
                current_layer = doc.get_page(current_page).get_layer(new_layer);
                page_count += 1;
                draw_structure_header(&current_layer, &project.structure_name);
                y = 280.0;
            }

            // Numbering fix
            let clean_name = if step.name.chars().next().map_or(false, |c| c.is_ascii_digit()) && step.name.contains(". ") {
                step.name.splitn(2, ". ").nth(1).unwrap_or(&step.name).to_uppercase()
            } else {
                step.name.to_uppercase()
            };

            // Beat Header (Minimalist)
            current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
            let beat_title = format!("{}. {}", i + 1, clean_name);
            current_layer.use_text(&beat_title, 12.0, Mm(left_margin), Mm(y), &font_bold);
            
            current_layer.set_fill_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
            let target_text = format!("({})", step.target);
            let target_width = Self::get_text_width(&target_text, 10.0);
            current_layer.use_text(target_text, 10.0, Mm(right_margin - target_width), Mm(y), &font_italic);
            y -= 5.0; // Aggressively condensed gap header to content

            // Beat Content
            current_layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
            let content = if step.content.is_empty() { "(Empty)" .to_string() } else { step.content.clone() };
            let wrapped_content = Self::wrap_text(&content, 82);
            
            for (line_idx, line) in wrapped_content.iter().enumerate() {
                if line.is_empty() && line_idx > 0 {
                    y -= 3.0;
                    continue;
                }
                current_layer.use_text(line, 10.5, Mm(left_margin), Mm(y), &font);
                y -= 5.0; // Aggressively condensed line height

                if y < 15.0 {
                    draw_footer(&current_layer, page_count);
                    let (new_page, new_layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                    current_page = new_page;
                    current_layer = doc.get_page(current_page).get_layer(new_layer);
                    page_count += 1;
                    draw_structure_header(&current_layer, &project.structure_name);
                    y = 280.0;
                }
            }
            y -= 6.0; // Aggressively reduced space between beats
        }

        draw_footer(&current_layer, page_count);

        let file = File::create(&path).ok()?;
        let mut writer = BufWriter::new(file);
        if doc.save(&mut writer).is_ok() {
            Some(path.to_string_lossy().to_string())
        } else {
            None
        }
    }
}
