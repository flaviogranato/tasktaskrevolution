use crate::domain::agile::agile_models::*;
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

/// KanbanBoard manages Kanban board functionality
#[derive(Debug, Clone)]
pub struct KanbanBoardManager {
    boards: HashMap<String, KanbanBoard>,
    task_movements: HashMap<String, Vec<TaskMovement>>,
}

/// Task movement tracking
#[derive(Debug, Clone)]
pub struct TaskMovement {
    pub task_id: String,
    pub from_column: String,
    pub to_column: String,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
}

impl KanbanBoardManager {
    /// Create a new KanbanBoardManager
    pub fn new() -> Self {
        Self {
            boards: HashMap::new(),
            task_movements: HashMap::new(),
        }
    }

    /// Create a new Kanban board
    pub fn create_board(&mut self, name: String, project_id: String) -> String {
        let board = KanbanBoard::new(name, project_id);
        let board_id = board.id.clone();
        self.boards.insert(board_id.clone(), board);
        board_id
    }

    /// Get board by ID
    pub fn get_board(&self, board_id: &str) -> Option<&KanbanBoard> {
        self.boards.get(board_id)
    }

    /// Get board by ID (mutable)
    pub fn get_board_mut(&mut self, board_id: &str) -> Option<&mut KanbanBoard> {
        self.boards.get_mut(board_id)
    }

    /// List boards for a project
    pub fn list_boards(&self, project_id: &str) -> Vec<&KanbanBoard> {
        self.boards.values()
            .filter(|board| board.project_id == project_id)
            .collect()
    }

    /// Add column to board
    pub fn add_column(&mut self, board_id: &str, column: KanbanColumn) -> Result<(), String> {
        if let Some(board) = self.boards.get_mut(board_id) {
            board.add_column(column);
            Ok(())
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Remove column from board
    pub fn remove_column(&mut self, board_id: &str, column_id: &str) -> Result<(), String> {
        if let Some(board) = self.boards.get_mut(board_id) {
            board.remove_column(column_id);
            Ok(())
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Move task between columns
    pub fn move_task(
        &mut self,
        board_id: &str,
        task_id: &str,
        from_column: &str,
        to_column: &str,
        user_id: Option<String>,
    ) -> Result<(), String> {
        if let Some(board) = self.boards.get_mut(board_id) {
            // Check WIP limits before moving
            if let Some(target_col) = board.columns.iter().find(|c| c.id == to_column) {
                if let Some(limit) = board.wip_limits.get(&target_col.id) {
                    if target_col.tasks.len() as u32 >= *limit {
                        return Err(format!("WIP limit exceeded for column {}", target_col.name));
                    }
                }
            }

            board.move_task(task_id, from_column, to_column)?;

            // Record movement
            let movement = TaskMovement {
                task_id: task_id.to_string(),
                from_column: from_column.to_string(),
                to_column: to_column.to_string(),
                timestamp: Utc::now(),
                user_id,
            };

            self.task_movements.entry(board_id.to_string())
                .or_insert_with(Vec::new)
                .push(movement);

            Ok(())
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Set WIP limit for column
    pub fn set_wip_limit(&mut self, board_id: &str, column_id: &str, limit: u32) -> Result<(), String> {
        if let Some(board) = self.boards.get_mut(board_id) {
            if board.columns.iter().any(|c| c.id == column_id) {
                board.wip_limits.insert(column_id.to_string(), limit);
                board.updated_at = Utc::now();
                Ok(())
            } else {
                Err(format!("Column {} not found", column_id))
            }
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Remove WIP limit for column
    pub fn remove_wip_limit(&mut self, board_id: &str, column_id: &str) -> Result<(), String> {
        if let Some(board) = self.boards.get_mut(board_id) {
            board.wip_limits.remove(column_id);
            board.updated_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Check WIP limit violations
    pub fn check_wip_violations(&self, board_id: &str) -> Result<Vec<String>, String> {
        if let Some(board) = self.boards.get(board_id) {
            Ok(board.check_wip_limits())
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Calculate flow metrics for board
    pub fn calculate_flow_metrics(&mut self, board_id: &str) -> Result<FlowMetrics, String> {
        if let Some(board) = self.boards.get_mut(board_id) {
            let movements = self.task_movements.get(board_id).cloned().unwrap_or_default();
            
            if movements.is_empty() {
                return Ok(FlowMetrics::default());
            }

            // Calculate cycle time (time from first column to last column)
            let cycle_times: Vec<Duration> = movements.iter()
                .filter(|m| m.from_column != m.to_column)
                .map(|m| Duration::seconds(1)) // Simplified for now
                .collect();

            let cycle_time = if !cycle_times.is_empty() {
                Duration::seconds(
                    cycle_times.iter().map(|d| d.num_seconds()).sum::<i64>() / cycle_times.len() as i64
                )
            } else {
                Duration::zero()
            };

            // Calculate throughput (tasks completed per day)
            let completed_tasks = movements.iter()
                .filter(|m| m.to_column == "Done" || m.to_column == "Completed")
                .count();

            let throughput = if !movements.is_empty() {
                let time_span = movements.last().unwrap().timestamp - movements.first().unwrap().timestamp;
                if time_span.num_days() > 0 {
                    completed_tasks as f64 / time_span.num_days() as f64
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // Calculate average WIP
            let total_tasks: usize = board.columns.iter().map(|c| c.tasks.len()).sum();
            let average_wip = total_tasks as f64;

            // Calculate flow efficiency (simplified)
            let flow_efficiency = if cycle_time.num_seconds() > 0 {
                (completed_tasks as f64 / cycle_time.num_days() as f64).min(1.0)
            } else {
                0.0
            };

            // Identify bottleneck columns
            let bottleneck_columns = board.columns.iter()
                .filter(|c| {
                    if let Some(limit) = board.wip_limits.get(&c.id) {
                        c.tasks.len() as u32 >= *limit
                    } else {
                        false
                    }
                })
                .map(|c| c.id.clone())
                .collect();

            let metrics = FlowMetrics {
                cycle_time,
                lead_time: cycle_time, // Simplified
                throughput,
                flow_efficiency,
                bottleneck_columns,
                average_wip,
            };

            board.flow_metrics = metrics.clone();
            Ok(metrics)
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Get task movement history
    pub fn get_task_movements(&self, board_id: &str) -> Option<&Vec<TaskMovement>> {
        self.task_movements.get(board_id)
    }

    /// Get task cycle time
    pub fn get_task_cycle_time(&self, board_id: &str, task_id: &str) -> Option<Duration> {
        if let Some(movements) = self.task_movements.get(board_id) {
            let task_movements: Vec<&TaskMovement> = movements.iter()
                .filter(|m| m.task_id == task_id)
                .collect();

            if task_movements.len() >= 2 {
                let first_movement = task_movements.first().unwrap();
                let last_movement = task_movements.last().unwrap();
                Some(last_movement.timestamp - first_movement.timestamp)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get board statistics
    pub fn get_board_statistics(&self, board_id: &str) -> Result<BoardStatistics, String> {
        if let Some(board) = self.boards.get(board_id) {
            let total_tasks: usize = board.columns.iter().map(|c| c.tasks.len()).sum();
            let wip_violations = board.check_wip_limits().len();
            
            let column_stats: Vec<ColumnStatistics> = board.columns.iter().map(|col| {
                ColumnStatistics {
                    column_id: col.id.clone(),
                    column_name: col.name.clone(),
                    task_count: col.tasks.len(),
                    wip_limit: board.wip_limits.get(&col.id).copied(),
                    utilization: if let Some(limit) = board.wip_limits.get(&col.id) {
                        col.tasks.len() as f64 / *limit as f64
                    } else {
                        0.0
                    },
                }
            }).collect();

            Ok(BoardStatistics {
                board_id: board_id.to_string(),
                board_name: board.name.clone(),
                total_columns: board.columns.len(),
                total_tasks,
                wip_violations,
                column_statistics: column_stats,
                flow_metrics: board.flow_metrics.clone(),
            })
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Delete board
    pub fn delete_board(&mut self, board_id: &str) -> Result<(), String> {
        if self.boards.remove(board_id).is_some() {
            self.task_movements.remove(board_id);
            Ok(())
        } else {
            Err(format!("Board {} not found", board_id))
        }
    }

    /// Get all boards
    pub fn get_all_boards(&self) -> Vec<&KanbanBoard> {
        self.boards.values().collect()
    }

    /// Create default board with standard columns
    pub fn create_default_board(&mut self, name: String, project_id: String) -> String {
        let board_id = self.create_board(name.clone(), project_id.clone());
        
        let default_columns = vec![
            KanbanColumn {
                id: "todo".to_string(),
                name: "To Do".to_string(),
                position: 1,
                wip_limit: None,
                tasks: Vec::new(),
                color: Some("#e3f2fd".to_string()),
                description: Some("Tasks to be started".to_string()),
            },
            KanbanColumn {
                id: "in_progress".to_string(),
                name: "In Progress".to_string(),
                position: 2,
                wip_limit: Some(3),
                tasks: Vec::new(),
                color: Some("#fff3e0".to_string()),
                description: Some("Tasks currently being worked on".to_string()),
            },
            KanbanColumn {
                id: "review".to_string(),
                name: "Review".to_string(),
                position: 3,
                wip_limit: Some(2),
                tasks: Vec::new(),
                color: Some("#f3e5f5".to_string()),
                description: Some("Tasks under review".to_string()),
            },
            KanbanColumn {
                id: "done".to_string(),
                name: "Done".to_string(),
                position: 4,
                wip_limit: None,
                tasks: Vec::new(),
                color: Some("#e8f5e8".to_string()),
                description: Some("Completed tasks".to_string()),
            },
        ];

        for column in default_columns {
            self.add_column(&board_id, column).unwrap();
        }

        board_id
    }
}

/// Board statistics
#[derive(Debug, Clone)]
pub struct BoardStatistics {
    pub board_id: String,
    pub board_name: String,
    pub total_columns: usize,
    pub total_tasks: usize,
    pub wip_violations: usize,
    pub column_statistics: Vec<ColumnStatistics>,
    pub flow_metrics: FlowMetrics,
}

/// Column statistics
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub column_id: String,
    pub column_name: String,
    pub task_count: usize,
    pub wip_limit: Option<u32>,
    pub utilization: f64,
}

impl Default for KanbanBoardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kanban_board_manager_creation() {
        let manager = KanbanBoardManager::new();
        assert_eq!(manager.boards.len(), 0);
    }

    #[test]
    fn test_create_board() {
        let mut manager = KanbanBoardManager::new();
        let board_id = manager.create_board("Test Board".to_string(), "proj1".to_string());
        
        assert!(manager.get_board(&board_id).is_some());
        let board = manager.get_board(&board_id).unwrap();
        assert_eq!(board.name, "Test Board");
        assert_eq!(board.project_id, "proj1");
    }

    #[test]
    fn test_add_column() {
        let mut manager = KanbanBoardManager::new();
        let board_id = manager.create_board("Test Board".to_string(), "proj1".to_string());
        
        let column = KanbanColumn {
            id: "col1".to_string(),
            name: "To Do".to_string(),
            position: 1,
            wip_limit: Some(5),
            tasks: Vec::new(),
            color: Some("#blue".to_string()),
            description: Some("Tasks to be done".to_string()),
        };

        assert!(manager.add_column(&board_id, column).is_ok());
        
        let board = manager.get_board(&board_id).unwrap();
        assert_eq!(board.columns.len(), 1);
    }

    #[test]
    fn test_move_task() {
        let mut manager = KanbanBoardManager::new();
        let board_id = manager.create_board("Test Board".to_string(), "proj1".to_string());
        
        // Add columns
        let todo_col = KanbanColumn {
            id: "todo".to_string(),
            name: "To Do".to_string(),
            position: 1,
            wip_limit: None,
            tasks: vec!["task1".to_string()],
            color: None,
            description: None,
        };
        
        let progress_col = KanbanColumn {
            id: "progress".to_string(),
            name: "In Progress".to_string(),
            position: 2,
            wip_limit: Some(3),
            tasks: Vec::new(),
            color: None,
            description: None,
        };

        manager.add_column(&board_id, todo_col).unwrap();
        manager.add_column(&board_id, progress_col).unwrap();

        // Move task
        assert!(manager.move_task(&board_id, "task1", "todo", "progress", None).is_ok());
        
        let board = manager.get_board(&board_id).unwrap();
        let todo_column = board.columns.iter().find(|c| c.id == "todo").unwrap();
        let progress_column = board.columns.iter().find(|c| c.id == "progress").unwrap();
        
        assert!(!todo_column.tasks.contains(&"task1".to_string()));
        assert!(progress_column.tasks.contains(&"task1".to_string()));
    }

    #[test]
    fn test_wip_limit_violation() {
        let mut manager = KanbanBoardManager::new();
        let board_id = manager.create_board("Test Board".to_string(), "proj1".to_string());
        
        let column = KanbanColumn {
            id: "col1".to_string(),
            name: "To Do".to_string(),
            position: 1,
            wip_limit: Some(2),
            tasks: vec!["task1".to_string(), "task2".to_string(), "task3".to_string()],
            color: None,
            description: None,
        };

        manager.add_column(&board_id, column).unwrap();
        manager.set_wip_limit(&board_id, "col1", 2).unwrap();

        let violations = manager.check_wip_violations(&board_id).unwrap();
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_create_default_board() {
        let mut manager = KanbanBoardManager::new();
        let board_id = manager.create_default_board("Default Board".to_string(), "proj1".to_string());
        
        let board = manager.get_board(&board_id).unwrap();
        assert_eq!(board.columns.len(), 4);
        assert_eq!(board.name, "Default Board");
    }
}
