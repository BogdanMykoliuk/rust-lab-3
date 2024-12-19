use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, serde::ts_seconds};

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: u32,
    title: String,
    description: String,
    completed: bool,
    #[serde(with = "ts_seconds")]
    created_at: DateTime<Utc>,
    user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

struct TodoApp {
    tasks: HashMap<u32, Task>,
    users: HashMap<String, User>,
    current_user: Option<String>,
    next_task_id: u32,
}

impl TodoApp {
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            users: HashMap::new(),
            current_user: None,
            next_task_id: 1,
        }
    }

    fn register(&mut self, username: String, password: String) -> Result<(), &'static str> {
        if self.users.contains_key(&username) {
            return Err("Username already exists");
        }

        self.users.insert(username.clone(), User {
            username,
            password,
        });
        self.save_users().unwrap();
        Ok(())
    }

    fn login(&mut self, username: String, password: String) -> Result<(), &'static str> {
        match self.users.get(&username) {
            Some(user) if user.password == password => {
                self.current_user = Some(username);
                Ok(())
            }
            _ => Err("Invalid username or password"),
        }
    }

    fn add_task(&mut self, title: String, description: String) -> Result<(), &'static str> {
        let user_id = self.current_user.clone().ok_or("Not logged in")?;

        let task = Task {
            id: self.next_task_id,
            title,
            description,
            completed: false,
            created_at: Utc::now(),
            user_id,
        };

        self.tasks.insert(self.next_task_id, task);
        self.next_task_id += 1;
        self.save_tasks().unwrap();
        Ok(())
    }

    fn complete_task(&mut self, task_id: u32) -> Result<(), &'static str> {
        let user_id = self.current_user.clone().ok_or("Not logged in")?;

        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;
        if task.user_id != user_id {
            return Err("Not authorized to modify this task");
        }

        task.completed = true;
        self.save_tasks().unwrap();
        Ok(())
    }

    fn edit_task(&mut self, task_id: u32, title: String, description: String) -> Result<(), &'static str> {
        let user_id = self.current_user.clone().ok_or("Not logged in")?;

        let task = self.tasks.get_mut(&task_id).ok_or("Task not found")?;
        if task.user_id != user_id {
            return Err("Not authorized to modify this task");
        }

        task.title = title;
        task.description = description;
        self.save_tasks().unwrap();
        Ok(())
    }

    fn delete_task(&mut self, task_id: u32) -> Result<(), &'static str> {
        let user_id = self.current_user.clone().ok_or("Not logged in")?;

        let task = self.tasks.get(&task_id).ok_or("Task not found")?;
        if task.user_id != user_id {
            return Err("Not authorized to delete this task");
        }

        self.tasks.remove(&task_id);
        self.save_tasks().unwrap();
        Ok(())
    }

    fn list_tasks(&self) -> Result<Vec<&Task>, &'static str> {
        let user_id = self.current_user.as_ref().ok_or("Not logged in")?;

        Ok(self.tasks.values()
            .filter(|task| task.user_id == *user_id)
            .collect())
    }

    fn save_tasks(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.tasks)?;
        fs::write("tasks.json", json)?;
        Ok(())
    }

    fn load_tasks(&mut self) -> io::Result<()> {
        match fs::read_to_string("tasks.json") {
            Ok(contents) => {
                self.tasks = serde_json::from_str(&contents)?;
                self.next_task_id = self.tasks.keys().max().map_or(1, |max| max + 1);
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn save_users(&self) -> io::Result<()> {
        let json = serde_json::to_string(&self.users)?;
        fs::write("users.json", json)?;
        Ok(())
    }

    fn load_users(&mut self) -> io::Result<()> {
        match fs::read_to_string("users.json") {
            Ok(contents) => {
                self.users = serde_json::from_str(&contents)?;
                Ok(())
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

fn main() {
    let mut app = TodoApp::new();
    app.load_tasks().unwrap();
    app.load_users().unwrap();

    loop {
        if app.current_user.is_none() {
            println!("\nWelcome to Todo App!");
            println!("1. Login");
            println!("2. Register");
            println!("3. Exit");

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();

            match choice.trim() {
                "1" => {
                    print!("Username: ");
                    io::stdout().flush().unwrap();
                    let mut username = String::new();
                    io::stdin().read_line(&mut username).unwrap();

                    print!("Password: ");
                    io::stdout().flush().unwrap();
                    let mut password = String::new();
                    io::stdin().read_line(&mut password).unwrap();

                    match app.login(username.trim().to_string(), password.trim().to_string()) {
                        Ok(_) => println!("Login successful!"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "2" => {
                    print!("Username: ");
                    io::stdout().flush().unwrap();
                    let mut username = String::new();
                    io::stdin().read_line(&mut username).unwrap();

                    print!("Password: ");
                    io::stdout().flush().unwrap();
                    let mut password = String::new();
                    io::stdin().read_line(&mut password).unwrap();

                    match app.register(username.trim().to_string(), password.trim().to_string()) {
                        Ok(_) => println!("Registration successful!"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "3" => break,
                _ => println!("Invalid choice"),
            }
        } else {
            println!("\nTodo App Menu:");
            println!("1. Add Task");
            println!("2. List Tasks");
            println!("3. Complete Task");
            println!("4. Edit Task");
            println!("5. Delete Task");
            println!("6. Logout");

            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();

            match choice.trim() {
                "1" => {
                    print!("Title: ");
                    io::stdout().flush().unwrap();
                    let mut title = String::new();
                    io::stdin().read_line(&mut title).unwrap();

                    print!("Description: ");
                    io::stdout().flush().unwrap();
                    let mut description = String::new();
                    io::stdin().read_line(&mut description).unwrap();

                    match app.add_task(title.trim().to_string(), description.trim().to_string()) {
                        Ok(_) => println!("Task added successfully!"),
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "2" => {
                    match app.list_tasks() {
                        Ok(tasks) => {
                            for task in tasks {
                                println!("\nID: {}", task.id);
                                println!("Title: {}", task.title);
                                println!("Description: {}", task.description);
                                println!("Status: {}", if task.completed { "Completed" } else { "Pending" });
                                println!("Created: {}", task.created_at);
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
                "3" => {
                    print!("Task ID: ");
                    io::stdout().flush().unwrap();
                    let mut id = String::new();
                    io::stdin().read_line(&mut id).unwrap();

                    match id.trim().parse() {
                        Ok(task_id) => {
                            match app.complete_task(task_id) {
                                Ok(_) => println!("Task marked as completed!"),
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                        Err(_) => println!("Invalid task ID"),
                    }
                }
                "4" => {
                    print!("Task ID: ");
                    io::stdout().flush().unwrap();
                    let mut id = String::new();
                    io::stdin().read_line(&mut id).unwrap();

                    print!("New Title: ");
                    io::stdout().flush().unwrap();
                    let mut title = String::new();
                    io::stdin().read_line(&mut title).unwrap();

                    print!("New Description: ");
                    io::stdout().flush().unwrap();
                    let mut description = String::new();
                    io::stdin().read_line(&mut description).unwrap();

                    match id.trim().parse() {
                        Ok(task_id) => {
                            match app.edit_task(task_id, title.trim().to_string(), description.trim().to_string()) {
                                Ok(_) => println!("Task updated successfully!"),
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                        Err(_) => println!("Invalid task ID"),
                    }
                }
                "5" => {
                    print!("Task ID: ");
                    io::stdout().flush().unwrap();
                    let mut id = String::new();
                    io::stdin().read_line(&mut id).unwrap();

                    match id.trim().parse() {
                        Ok(task_id) => {
                            match app.delete_task(task_id) {
                                Ok(_) => println!("Task deleted successfully!"),
                                Err(e) => println!("Error: {}", e),
                            }
                        }
                        Err(_) => println!("Invalid task ID"),
                    }
                }
                "6" => {
                    app.current_user = None;
                    println!("Logged out successfully!");
                }
                _ => println!("Invalid choice"),
            }
        }
    }
}
