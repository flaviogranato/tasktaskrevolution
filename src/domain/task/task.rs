use chrono::NaiveDate;
use crate::domain::shared_kernel::errors::DomainError;
use crate::domain::shared_kernel::validation::DomainValidation;
use validator::Validate;

#[derive(Debug, Clone, Validate)]
pub struct Task {
    #[validate(length(min = 1, max = 100, message = "O título deve ter entre 1 e 100 caracteres"))]
    title: String,
    
    #[validate(length(max = 1000, message = "A descrição não pode ter mais de 1000 caracteres"))]
    description: String,
    
    due_date: NaiveDate,
    completed: bool,
}

impl DomainValidation for Task {}

impl Task {
    pub fn new(title: String, description: String, due_date: NaiveDate) -> Result<Self, DomainError> {
        let task = Self {
            title,
            description,
            due_date,
            completed: false,
        };
        
        task.validate_domain()?;
        Ok(task)
    }

    pub fn set_title(&mut self, title: String) -> Result<(), DomainError> {
        self.title = title;
        self.validate_domain()
    }

    pub fn set_description(&mut self, description: String) -> Result<(), DomainError> {
        self.description = description;
        self.validate_domain()
    }

    pub fn set_due_date(&mut self, due_date: NaiveDate) -> Result<(), DomainError> {
        self.due_date = due_date;
        self.validate_domain()
    }

    pub fn complete(&mut self) -> Result<(), DomainError> {
        self.completed = true;
        Ok(())
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn due_date(&self) -> NaiveDate {
        self.due_date
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_create_task() {
        let title = "Tarefa de teste".to_string();
        let description = "Descrição da tarefa".to_string();
        let due_date = Utc::now().naive_utc().date();

        let task = Task::new(title.clone(), description.clone(), due_date).unwrap();

        assert_eq!(task.title(), &title);
        assert_eq!(task.description(), &description);
        assert_eq!(task.due_date(), due_date);
        assert!(!task.is_completed());
    }

    #[test]
    fn test_update_task() {
        let mut task = Task::new(
            "Tarefa original".to_string(),
            "Descrição original".to_string(),
            Utc::now().naive_utc().date(),
        ).unwrap();

        let new_title = "Novo título".to_string();
        let new_description = "Nova descrição".to_string();
        let new_due_date = Utc::now().naive_utc().date();

        task.set_title(new_title.clone()).unwrap();
        task.set_description(new_description.clone()).unwrap();
        task.set_due_date(new_due_date).unwrap();

        assert_eq!(task.title(), &new_title);
        assert_eq!(task.description(), &new_description);
        assert_eq!(task.due_date(), new_due_date);
    }

    #[test]
    fn test_complete_task() {
        let mut task = Task::new(
            "Tarefa".to_string(),
            "Descrição".to_string(),
            Utc::now().naive_utc().date(),
        ).unwrap();

        assert!(!task.is_completed());
        task.complete().unwrap();
        assert!(task.is_completed());
    }

    #[test]
    fn test_task_validation() {
        // Teste com título vazio
        let result = Task::new(
            "".to_string(),
            "Descrição".to_string(),
            Utc::now().naive_utc().date(),
        );
        assert!(result.is_err());

        // Teste com título muito longo
        let result = Task::new(
            "a".repeat(101),
            "Descrição".to_string(),
            Utc::now().naive_utc().date(),
        );
        assert!(result.is_err());

        // Teste com descrição muito longa
        let result = Task::new(
            "Título".to_string(),
            "a".repeat(1001),
            Utc::now().naive_utc().date(),
        );
        assert!(result.is_err());
    }
}
