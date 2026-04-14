//! Custom widgets for TUI

pub mod dropdown;
pub mod tabs;

// Re-export types for convenience
pub use dropdown::{DropdownSelector, ProviderInfo, DropdownState, DropdownAction};
pub use tabs::{TabBar, TabItem, TabAction, KanbanBoard, KanbanCard, KanbanAction, KanbanColumn, Priority};
