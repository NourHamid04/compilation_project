mod common;
mod miniimp;
mod minifun;

use miniimp::ast::Program;
use miniimp::eval::eval_program as eval_miniimp_program;
use miniimp::lexer::tokenize;
use miniimp::parser::parse_tokens;

use minifun::eval::eval_program as eval_minifun_program;
use minifun::lexer::tokenize as tokenize_minifun;
use minifun::parser::parse_tokens as parse_minifun_tokens;
use minifun::typecheck::typecheck_program;
use miniimp::cfg::build_cfg;
use miniimp::dataflow::{
    defined_variables,
    live_variables,
    reaching_definitions,
};



use miniimp::optimizations::{
    check_undefined_variables,
    constant_folding,
    constant_propagation,
    dead_store_elimination,
    optimize_cfg,
};






fn test_minifun(source: &str) {
    println!("MiniFun source: {}", source);

    match tokenize_minifun(source) {
        Ok(tokens) => {
            println!("MiniFun Tokens: {:?}", tokens);

            match parse_minifun_tokens(tokens) {
                Ok(term) => {
                    println!("MiniFun Parsed AST: {:?}", term);

                    match typecheck_program(&term) {
                        Ok(term_type) => {
                            println!("MiniFun typechecking finished successfully.");
                            println!("MiniFun type = {:?}", term_type);

                            match eval_minifun_program(&term) {
                                Ok(result) => {
                                    println!("MiniFun evaluation finished successfully.");
                                    println!("MiniFun result = {:?}", result);
                                }
                                Err(error) => {
                                    println!("MiniFun runtime error:");
                                    println!("{}", error);
                                }
                            }
                        }
                        Err(error) => {
                            println!("MiniFun typechecking error:");
                            println!("{}", error);
                        }
                    }
                }
                Err(error) => {
                    println!("MiniFun parser error:");
                    println!("{}", error);
                }
            }
        }
        Err(error) => {
            println!("MiniFun lexer error:");
            println!("{}", error);
        }
    }

    println!();
    println!("-----------------------------");
    println!();
}

fn main() {
let source = "
a := 30;
b := 9 - (0 * a);
x := 5;
out := x + b
";
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

                        // Build the CFG from the MiniImp program
                        let cfg = build_cfg(&program);

                        // Print the CFG
                        println!("CFG:");
                        println!("{:#?}", cfg);


                        let defined_cfg = defined_variables(&cfg);
                        println!("Defined variables:");
                        println!("{:#?}", defined_cfg);

                        let live_cfg = live_variables(&cfg);
                        println!("Live variables:");
                        println!("{:#?}", live_cfg);

                        let reaching_cfg = reaching_definitions(&cfg);
                        println!("Reaching definitions:");
                        println!("{:#?}", reaching_cfg);



                        // ===== Fragment 7  =====

                        check_undefined_variables(&cfg);

                        println!();
                        println!("-----------------------------");
                        println!();

                        let (folded_cfg, _) = constant_folding(&cfg);

                        println!("After Constant Folding:");
                        println!("{:#?}", folded_cfg);

                        println!();
                        println!("-----------------------------");
                        println!();

                        let (propagated_cfg, _) = constant_propagation(&cfg);

                        println!("After Constant Propagation:");
                        println!("{:#?}", propagated_cfg);

                        println!();
                        println!("-----------------------------");
                        println!();

                        let (dead_store_cfg, _) = dead_store_elimination(&cfg);

                        println!("After Dead Store Elimination:");
                        println!("{:#?}", dead_store_cfg);

                        println!();
                        println!("-----------------------------");
                        println!();

                        let optimized_cfg = optimize_cfg(&cfg);

                        println!("After Optimization Pipeline:");
                        println!("{:#?}", optimized_cfg);












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
    println!("=============================");
    println!();

    // Well-typed Fragment 3 tests
    test_minifun("fun x : int => x + 1");
    test_minifun("(fun x : int => x + 1) 5");
    test_minifun("let x = 5 in x + 1");
    test_minifun("if true then 10 else 20");
    test_minifun("letfun f x : int -> int = if x < 2 then 1 else x + f (x - 1) in f 4");

    // Ill-typed Fragment 3 tests
    test_minifun("fun x : bool => x + 1");
    test_minifun("if true then 1 else false");
    test_minifun("5 2");
    test_minifun("letfun f x : int = x + 1 in f 5");
}