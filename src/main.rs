mod common;
mod miniimp;
mod minifun;

use miniimp::ast::Program;
use miniimp::eval::eval_program as eval_miniimp_program;
use miniimp::lexer::tokenize;
use miniimp::parser::parse_tokens;

use minifun::ast::Term;
use minifun::eval::eval_program as eval_minifun_program;




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

                    match eval_miniimp_program(&program, 5) {
                        Ok(result) => {
                            println!("MiniImp program finished successfully.");
                            println!("MiniImp result = {}", result);
                        }
                        Err(error) => {
                            println!("MiniImp runtime error:");
                            println!("{}", error);
                        }
                    }
                }
                Err(error) => {
                    println!("MiniImp parser error:");
                    println!("{}", error);
                }
            }
        }
        Err(error) => {
            println!("MiniImp lexer error:");
            println!("{}", error);
        }
    }

    println!();
    println!("-----------------------------");
    println!();

    // MiniFun test 1
    let term1 = Term::Add(
        Box::new(Term::Int(40)),
        Box::new(Term::Int(2)),
    );

    match eval_minifun_program(&term1) {
        Ok(value) => {
            println!("MiniFun test 1 finished successfully.");
            println!("MiniFun result 1 = {:?}", value);
        }
        Err(error) => {
            println!("MiniFun runtime error in test 1:");
            println!("{}", error);
        }
    }

    println!();
    println!("-----------------------------");
    println!();

    // MiniFun test 2
    let term2 = Term::App(
        Box::new(Term::Fun(
            "x".to_string(),
            Box::new(Term::Add(
                Box::new(Term::Var("x".to_string())),
                Box::new(Term::Int(1)),
            )),
        )),
        Box::new(Term::Int(5)),
    );

    match eval_minifun_program(&term2) {
        Ok(value) => {
            println!("MiniFun test 2 finished successfully.");
            println!("MiniFun result 2 = {:?}", value);
        }
        Err(error) => {
            println!("MiniFun runtime error in test 2:");
            println!("{}", error);
        }
    }
}