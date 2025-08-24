use crate::condition::examples::run_all_examples_with_integration;

#[tokio::main]
async fn main() -> Result<(), String> {
    println!("🎯 Демонстрация системы условий для торговых сигналов");
    println!("=" .repeat(60));
    
    // Запускаем все примеры
    run_all_examples_with_integration().await?;
    
    println!("\n🎉 Демонстрация завершена!");
    Ok(())
}
