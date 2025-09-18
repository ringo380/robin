// Robin Game Engine - Data Display Components
// Tables, lists, cards, and data visualization components

use super::*;
use crate::engine::error::RobinResult;
use crate::engine::generation::templates::UITheme;
use crate::engine::ui::modern_architecture::ComponentContext;
use serde::{Serialize, Deserialize};

/// Table component for displaying structured data
#[derive(Debug)]
pub struct Table {
    id: ComponentId,
    props: TableProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableProps {
    pub base: ComponentProps,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<TableRow>,
    pub sortable: bool,
    pub selectable: bool,
    pub pagination: Option<PaginationProps>,
    pub loading: bool,
    pub empty_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub key: String,
    pub title: String,
    pub sortable: bool,
    pub width: Option<String>,
    pub align: TextAlign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRow {
    pub id: String,
    pub cells: Vec<TableCell>,
    pub selected: bool,
    pub disabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    pub content: String,
    pub value: Option<String>, // For sorting
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationProps {
    pub current_page: usize,
    pub total_pages: usize,
    pub page_size: usize,
    pub total_items: usize,
}

impl Table {
    pub fn new(props: TableProps) -> Self {
        let id = utils::generate_component_id("table");
        Self { id, props }
    }
}

impl Component for Table {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput::with_legacy_fields(
            "table".to_string(),
            "Table Component".to_string(),
            Style::default(),
            HashMap::new(),
            Vec::new(),
        ))
    }

    fn type_name(&self) -> &'static str {
        "Table"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Table {
    fn component_type(&self) -> &'static str { "table" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}

/// Card component for displaying content in a card layout
#[derive(Debug)]
pub struct Card {
    id: ComponentId,
    props: CardProps,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardProps {
    pub base: ComponentProps,
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub image: Option<String>,
    pub variant: CardVariant,
    pub elevation: u8,
    pub clickable: bool,
    pub loading: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CardVariant {
    Default,
    Outlined,
    Elevated,
    Flat,
}

impl Card {
    pub fn new(props: CardProps) -> Self {
        let id = utils::generate_component_id("card");
        Self { id, props }
    }
}

impl Component for Card {
    fn id(&self) -> ComponentId { self.id.clone() }
    fn render(&self, _ctx: &RenderContext) -> RobinResult<RenderOutput> {
        Ok(RenderOutput::with_legacy_fields(
            "div".to_string(),
            "Card Component".to_string(),
            Style::default(),
            HashMap::new(),
            Vec::new(),
        ))
    }

    fn type_name(&self) -> &'static str {
        "Card"
    }

    fn init(&mut self, _ctx: &mut ComponentContext) -> RobinResult<()> {
        Ok(())
    }

    fn update(&mut self, _ctx: &mut ComponentContext, _delta_time: f32) -> RobinResult<()> {
        Ok(())
    }
}

impl UIComponent for Card {
    fn component_type(&self) -> &'static str { "card" }
    fn props(&self) -> &ComponentProps { &self.props.base }
    fn set_props(&mut self, props: ComponentProps) -> RobinResult<()> {
        self.props.base = props;
        Ok(())
    }
    fn handle_event(&mut self, _event: UIEvent) -> RobinResult<Vec<UIEvent>> { Ok(Vec::new()) }
    fn accessibility_info(&self) -> AccessibilityInfo { AccessibilityInfo::default() }
    fn apply_theme(&mut self, _theme: &UITheme) -> RobinResult<()> { Ok(()) }
}