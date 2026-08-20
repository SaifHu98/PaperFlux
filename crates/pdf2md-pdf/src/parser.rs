use crate::elements::{GraphicsState, PathSegment, RawPage, TextSpan};
use crate::font::FontMap;
use pdf2md_ast::geometry::{BoundingBox, Color, Matrix, Point, Rect};
use std::collections::HashMap;

pub struct ContentStreamParser<'a> {
    fonts: &'a HashMap<String, FontMap>,
    state_stack: Vec<GraphicsState>,
    current_state: GraphicsState,
    current_path: Vec<Point>,
    page_height: f32,
}

impl<'a> ContentStreamParser<'a> {
    pub fn new(fonts: &'a HashMap<String, FontMap>, page_height: f32) -> Self {
        Self {
            fonts,
            state_stack: Vec::new(),
            current_state: GraphicsState::default(),
            current_path: Vec::new(),
            page_height,
        }
    }

    pub fn parse_content_stream(&mut self, stream_data: &[u8], page: &mut RawPage) {
        let text_content = String::from_utf8_lossy(stream_data);
        let tokens = tokenize_pdf_stream(&text_content);
        let mut i = 0;

        let mut operands = Vec::new();

        while i < tokens.len() {
            let token = &tokens[i];

            if is_operator(token) {
                self.execute_operator(token, &operands, page);
                operands.clear();
            } else {
                operands.push(token.clone());
            }

            i += 1;
        }
    }

    fn execute_operator(&mut self, op: &str, args: &[String], page: &mut RawPage) {
        match op {
            "q" => {
                self.state_stack.push(self.current_state.clone());
            }
            "Q" => {
                if let Some(state) = self.state_stack.pop() {
                    self.current_state = state;
                }
            }
            "cm" => {
                if args.len() >= 6 {
                    if let (Ok(a), Ok(b), Ok(c), Ok(d), Ok(e), Ok(f)) = (
                        args[0].parse::<f32>(),
                        args[1].parse::<f32>(),
                        args[2].parse::<f32>(),
                        args[3].parse::<f32>(),
                        args[4].parse::<f32>(),
                        args[5].parse::<f32>(),
                    ) {
                        let m = Matrix::new(a, b, c, d, e, f);
                        self.current_state.ctm = self.current_state.ctm.multiply(&m);
                    }
                }
            }
            "BT" => {
                self.current_state.text_matrix = Matrix::IDENTITY;
                self.current_state.text_line_matrix = Matrix::IDENTITY;
            }
            "ET" => {
                // Text object finished
            }
            "Tf" => {
                if args.len() >= 2 {
                    self.current_state.font_name = args[0].trim_start_matches('/').to_string();
                    if let Ok(size) = args[1].parse::<f32>() {
                        self.current_state.font_size = size;
                    }
                }
            }
            "Tm" => {
                if args.len() >= 6 {
                    if let (Ok(a), Ok(b), Ok(c), Ok(d), Ok(e), Ok(f)) = (
                        args[0].parse::<f32>(),
                        args[1].parse::<f32>(),
                        args[2].parse::<f32>(),
                        args[3].parse::<f32>(),
                        args[4].parse::<f32>(),
                        args[5].parse::<f32>(),
                    ) {
                        let m = Matrix::new(a, b, c, d, e, f);
                        self.current_state.text_matrix = m;
                        self.current_state.text_line_matrix = m;
                    }
                }
            }
            "Td" | "TD" => {
                if args.len() >= 2 {
                    if let (Ok(tx), Ok(ty)) = (args[0].parse::<f32>(), args[1].parse::<f32>()) {
                        let translation = Matrix::new(1.0, 0.0, 0.0, 1.0, tx, ty);
                        self.current_state.text_line_matrix =
                            self.current_state.text_line_matrix.multiply(&translation);
                        self.current_state.text_matrix = self.current_state.text_line_matrix;
                        if op == "TD" {
                            self.current_state.leading = -ty;
                        }
                    }
                }
            }
            "T*" => {
                let translation = Matrix::new(1.0, 0.0, 0.0, 1.0, 0.0, -self.current_state.leading);
                self.current_state.text_line_matrix =
                    self.current_state.text_line_matrix.multiply(&translation);
                self.current_state.text_matrix = self.current_state.text_line_matrix;
            }
            "TL" => {
                if let Some(arg) = args.last() {
                    if let Ok(leading) = arg.parse::<f32>() {
                        self.current_state.leading = leading;
                    }
                }
            }
            "Tc" => {
                if let Some(arg) = args.last() {
                    if let Ok(tc) = arg.parse::<f32>() {
                        self.current_state.char_spacing = tc;
                    }
                }
            }
            "Tw" => {
                if let Some(arg) = args.last() {
                    if let Ok(tw) = arg.parse::<f32>() {
                        self.current_state.word_spacing = tw;
                    }
                }
            }
            "Tj" | "'" | "\"" => {
                if let Some(str_arg) = args.last() {
                    if op == "'" || op == "\"" {
                        let translation =
                            Matrix::new(1.0, 0.0, 0.0, 1.0, 0.0, -self.current_state.leading);
                        self.current_state.text_line_matrix =
                            self.current_state.text_line_matrix.multiply(&translation);
                        self.current_state.text_matrix = self.current_state.text_line_matrix;
                    }
                    self.emit_text_string(str_arg, page);
                }
            }
            "TJ" => {
                if let Some(array_arg) = args.last() {
                    self.emit_tj_array(array_arg, page);
                }
            }
            "re" => {
                if args.len() >= 4 {
                    if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                        args[0].parse::<f32>(),
                        args[1].parse::<f32>(),
                        args[2].parse::<f32>(),
                        args[3].parse::<f32>(),
                    ) {
                        let p1 = self.current_state.ctm.transform_point(Point::new(x, y));
                        let p2 = self
                            .current_state
                            .ctm
                            .transform_point(Point::new(x + w, y + h));
                        // Convert PDF bottom-left origin to top-left origin
                        let top_y = (self.page_height - p1.y.max(p2.y)).max(0.0);
                        let rect = Rect::new(
                            p1.x.min(p2.x),
                            top_y,
                            (p2.x - p1.x).abs(),
                            (p2.y - p1.y).abs(),
                        );

                        page.paths.push(PathSegment {
                            rect: Some(rect),
                            points: vec![
                                Point::new(rect.x, rect.y),
                                Point::new(rect.x + rect.width, rect.y + rect.height),
                            ],
                            is_stroke: true,
                            is_fill: false,
                            stroke_width: self.current_state.line_width,
                            color: self.current_state.stroke_color,
                        });
                    }
                }
            }
            "m" => {
                if args.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (args[0].parse::<f32>(), args[1].parse::<f32>()) {
                        let p = self.current_state.ctm.transform_point(Point::new(x, y));
                        let top_p = Point::new(p.x, (self.page_height - p.y).max(0.0));
                        self.current_path = vec![top_p];
                    }
                }
            }
            "l" => {
                if args.len() >= 2 {
                    if let (Ok(x), Ok(y)) = (args[0].parse::<f32>(), args[1].parse::<f32>()) {
                        let p = self.current_state.ctm.transform_point(Point::new(x, y));
                        let top_p = Point::new(p.x, (self.page_height - p.y).max(0.0));
                        self.current_path.push(top_p);
                    }
                }
            }
            "S" | "s" | "f" | "F" | "f*" | "B" | "B*" | "b" | "b*" => {
                if self.current_path.len() >= 2 {
                    let is_stroke = op.contains('S')
                        || op.contains('s')
                        || op.contains('B')
                        || op.contains('b');
                    let is_fill = op.contains('f')
                        || op.contains('F')
                        || op.contains('B')
                        || op.contains('b');
                    page.paths.push(PathSegment {
                        rect: None,
                        points: self.current_path.clone(),
                        is_stroke,
                        is_fill,
                        stroke_width: self.current_state.line_width,
                        color: if is_fill {
                            self.current_state.fill_color
                        } else {
                            self.current_state.stroke_color
                        },
                    });
                }
                self.current_path.clear();
            }
            "rg" | "k" | "g" => {
                // Set fill color
                if op == "g" && !args.is_empty() {
                    if let Ok(gray) = args[0].parse::<f32>() {
                        self.current_state.fill_color = Color::rgb(gray, gray, gray);
                    }
                } else if op == "rg" && args.len() >= 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        args[0].parse::<f32>(),
                        args[1].parse::<f32>(),
                        args[2].parse::<f32>(),
                    ) {
                        self.current_state.fill_color = Color::rgb(r, g, b);
                    }
                }
            }
            "RG" | "K" | "G" => {
                // Set stroke color
                if op == "G" && !args.is_empty() {
                    if let Ok(gray) = args[0].parse::<f32>() {
                        self.current_state.stroke_color = Color::rgb(gray, gray, gray);
                    }
                } else if op == "RG" && args.len() >= 3 {
                    if let (Ok(r), Ok(g), Ok(b)) = (
                        args[0].parse::<f32>(),
                        args[1].parse::<f32>(),
                        args[2].parse::<f32>(),
                    ) {
                        self.current_state.stroke_color = Color::rgb(r, g, b);
                    }
                }
            }
            "w" => {
                if let Some(arg) = args.last() {
                    if let Ok(w) = arg.parse::<f32>() {
                        self.current_state.line_width = w;
                    }
                }
            }
            _ => {}
        }
    }

    fn emit_text_string(&mut self, encoded: &str, page: &mut RawPage) {
        let font = self.fonts.get(&self.current_state.font_name);
        let decoded_text = decode_pdf_string(encoded, font);
        if decoded_text.is_empty() {
            return;
        }

        let font_size = self.current_state.font_size;
        let eff_matrix = self
            .current_state
            .text_matrix
            .multiply(&self.current_state.ctm);
        let start_point = eff_matrix.transform_point(Point::ZERO);

        let char_count = decoded_text.chars().count().max(1);
        let approx_width = (char_count as f32) * font_size * 0.55 * eff_matrix.scale_x();
        let approx_height = font_size * eff_matrix.scale_y();

        // Convert PDF coordinate system (Y=0 at bottom) to Top-Left origin
        let top_y = (self.page_height - start_point.y - approx_height).max(0.0);
        let bbox = BoundingBox::new(start_point.x, top_y, approx_width, approx_height);

        let is_bold = font.map(|f| f.is_bold).unwrap_or(false);
        let is_italic = font.map(|f| f.is_italic).unwrap_or(false);
        let is_monospace = font.map(|f| f.is_monospace).unwrap_or(false);

        let direction = if pdf2md_text::bidi::contains_rtl(&decoded_text) {
            pdf2md_ast::WritingDirection::RightToLeft
        } else {
            pdf2md_ast::WritingDirection::LeftToRight
        };

        let baseline = pdf2md_ast::Baseline::new(
            top_y + approx_height,
            start_point.x,
            start_point.x + approx_width,
        );

        page.text_spans.push(TextSpan {
            text: decoded_text,
            bbox,
            baseline,
            font_name: self.current_state.font_name.clone(),
            font_size,
            is_bold,
            is_italic,
            is_monospace,
            color: self.current_state.fill_color,
            matrix: eff_matrix,
            char_spacing: self.current_state.char_spacing,
            word_spacing: self.current_state.word_spacing,
            leading: self.current_state.leading,
            direction,
        });

        // Advance text matrix
        let advance = Matrix::new(
            1.0,
            0.0,
            0.0,
            1.0,
            (char_count as f32) * font_size * 0.55,
            0.0,
        );
        self.current_state.text_matrix = self.current_state.text_matrix.multiply(&advance);
    }

    fn emit_tj_array(&mut self, array_str: &str, page: &mut RawPage) {
        let items = parse_tj_elements(array_str);
        for item in items {
            match item {
                TjItem::String(s) => self.emit_text_string(&s, page),
                TjItem::Adjustment(adj) => {
                    let dx = -adj / 1000.0 * self.current_state.font_size;
                    let advance = Matrix::new(1.0, 0.0, 0.0, 1.0, dx, 0.0);
                    self.current_state.text_matrix =
                        self.current_state.text_matrix.multiply(&advance);
                }
            }
        }
    }
}

enum TjItem {
    String(String),
    Adjustment(f32),
}

fn parse_tj_elements(raw: &str) -> Vec<TjItem> {
    let mut items = Vec::new();
    let content = raw.trim().trim_start_matches('[').trim_end_matches(']');
    let mut chars = content.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch == '(' {
            chars.next();
            let mut s = String::new();
            let mut depth = 1;
            while let Some(c) = chars.next() {
                if c == '\\' {
                    if let Some(escaped) = chars.next() {
                        s.push(escaped);
                    }
                } else if c == '(' {
                    depth += 1;
                    s.push(c);
                } else if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    s.push(c);
                } else {
                    s.push(c);
                }
            }
            items.push(TjItem::String(format!("({})", s)));
        } else if ch == '<' {
            chars.next();
            let mut hex = String::new();
            for c in chars.by_ref() {
                if c == '>' {
                    break;
                }
                hex.push(c);
            }
            items.push(TjItem::String(format!("<{}>", hex)));
        } else if ch.is_whitespace() {
            chars.next();
        } else {
            let mut num_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '-' || c == '.' {
                    num_str.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            if let Ok(val) = num_str.parse::<f32>() {
                items.push(TjItem::Adjustment(val));
            }
        }
    }

    items
}

fn is_operator(token: &str) -> bool {
    matches!(
        token,
        "q" | "Q"
            | "cm"
            | "BT"
            | "ET"
            | "Tf"
            | "Tm"
            | "Td"
            | "TD"
            | "T*"
            | "TL"
            | "Tc"
            | "Tw"
            | "Tj"
            | "TJ"
            | "'"
            | "\""
            | "re"
            | "m"
            | "l"
            | "c"
            | "v"
            | "y"
            | "h"
            | "S"
            | "s"
            | "f"
            | "F"
            | "f*"
            | "B"
            | "B*"
            | "b"
            | "b*"
            | "rg"
            | "k"
            | "g"
            | "RG"
            | "K"
            | "G"
            | "w"
            | "Do"
    )
}

fn tokenize_pdf_stream(stream: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut chars = stream.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
        } else if ch == '(' {
            chars.next();
            let mut s = String::new();
            s.push('(');
            let mut depth = 1;
            let mut escaped = false;
            for c in chars.by_ref() {
                s.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '(' {
                    depth += 1;
                } else if c == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
            }
            tokens.push(s);
        } else if ch == '[' {
            chars.next();
            let mut s = String::new();
            s.push('[');
            for c in chars.by_ref() {
                s.push(c);
                if c == ']' {
                    break;
                }
            }
            tokens.push(s);
        } else if ch == '<' {
            chars.next();
            let mut s = String::new();
            s.push('<');
            for c in chars.by_ref() {
                s.push(c);
                if c == '>' {
                    break;
                }
            }
            tokens.push(s);
        } else {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() || c == '(' || c == '[' || c == '<' || c == '/' {
                    break;
                }
                token.push(chars.next().unwrap());
            }
            if !token.is_empty() {
                tokens.push(token);
            } else if ch == '/' {
                let mut slash_token = String::new();
                slash_token.push(chars.next().unwrap());
                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || c == '(' || c == '[' || c == '<' || c == '/' {
                        break;
                    }
                    slash_token.push(chars.next().unwrap());
                }
                tokens.push(slash_token);
            }
        }
    }

    tokens
}

fn decode_pdf_string(encoded: &str, font: Option<&FontMap>) -> String {
    if encoded.starts_with('(') && encoded.ends_with(')') {
        let inner = &encoded[1..encoded.len() - 1];
        if let Some(f) = font {
            if f.to_unicode.is_empty() && inner.chars().any(|c| (c as u32) > 127) {
                // String is already UTF-8 Unicode (e.g. Arabic text stream)
                let mut out = String::new();
                for ch in inner.chars() {
                    out.push_str(&f.decode_code(ch as u32));
                }
                out
            } else {
                let mut out = String::new();
                for b in inner.bytes() {
                    out.push_str(&f.decode_code(b as u32));
                }
                out
            }
        } else {
            inner.to_string()
        }
    } else if encoded.starts_with('<') && encoded.ends_with('>') {
        let hex = &encoded[1..encoded.len() - 1];
        let mut out = String::new();
        if let Some(f) = font {
            if hex.len().is_multiple_of(4) {
                for chunk in hex.as_bytes().chunks(4) {
                    if let Ok(c_str) = std::str::from_utf8(chunk) {
                        if let Ok(code) = u32::from_str_radix(c_str, 16) {
                            out.push_str(&f.decode_code(code));
                        }
                    }
                }
            } else {
                for chunk in hex.as_bytes().chunks(2) {
                    if let Ok(c_str) = std::str::from_utf8(chunk) {
                        if let Ok(code) = u32::from_str_radix(c_str, 16) {
                            out.push_str(&f.decode_code(code));
                        }
                    }
                }
            }
        } else {
            for chunk in hex.as_bytes().chunks(2) {
                if let Ok(c_str) = std::str::from_utf8(chunk) {
                    if let Ok(code) = u32::from_str_radix(c_str, 16) {
                        if let Some(ch) = char::from_u32(code) {
                            out.push(ch);
                        }
                    }
                }
            }
        }
        out
    } else {
        encoded.to_string()
    }
}
