use crate::engine::{
    math::Vec2,
    ui::{UIBounds, Anchor, ElementId},
};
use std::collections::HashMap;

/// Layout direction for flex containers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,        // Horizontal
    Column,     // Vertical
    RowReverse, // Horizontal reversed
    ColumnReverse, // Vertical reversed
}

/// Justification for main axis alignment
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart,    // Pack to start
    FlexEnd,      // Pack to end
    Center,       // Pack to center
    SpaceBetween, // Distribute with space between
    SpaceAround,  // Distribute with space around
    SpaceEvenly,  // Distribute evenly
}

/// Alignment for cross axis
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    FlexStart, // Align to start
    FlexEnd,   // Align to end
    Center,    // Align to center
    Stretch,   // Stretch to fill
}

/// Flex wrap behavior
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexWrap {
    NoWrap, // Single line
    Wrap,   // Multi-line
}

/// Layout constraints for elements
#[derive(Debug, Clone)]
pub struct LayoutConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
    pub aspect_ratio: Option<f32>,
}

impl Default for LayoutConstraints {
    fn default() -> Self {
        Self {
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            aspect_ratio: None,
        }
    }
}

/// Flex item properties
#[derive(Debug, Clone)]
pub struct FlexItem {
    pub flex_grow: f32,   // How much the item should grow
    pub flex_shrink: f32, // How much the item should shrink
    pub flex_basis: Option<f32>, // Initial size before growing/shrinking
    pub align_self: Option<AlignItems>, // Override container alignment
    pub order: i32,       // Display order (for reordering)
}

impl Default for FlexItem {
    fn default() -> Self {
        Self {
            flex_grow: 0.0,
            flex_shrink: 1.0,
            flex_basis: None,
            align_self: None,
            order: 0,
        }
    }
}

/// Container layout properties
#[derive(Debug, Clone)]
pub struct LayoutContainer {
    pub element_id: ElementId,
    pub bounds: UIBounds,
    pub children: Vec<ElementId>,
    pub layout_type: LayoutType,
    pub constraints: LayoutConstraints,
}

/// Different layout types
#[derive(Debug, Clone)]
pub enum LayoutType {
    /// No specific layout
    None,
    /// Flexbox layout
    Flex {
        direction: FlexDirection,
        justify_content: JustifyContent,
        align_items: AlignItems,
        wrap: FlexWrap,
        gap: f32,
    },
    /// Grid layout
    Grid {
        columns: u32,
        rows: u32,
        column_gap: f32,
        row_gap: f32,
        auto_flow: GridAutoFlow,
    },
    /// Stack layout (elements on top of each other)
    Stack {
        alignment: Anchor,
        spacing: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GridAutoFlow {
    Row,
    Column,
}

/// Layout manager for handling UI element positioning and sizing
pub struct LayoutManager {
    containers: HashMap<ElementId, LayoutContainer>,
    items: HashMap<ElementId, FlexItem>,
    element_bounds: HashMap<ElementId, UIBounds>,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            containers: HashMap::new(),
            items: HashMap::new(),
            element_bounds: HashMap::new(),
        }
    }

    /// Register an element with the layout system
    pub fn register_element(&mut self, id: ElementId, bounds: UIBounds) {
        self.element_bounds.insert(id, bounds);
    }

    /// Create a flex container
    pub fn create_flex_container(
        &mut self,
        id: ElementId,
        bounds: UIBounds,
        direction: FlexDirection,
        justify_content: JustifyContent,
        align_items: AlignItems,
    ) {
        let container = LayoutContainer {
            element_id: id,
            bounds: bounds.clone(),
            children: Vec::new(),
            layout_type: LayoutType::Flex {
                direction,
                justify_content,
                align_items,
                wrap: FlexWrap::NoWrap,
                gap: 0.0,
            },
            constraints: LayoutConstraints::default(),
        };
        
        self.containers.insert(id, container);
        self.element_bounds.insert(id, bounds);
    }

    /// Create a grid container
    pub fn create_grid_container(
        &mut self,
        id: ElementId,
        bounds: UIBounds,
        columns: u32,
        rows: u32,
        column_gap: f32,
        row_gap: f32,
    ) {
        let container = LayoutContainer {
            element_id: id,
            bounds: bounds.clone(),
            children: Vec::new(),
            layout_type: LayoutType::Grid {
                columns,
                rows,
                column_gap,
                row_gap,
                auto_flow: GridAutoFlow::Row,
            },
            constraints: LayoutConstraints::default(),
        };
        
        self.containers.insert(id, container);
        self.element_bounds.insert(id, bounds);
    }

    /// Add a child to a container
    pub fn add_child(&mut self, container_id: ElementId, child_id: ElementId) {
        if let Some(container) = self.containers.get_mut(&container_id) {
            if !container.children.contains(&child_id) {
                container.children.push(child_id);
                log::debug!("Added child {} to container {}", child_id, container_id);
            }
        }
    }

    /// Set flex properties for an item
    pub fn set_flex_item(&mut self, id: ElementId, flex_item: FlexItem) {
        self.items.insert(id, flex_item);
    }

    /// Perform layout calculations for all containers
    pub fn layout(&mut self, screen_size: Vec2) {
        // Sort containers by dependency (parents before children)
        let container_ids: Vec<ElementId> = self.containers.keys().copied().collect();
        
        for &container_id in &container_ids {
            self.layout_container(container_id, screen_size);
        }
    }

    /// Layout a specific container
    fn layout_container(&mut self, container_id: ElementId, screen_size: Vec2) {
        let container = match self.containers.get(&container_id).cloned() {
            Some(c) => c,
            None => return,
        };

        match container.layout_type {
            LayoutType::Flex { direction, justify_content, align_items, wrap: _, gap } => {
                self.layout_flex_container(&container, direction, justify_content, align_items, gap);
            }
            LayoutType::Grid { columns, rows, column_gap, row_gap, auto_flow: _ } => {
                self.layout_grid_container(&container, columns, rows, column_gap, row_gap);
            }
            LayoutType::Stack { alignment, spacing } => {
                self.layout_stack_container(&container, alignment, spacing);
            }
            LayoutType::None => {
                // No layout - elements keep their specified positions
            }
        }
    }

    /// Layout flex container
    fn layout_flex_container(
        &mut self,
        container: &LayoutContainer,
        direction: FlexDirection,
        justify_content: JustifyContent,
        align_items: AlignItems,
        gap: f32,
    ) {
        if container.children.is_empty() {
            return;
        }

        let is_horizontal = matches!(direction, FlexDirection::Row | FlexDirection::RowReverse);
        let container_bounds = &container.bounds;
        
        // Calculate available space
        let available_width = container_bounds.size.x;
        let available_height = container_bounds.size.y;
        let available_main_size = if is_horizontal { available_width } else { available_height };
        
        // Collect child information
        let mut child_sizes = Vec::new();
        let mut total_flex_grow = 0.0;
        let mut used_space = 0.0;
        
        for &child_id in &container.children {
            if let Some(child_bounds) = self.element_bounds.get(&child_id) {
                let main_size = if is_horizontal { child_bounds.size.x } else { child_bounds.size.y };
                child_sizes.push(main_size);
                used_space += main_size;
                
                if let Some(flex_item) = self.items.get(&child_id) {
                    total_flex_grow += flex_item.flex_grow;
                }
            }
        }
        
        // Add gap space
        let total_gap = gap * (container.children.len() as f32 - 1.0).max(0.0);
        used_space += total_gap;
        
        // Calculate remaining space for flex grow
        let remaining_space = (available_main_size - used_space).max(0.0);
        let space_per_grow = if total_flex_grow > 0.0 { remaining_space / total_flex_grow } else { 0.0 };
        
        // Position children
        let mut current_pos = match justify_content {
            JustifyContent::FlexStart => 0.0,
            JustifyContent::FlexEnd => available_main_size - used_space,
            JustifyContent::Center => (available_main_size - used_space) * 0.5,
            JustifyContent::SpaceBetween => 0.0,
            JustifyContent::SpaceAround => {
                let space = (available_main_size - used_space + total_gap) / (container.children.len() as f32 * 2.0);
                space
            },
            JustifyContent::SpaceEvenly => {
                let space = (available_main_size - used_space + total_gap) / (container.children.len() as f32 + 1.0);
                space
            },
        };
        
        for (i, &child_id) in container.children.iter().enumerate() {
            if let Some(child_bounds) = self.element_bounds.get_mut(&child_id) {
                let mut main_size = child_sizes[i];
                
                // Apply flex grow
                if let Some(flex_item) = self.items.get(&child_id) {
                    main_size += flex_item.flex_grow * space_per_grow;
                }
                
                // Position the child
                let (x, y) = if is_horizontal {
                    let cross_pos = match align_items {
                        AlignItems::FlexStart => container_bounds.position.y,
                        AlignItems::FlexEnd => container_bounds.position.y + available_height - child_bounds.size.y,
                        AlignItems::Center => container_bounds.position.y + (available_height - child_bounds.size.y) * 0.5,
                        AlignItems::Stretch => container_bounds.position.y,
                    };
                    
                    (container_bounds.position.x + current_pos, cross_pos)
                } else {
                    let cross_pos = match align_items {
                        AlignItems::FlexStart => container_bounds.position.x,
                        AlignItems::FlexEnd => container_bounds.position.x + available_width - child_bounds.size.x,
                        AlignItems::Center => container_bounds.position.x + (available_width - child_bounds.size.x) * 0.5,
                        AlignItems::Stretch => container_bounds.position.x,
                    };
                    
                    (cross_pos, container_bounds.position.y + current_pos)
                };
                
                child_bounds.position = Vec2::new(x, y);
                
                if is_horizontal {
                    child_bounds.size.x = main_size;
                    if matches!(align_items, AlignItems::Stretch) {
                        child_bounds.size.y = available_height;
                    }
                } else {
                    child_bounds.size.y = main_size;
                    if matches!(align_items, AlignItems::Stretch) {
                        child_bounds.size.x = available_width;
                    }
                }
                
                current_pos += main_size + gap;
                
                // Handle space distribution
                match justify_content {
                    JustifyContent::SpaceBetween if i < container.children.len() - 1 => {
                        let extra_space = (available_main_size - used_space) / (container.children.len() as f32 - 1.0);
                        current_pos += extra_space;
                    },
                    JustifyContent::SpaceAround => {
                        let space = (available_main_size - used_space + total_gap) / (container.children.len() as f32 * 2.0);
                        current_pos += space;
                    },
                    JustifyContent::SpaceEvenly => {
                        let space = (available_main_size - used_space + total_gap) / (container.children.len() as f32 + 1.0);
                        current_pos += space;
                    },
                    _ => {}
                }
            }
        }
    }

    /// Layout grid container
    fn layout_grid_container(
        &mut self,
        container: &LayoutContainer,
        columns: u32,
        rows: u32,
        column_gap: f32,
        row_gap: f32,
    ) {
        if container.children.is_empty() {
            return;
        }

        let container_bounds = &container.bounds;
        let available_width = container_bounds.size.x - column_gap * (columns as f32 - 1.0);
        let available_height = container_bounds.size.y - row_gap * (rows as f32 - 1.0);
        
        let cell_width = available_width / columns as f32;
        let cell_height = available_height / rows as f32;
        
        for (i, &child_id) in container.children.iter().enumerate() {
            let col = (i as u32) % columns;
            let row = (i as u32) / columns;
            
            if let Some(child_bounds) = self.element_bounds.get_mut(&child_id) {
                let x = container_bounds.position.x + col as f32 * (cell_width + column_gap);
                let y = container_bounds.position.y + row as f32 * (cell_height + row_gap);
                
                child_bounds.position = Vec2::new(x, y);
                child_bounds.size = Vec2::new(cell_width, cell_height);
            }
        }
    }

    /// Layout stack container
    fn layout_stack_container(&mut self, container: &LayoutContainer, alignment: Anchor, spacing: f32) {
        if container.children.is_empty() {
            return;
        }

        let container_bounds = &container.bounds;
        let anchor_offset = alignment.to_offset(container_bounds.size);
        
        let mut current_offset = 0.0;
        
        for &child_id in &container.children {
            if let Some(child_bounds) = self.element_bounds.get_mut(&child_id) {
                let base_pos = container_bounds.position + anchor_offset;
                
                // Adjust position based on alignment
                let adjusted_pos = match alignment {
                    Anchor::TopLeft | Anchor::MiddleLeft | Anchor::BottomLeft => base_pos,
                    Anchor::TopCenter | Anchor::MiddleCenter | Anchor::BottomCenter => {
                        Vec2::new(base_pos.x - child_bounds.size.x * 0.5, base_pos.y)
                    },
                    Anchor::TopRight | Anchor::MiddleRight | Anchor::BottomRight => {
                        Vec2::new(base_pos.x - child_bounds.size.x, base_pos.y)
                    },
                };
                
                child_bounds.position = Vec2::new(adjusted_pos.x, adjusted_pos.y + current_offset);
                current_offset += child_bounds.size.y + spacing;
            }
        }
    }

    /// Get the computed bounds for an element
    pub fn get_element_bounds(&self, id: ElementId) -> Option<&UIBounds> {
        self.element_bounds.get(&id)
    }

    /// Update element bounds
    pub fn update_element_bounds(&mut self, id: ElementId, bounds: UIBounds) {
        self.element_bounds.insert(id, bounds);
    }

    /// Remove an element from layout management
    pub fn remove_element(&mut self, id: ElementId) {
        self.element_bounds.remove(&id);
        self.items.remove(&id);
        
        // Remove from containers
        if let Some(container) = self.containers.remove(&id) {
            // Remove as child from parent containers
            for (_, parent_container) in self.containers.iter_mut() {
                parent_container.children.retain(|&child_id| child_id != id);
            }
        }
        
        // Remove as child from all containers
        for container in self.containers.values_mut() {
            container.children.retain(|&child_id| child_id != id);
        }
    }

    /// Clear all layout data
    pub fn clear(&mut self) {
        self.containers.clear();
        self.items.clear();
        self.element_bounds.clear();
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}