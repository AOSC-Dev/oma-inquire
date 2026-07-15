use inquire::{formatter::MultiOptionFormatter, Sort};

fn main() {
    let todos = vec![
        "Fix critical database connection leak",
        "Refactor package manager auth logic",
        "Design gorgeous TUI rendering layouts",
        "Add comprehensive tests for action.rs",
        "Update documentation with exquisite details",
    ];

    let formatter: MultiOptionFormatter<'_, &str> = &|ans| {
        if ans.is_empty() {
            "Empty list".to_string()
        } else if ans.len() == 1 {
            format!("1 task (High priority: {})", ans[0].value)
        } else {
            format!("{} tasks ordered", ans.len(),)
        }
    };

    let ans = Sort::new("Arrange these tasks in order of importance:", todos)
        .with_vim_mode(true)
        .with_formatter(formatter)
        .prompt();

    match ans {
        Ok(sorted_tasks) => {
            println!("Excellent! Here is your structured roadmap:");
            for (index, task) in sorted_tasks.iter().enumerate() {
                println!("  {}. {}", index + 1, task);
            }
        }
        Err(_) => {
            println!("Sorting cancelled. Tasks will remain in their default order.");
        }
    }
}
