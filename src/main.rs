//! TaskTaskRevolution CLI
//! 
//! Interface de linha de comando para o TTR.
//! A lógica principal está em lib.rs para facilitar testes.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    TaskTaskRevolution::run()
}
