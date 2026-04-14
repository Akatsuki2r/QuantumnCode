//! Custom widgets for TUI

pub mod dropdown;
pub mod tabs;

// Re-export types for convenience
pub use dropdown::{DropdownAction, DropdownSelector, DropdownState, ProviderInfo};
pub use tabs::{
    KanbanAction, KanbanBoard, KanbanCard, KanbanColumn, Priority, TabAction, TabBar, TabItem,
};
