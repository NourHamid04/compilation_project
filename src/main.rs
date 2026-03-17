mod common;
mod miniimp;
mod minifun;

use miniimp::ast::Program;
use miniimp::eval::eval_program;
use miniimp::lexer::tokenize;
use miniimp::parser::parse_tokens;

fn main() {
    let source = "out := in + 2";

    match tokenize(source) {
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens);

            match parse_tokens(tokens) {
                Ok(cmd) => {
                    println!("Parsed AST: {:?}", cmd);

                    let program = Program {
                        input_var: "in".to_string(),
                        output_var: "out".to_string(),
                        body: cmd,
                    };

                    match eval_program(&program, 5) {
                        Ok(result) => {
                            println!("Program finished successfully.");
                            println!("Result = {}", result);
                        }
                        Err(error) => {
                            println!("Runtime error:");
                            println!("{}", error);
                        }
                    }
                }
                Err(error) => {
                    println!("Parser error:");
                    println!("{}", error);
                }
            }
        }
        Err(error) => {
            println!("Lexer error:");
            println!("{}", error);
        }
    }
}

// fn main() {
//     let source = "out := in + 2;";

//     match tokenize(source) {
//         Ok(tokens) => {
//             println!("Tokens:");
//             for token in tokens {
//                 println!("{:?}", token);
//             }
//         }
//         Err(error) => {
//             println!("{}", error);
//         }
//     }
// }



// fn main() {
//     let program = Program {
//         input_var: "in".to_string(),
//         output_var: "out".to_string(),

//         body: Cmd::Assign(
//             "out".to_string(),
//             Expr::Add(
//                 Box::new(Expr::Var("in".to_string())),
//                 Box::new(Expr::Int(2)),
//             ),
//         ),
//     };

//     // Run the program with input = 5
//     match eval_program(&program, 5) {
//         Ok(result) => {
//             println!("Program finished successfully.");
//             println!("Result = {}", result);
//         }

//         Err(error) => {
//             println!("Program failed with error:");
//             println!("{}", error);
//         }
//     }
// }